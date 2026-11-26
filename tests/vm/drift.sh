#!/bin/bash
# Drift on the VM (ENK-CFG-014; chapter 6): an expert edits an owned section by hand with `uci`,
# which also hands the file back to root. The daemon must report the drift, not overwrite it, and
# restore on request; the restore meets a file that is not its own, exits, is respawned by procd
# with ownership restored by the init script, and completes the restore at start-up (ENK-CFG-009).
#
#   tests/vm/drift.sh VM_DIR     (after tests/vm/claim.sh: a CLAIMED device)
set -euo pipefail
VM=${1:?VM_DIR}
PASSWORD="correct horse battery staple"
PORT=18085
cd "$VM"
FAILS=0
pass() { echo "  ok   $*"; }
fail() { echo "  FAIL $*"; FAILS=$((FAILS + 1)); }
check() { if [ "$2" = "$3" ]; then pass "$1"; else fail "$1 (got '$2', expected '$3')"; fi; }
api() { local method=$1 path=$2 token=${3:-} body=${4:-}
  local args=(-s -o /tmp/enkrnett-d-body -w '%{http_code}' --connect-timeout 5 -m 40 --retry 8 --retry-connrefused --retry-all-errors --retry-delay 2 -X "$method" "http://127.0.0.1:$PORT/api/1/$path" -H "Host: 192.168.207.1")
  [ -n "$token" ] && args+=(-H "Enkrnett-Session: $token")
  [ "$method" = POST ] && args+=(-H 'Content-Type: application/json' -d "${body:-{\}}")
  curl "${args[@]}" 2>/dev/null || true; }
jq_() { python3 -c "import json,sys; d=json.load(open('/tmp/enkrnett-d-body')); print(eval('d'+sys.argv[1]))" "$1" 2>/dev/null || echo "?"; }

./vm.sh start >/dev/null || true
python3 vmsh.py wait 120 >/dev/null 2>&1 || true
ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o LogLevel=ERROR -i id_ed25519 -p 2222 -N -L "$PORT:192.168.207.1:8080" root@127.0.0.1 &
TUNNEL=$!; trap 'kill $TUNNEL 2>/dev/null || true' EXIT; sleep 2

echo "## a CLAIMED device without drift"
api GET '' >/dev/null; check "state CLAIMED" "$(jq_ "['state']")" CLAIMED
S=$(api POST login '' "{\"password\":\"$PASSWORD\"}"); check "login" "$S" 200; T=$(jq_ "['token']")
api GET status "$T" >/dev/null; check "no drift reported" "$(jq_ "['drift']")" False
GUEST=$(jq_ "['settings']['guest']['name']")

echo "## an expert edits the guest name by hand with uci (the file becomes root's)"
./vmssh "uci set wireless.enk_guest.ssid='Hand Edit'; uci commit wireless; ls -la /etc/config/wireless | awk '{print \$1, \$3}'" | sed 's/^/  /'
sleep 31
# The daemon cannot even read a file that uci made root's (0600): it exits, is respawned with
# ownership restored, and reports the drift it then finds. Sessions went with the restart.
S=$(api GET status "$T"); if [ "$S" != 200 ]; then sleep 10; S=$(api POST login '' "{\"password\":\"$PASSWORD\"}"); check "login again after the respawn" "$S" 200; T=$(jq_ "['token']"); api GET status "$T" >/dev/null; fi
check "ENK-CFG-014 drift reported within a minute" "$(jq_ "['drift']")" True
./vmssh "uci -q get wireless.enk_guest.ssid" > /tmp/enkrnett-d-ssid; check "the hand edit is still there (not overwritten by itself)" "$(cat /tmp/enkrnett-d-ssid)" "Hand Edit"
./vmssh "logread | grep -c 'drift_detected'" > /tmp/enkrnett-d-n; [ "$(cat /tmp/enkrnett-d-n)" -ge 1 ] && pass "drift_detected logged (ENK-API-029)" || fail "no drift_detected in the log"

echo "## restore my settings: an ordinary change now that the files are the daemon's again"
S=$(api POST settings/restore "$T" '{}'); check "restore accepted" "$S" 202
for i in $(seq 1 60); do api GET change "$T" >/dev/null; st=$(jq_ "['state']"); case "$st" in settled|undone) break ;; esac; sleep 2; done
check "the restore settled" "$(jq_ "['state']")" settled
check "kind restore" "$(jq_ "['kind']")" restore
./vmssh "uci -q get wireless.enk_guest.ssid; ls -la /etc/config/wireless | awk '{print \$1, \$3}'" > /tmp/enkrnett-d-after; sed 's/^/  /' /tmp/enkrnett-d-after
./vmssh "logread | grep -c files_not_ours" > /tmp/enkrnett-d-n; [ "$(cat /tmp/enkrnett-d-n)" -ge 1 ] && pass "the daemon left the file it could not read to the respawn (files_not_ours)" || fail "no files_not_ours"
grep -qx "$GUEST" /tmp/enkrnett-d-after && pass "the guest name is back: $GUEST" || fail "guest name not restored"
grep -q 'enkrnett$' /tmp/enkrnett-d-after && pass "the file is the daemon's again" || fail "file owner"
api GET status "$T" >/dev/null; check "no drift after the restore" "$(jq_ "['drift']")" False
api POST logout "$T" >/dev/null
echo; if [ "$FAILS" = 0 ]; then echo "drift on the VM: every check passed"; else echo "drift on the VM: $FAILS check(s) failed"; exit 1; fi
