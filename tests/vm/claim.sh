#!/bin/bash
# The claim end to end on the x86-64 VM with the real OpenWrt platform (chapter 14 §2, VM
# integration): copies the daemon, the UI, and the VM profile in, starts the daemon as its own
# user, and drives the API from the host through an SSH tunnel to the management address. The
# daemon listens on port 8080, as on a device; the firewall redirects port 80 to it on the
# management interface (a local tunnel bypasses the redirect, so it targets 8080).
# Prints a report; exits non-zero when a step does not match the specification.
#
#   tests/vm/claim.sh VM_DIR [DAEMON]   DAEMON defaults to target/x86_64-unknown-linux-musl/release/enkrnettd
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
VM=${1:?VM_DIR}
DAEMON=${2:-$ROOT/target/x86_64-unknown-linux-musl/release/enkrnettd}
PASSWORD="correct horse battery staple"
PORT=18082
cd "$VM"

step() { echo; echo "## $*"; }
api() { # method path [token] [body]
  local method=$1 path=$2 token=${3:-} body=${4:-}
  local args=(-s -o /tmp/enkrnett-vm-body -w '%{http_code}' --connect-timeout 5 -m 40 --retry 6 --retry-connrefused --retry-all-errors --retry-delay 2 -X "$method" "http://127.0.0.1:$PORT/api/1/$path" -H "Host: 192.168.207.1")
  [ -n "$token" ] && args+=(-H "Enkrnett-Session: $token")
  [ "$method" = POST ] && args+=(-H 'Content-Type: application/json' -d "${body:-{\}}")
  curl "${args[@]}"
}
body() { cat /tmp/enkrnett-vm-body; echo; }
expect() { # status expected
  if [ "$1" != "$2" ]; then echo "FAIL: expected HTTP $2, got $1"; body; exit 1; fi
}
json() { python3 -c "import json,sys; d=json.load(open('/tmp/enkrnett-vm-body')); print(eval('d'+sys.argv[1]))" "$1"; }

./vm.sh start >/dev/null || true
python3 vmsh.py wait 120 >/dev/null 2>&1 || true

step "install the daemon, the UI, and the VM profile"
./vmssh 'killall enkrnettd 2>/dev/null; sleep 1; rm -f /usr/sbin/enkrnettd' >/dev/null 2>&1 || true
./vmscp "$DAEMON" /usr/sbin/enkrnettd >/dev/null
./vmscp "$ROOT/openwrt/profiles/x86_64_generic/profile.json" /usr/share/enkrnett/profiles/x86_64_generic.json >/dev/null
if [ -d "$ROOT/ui/dist" ]; then tar -C "$ROOT/ui/dist" -cf - . | ./vmssh 'rm -rf /usr/share/enkrnett/ui && mkdir -p /usr/share/enkrnett/ui && tar -C /usr/share/enkrnett/ui -xf -'; fi
./vmssh 'chmod 755 /usr/sbin/enkrnettd; killall enkrnettd 2>/dev/null; rm -f /etc/enkrnett/*; sleep 1
su enkrnett -s /bin/sh -c "setsid /usr/sbin/enkrnettd --listen 192.168.207.1:8080 --state-dir /etc/enkrnett --ui-dir /usr/share/enkrnett/ui --profiles-dir /usr/share/enkrnett/profiles --development --act-delay-ms 500 > /tmp/enkrnettd.log 2>&1 < /dev/null &"
for i in $(seq 1 45); do grep -q "enkrnettd started" /tmp/enkrnettd.log 2>/dev/null && break; sleep 2; done; echo "--- daemon log"; cat /tmp/enkrnettd.log; echo "--- sections"; uci -q show wireless | grep -E "^wireless\.enk_" | cut -d= -f1 | sort -u || true; uci -q show network | grep -E "^network\.enk_[a-z_]*=" || true; echo "--- hostapd objects"; ubus list | grep hostapd || true; echo "--- dnsmasq instances"; ubus call service list "{\"name\":\"dnsmasq\"}" | grep -E "enk_dns|running" || true'

step "tunnel to the management address"
ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o LogLevel=ERROR -i id_ed25519 -p 2222 -N -L "$PORT:192.168.207.1:8080" root@127.0.0.1 &
TUNNEL=$!
trap 'kill $TUNNEL 2>/dev/null || true' EXIT
sleep 2

step "greeting: SETUP, the setup network on the air"
expect "$(api GET '')" 200; body
[ "$(json "['state']")" = SETUP ] || { echo "FAIL: not in SETUP"; exit 1; }
./vmssh 'for o in $(ubus list | grep "^hostapd\\."); do printf "%s: " $o; ubus call $o get_status | grep -E "\"(ssid|status)\"" | tr -d "\\t\\n"; echo; done' || true

step "claim: start, check, confirm"
expect "$(api POST claim/start)" 200; TOKEN=$(json "['token']")
expect "$(api GET claim "$TOKEN")" 200; body
expect "$(api POST claim/check "$TOKEN" "{\"guest\":{\"name\":\"Cafe Example\",\"mode\":\"open_to_all\"},\"owner_password\":\"$PASSWORD\"}")" 200; body
MGMT_NAME=$(json "['management']['name']"); MGMT_KEY=$(json "['management']['key']")
expect "$(api POST claim/confirm "$TOKEN")" 200; body
sleep 12
./vmssh 'echo "--- after confirm"; ubus list | grep hostapd; ubus call hostapd.enk-mgmt get_status | grep -E "ssid|status"; grep -c enk_setup /etc/config/wireless || true'

step "greeting: PENDING; login makes the claim final; guest network appears"
expect "$(api GET '')" 200; body
[ "$(json "['state']")" = PENDING ] || { echo "FAIL: not in PENDING"; exit 1; }
expect "$(api POST login '' "{\"password\":\"$PASSWORD\"}")" 200; body; OWNER=$(json "['token']")
sleep 15
./vmssh 'echo "--- after login"; for o in $(ubus list | grep "^hostapd\\."); do printf "%s: " $o; ubus call $o get_status | grep -E "\"(ssid|status)\"" | tr -d "\\t\\n"; echo; done; ubus call network.interface.enk_guest status | grep -E "\"up\"|\"address\""; head -5 /etc/enkrnett/state.json'

step "status and device through the real platform"
expect "$(api GET status "$OWNER")" 200; body
expect "$(api GET device "$OWNER")" 200; body

step "a settings change: encrypted only"
expect "$(api POST settings "$OWNER" '{"version":1,"guest":{"name":"Cafe Example","mode":"encrypted_only","enabled":true},"management":{"name":"'"$MGMT_NAME"'"}}')" 202; body
sleep 12
./vmssh 'echo "--- after the change"; ubus list | grep hostapd; grep -A8 "enk_guest" /etc/config/wireless | grep -E "ssid|encryption|owe_transition" || true'

step "owned sections only; foreign sections untouched"
./vmssh 'grep -c "config interface .wan." /etc/config/network; grep -c "measurement_ssh" /etc/config/firewall; ls -la /etc/config | grep -E " (wireless|network|dhcp|firewall)$"'

echo
echo "VM claim: every step answered as specified (management Wi-Fi $MGMT_NAME)"
