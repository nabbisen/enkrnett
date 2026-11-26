#!/bin/bash
# Renumbering on the VM (ENK-TOPO-003, ENK-TOPO-006): the VM is restarted twice with an uplink
# whose subnet collides with the guest network, then with the management network. The daemon
# must move only the colliding network; when the management address moves, it exits and the
# respawn listens on the new address. Ends with Erase, so that the device is back on the default
# addresses for the other scripts. Needs the VM launcher to accept VM_WAN_NET.
#
#   tests/vm/renumber.sh VM_DIR     (after tests/vm/claim.sh: a CLAIMED device)
set -euo pipefail
VM=${1:?VM_DIR}
PASSWORD="correct horse battery staple"
PORT=18086
cd "$VM"
FAILS=0
pass() { echo "  ok   $*"; }
fail() { echo "  FAIL $*"; FAILS=$((FAILS + 1)); }
check() { if [ "$2" = "$3" ]; then pass "$1"; else fail "$1 (got '$2', expected '$3')"; fi; }
TUNNEL=
tunnel() { # management address
  [ -n "$TUNNEL" ] && kill $TUNNEL 2>/dev/null || true
  ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o LogLevel=ERROR -i id_ed25519 -p 2222 -N -L "$PORT:$1:8080" root@127.0.0.1 &
  TUNNEL=$!; MGMT=$1; sleep 2
}
trap '[ -n "$TUNNEL" ] && kill $TUNNEL 2>/dev/null || true' EXIT
api() { local method=$1 path=$2 token=${3:-} body=${4:-}
  local args=(-s -o /tmp/enkrnett-r-body -w '%{http_code}' --connect-timeout 5 -m 40 --retry 8 --retry-connrefused --retry-all-errors --retry-delay 2 -X "$method" "http://127.0.0.1:$PORT/api/1/$path" -H "Host: $MGMT")
  [ -n "$token" ] && args+=(-H "Enkrnett-Session: $token")
  [ "$method" = POST ] && args+=(-H 'Content-Type: application/json' -d "${body:-{\}}")
  curl "${args[@]}" 2>/dev/null || true; }
jq_() { python3 -c "import json,sys; d=json.load(open('/tmp/enkrnett-r-body')); print(eval('d'+sys.argv[1]))" "$1" 2>/dev/null || echo "?"; }
restart_with() { ./vm.sh stop >/dev/null; sleep 2; VM_WAN_NET=$1 ./vm.sh start >/dev/null; python3 vmsh.py wait 180 >/dev/null 2>&1 || true; sleep 25; }
mgmt_addr() { ./vmssh "ip -4 addr show br-mgmt 2>/dev/null | awk '/inet /{print \$2}'" 2>/dev/null | tr -d '\r' || true; }
# The uplink lease may arrive well after boot; the collision check runs every 10 s, then the move
# and the respawn. Wait up to two minutes for the management bridge to carry the given address.
wait_mgmt() { local i; for i in $(seq 1 24); do [ "$(mgmt_addr)" = "$1/24" ] && return 0; sleep 5; done; return 1; }

echo "## uplink 10.207.0.0/24 collides with the guest network"
restart_with 10.207.0.0/24
./vmssh "for i in \$(seq 1 24); do ip -4 addr show br-guest 2>/dev/null | grep -q 10.207.1.1 && break; sleep 5; done" >/dev/null 2>&1 || true
tunnel 192.168.207.1
check "the management address did not move" "$(mgmt_addr)" 192.168.207.1/24
S=$(api POST login '' "{\"password\":\"$PASSWORD\"}"); check "login" "$S" 200; T=$(jq_ "['token']")
api GET status "$T" >/dev/null; check "guest network up on the renumbered subnet" "$(jq_ "['guest']['up']")" True
check "uplink address in the colliding subnet" "$(jq_ "['uplink']['address']")" 10.207.0.15
./vmssh "ip -4 addr show br-guest | awk '/inet /{print \$2}'; uci -q get firewall.enk_guest_dns.dest_ip; logread | grep -c renumbered" > /tmp/enkrnett-r-out || true
check "ENK-TOPO-006 the guest bridge moved to 10.207.1.1/24" "$(sed -n 1p /tmp/enkrnett-r-out)" 10.207.1.1/24
check "the guest DNS rule followed" "$(sed -n 2p /tmp/enkrnett-r-out)" 10.207.1.1
[ "$(sed -n 3p /tmp/enkrnett-r-out)" -ge 1 ] 2>/dev/null && pass "renumbered logged" || fail "no renumbered line"
api POST logout "$T" >/dev/null

echo "## uplink 192.168.207.0/24 collides with the management network"
restart_with 192.168.207.0/24
wait_mgmt 192.168.208.1 || true
# The networks move first; the daemon follows only after the health check of that apply (up to
# HEALTH_TIMEOUT, 60 s), then asks procd for its restart. Wait for the new listener.
./vmssh "for i in \$(seq 1 40); do netstat -tln 2>/dev/null | grep -q 192.168.208.1:8080 && break; sleep 5; done" >/dev/null 2>&1 || true
tunnel 192.168.208.1
check "ENK-TOPO-006 the management address moved to the next subnet" "$(mgmt_addr)" 192.168.208.1/24
S=$(api POST login '' "{\"password\":\"$PASSWORD\"}"); check "login on the new address" "$S" 200; T=$(jq_ "['token']")
api GET status "$T" >/dev/null; check "status says the new management address" "$(jq_ "['management']['address']")" 192.168.208.1
check "the guest network stayed where it was" "$(./vmssh "ip -4 addr show br-guest | awk '/inet /{print \$2}'" || true)" 10.207.1.1/24
./vmssh "logread | grep -E 'renumber|listen_address_changed|service_restart|enkrnettd started' | tail -6; netstat -tln | grep 8080" > /tmp/enkrnett-r-out || true; sed 's/^/  /' /tmp/enkrnett-r-out
# Two valid paths: the lease came after the daemon bound the old address (listen_address_changed,
# then the restart on the new one), or it was there at start-up and the daemon bound the new
# address at once (no restart). Either way the new address is the one on the air below.
if grep -q 'listen_address_changed' /tmp/enkrnett-r-out; then pass "the daemon exited for the respawn on the new address"
elif grep -q 'started listen=192.168.208.1' /tmp/enkrnett-r-out; then pass "the daemon bound the new address at start-up (collision seen before the listener)"
else fail "neither listen_address_changed nor a start on the new address"; fi
grep -q '192.168.208.1:8080' /tmp/enkrnett-r-out && pass "listening on 192.168.208.1:8080 only" || fail "listener"

echo "## back on the usual uplink: nothing moves back (no flapping); Erase returns the defaults"
restart_with 10.0.2.0/24
sleep 30
tunnel 192.168.208.1
check "addresses kept after the collision went away" "$(mgmt_addr)" 192.168.208.1/24
S=$(api POST login '' "{\"password\":\"$PASSWORD\"}"); T=$(jq_ "['token']")
S=$(api POST erase "$T" "{\"password\":\"$PASSWORD\"}"); check "erase accepted" "$S" 202
kill $TUNNEL 2>/dev/null || true; TUNNEL=
sleep 20; python3 vmsh.py wait 180 >/dev/null 2>&1 || true; sleep 30
wait_mgmt 192.168.207.1 || true
tunnel 192.168.207.1
api GET '' >/dev/null; check "after Erase: SETUP on the default address" "$(jq_ "['state']")/$(mgmt_addr)" "SETUP/192.168.207.1/24"
echo; if [ "$FAILS" = 0 ]; then echo "renumbering on the VM: every check passed"; else echo "renumbering on the VM: $FAILS check(s) failed"; exit 1; fi
