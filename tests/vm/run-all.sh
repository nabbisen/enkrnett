#!/bin/bash
# The whole VM chain (chapter 14 §2, VM integration): prepare, package, claim, then chapters 2, 6
# and 7 on the real platform, then the RFC 003/004 contract with Erase through a boot. Each
# script's output goes to LOG_DIR; the summary names what passed and what did not.
#
#   tests/vm/run-all.sh VM_DIR [LOG_DIR]     VM_DIR from tests/vm/image.sh (or the measurement VM)
#
# Needs: the static x86-64 daemon (see package.sh), ui/dist, the x86/64 SDK (fetched by
# package.sh), QEMU. About 25 minutes with KVM.
set -uo pipefail
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
VM=${1:?VM_DIR}
LOGS=${2:-$ROOT/openwrt/out/vm-logs}
mkdir -p "$LOGS"; LOGS=$(cd "$LOGS" && pwd)
VM=$(cd "$VM" && pwd)
FAILED=()
step() { # name script
  local name=$1 script=$2 rc
  echo "=== $name"
  "$ROOT/tests/vm/$script" "$VM" >"$LOGS/$name.log" 2>&1; rc=$?
  tail -n 3 "$LOGS/$name.log" | sed 's/^/    /'
  [ $rc -eq 0 ] || { echo "    FAILED (exit $rc; $LOGS/$name.log)"; FAILED+=("$name"); }
  return $rc
}
daemon_log() { ( cd "$VM" && python3 vmsh.py run 'logread | grep enkrnettd | tail -200' >"$LOGS/daemon-log.txt" 2>&1 ) || true; }
# The first three steps are the foundation; when one fails, the rest would only report the fallout.
foundation() { step "$@" || { echo "foundation step $1 failed; stopping the chain"; daemon_log; exit 1; }; }
# The uplink must be the default one: renumber.sh restarts the VM with other uplinks and ends on it.
( cd "$VM" && ./vm.sh stop >/dev/null 2>&1; sleep 1; ./vm.sh start >/dev/null && python3 vmsh.py wait 240 >/dev/null 2>&1 ) || { echo "the VM did not come up"; exit 1; }
foundation prepare prepare.sh
sleep 30   # prepare.sh reboots the VM; package.sh waits for ssh, but not for the shutdown to begin
foundation package package.sh
sleep 40
foundation claim claim.sh
# claim.sh leaves a CLAIMED device; isolation.sh and drift.sh work on it.
step isolation isolation.sh || true
step drift drift.sh || true
# renumber.sh restarts the VM twice with colliding uplinks and ends with Erase: SETUP again.
step renumber renumber.sh || true
# contract.sh starts from SETUP (it erases first otherwise) and ends with Erase through a boot.
step contract contract.sh || true
daemon_log
echo
if [ ${#FAILED[@]} -eq 0 ]; then echo "VM chain: every script passed (logs in $LOGS)"; else echo "VM chain: failed: ${FAILED[*]} (logs in $LOGS)"; exit 1; fi
