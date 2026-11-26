#!/bin/bash
# The checks derived from RFC 003 and RFC 004 (chapters 3, 4, 9) against the daemon that procd
# started from the installed package on the VM: request security, authorization by state, the
# claim, sessions, the login delay, and Erase through a real boot (the package path).
#
#   tests/vm/contract.sh VM_DIR
set -euo pipefail
VM=${1:?VM_DIR}
PASSWORD="correct horse battery staple"
PORT=18083
cd "$VM"
FAILS=0
pass() { echo "  ok   $*"; }
fail() { echo "  FAIL $*"; FAILS=$((FAILS + 1)); }
check() { # description actual expected
  if [ "$2" = "$3" ]; then pass "$1"; else fail "$1 (got '$2', expected '$3')"; fi
}
api() { # method path [token] [body] -> prints status; body in /tmp/enkrnett-c-body
  local method=$1 path=$2 token=${3:-} body=${4:-}
  local args=(-s -o /tmp/enkrnett-c-body -w '%{http_code}' --connect-timeout 5 -m 40 --retry 6 --retry-connrefused --retry-all-errors --retry-delay 2 -X "$method" "http://127.0.0.1:$PORT/api/1/$path" -H "Host: 192.168.207.1")
  [ -n "$token" ] && args+=(-H "Enkrnett-Session: $token")
  [ "$method" = POST ] && args+=(-H 'Content-Type: application/json' -d "${body:-{\}}")
  curl "${args[@]}" 2>/dev/null || true
}
# For the login-delay checks: no --retry, because curl retries a 429 after Retry-After by itself.
api1() {
  local method=$1 path=$2 token=${3:-} body=${4:-}
  local args=(-s -o /tmp/enkrnett-c-body -D /tmp/enkrnett-c-headers -w '%{http_code}' --connect-timeout 5 -m 40 -X "$method" "http://127.0.0.1:$PORT/api/1/$path" -H "Host: 192.168.207.1")
  [ -n "$token" ] && args+=(-H "Enkrnett-Session: $token")
  [ "$method" = POST ] && args+=(-H 'Content-Type: application/json' -d "${body:-{\}}")
  curl "${args[@]}" 2>/dev/null || true
}
raw() { curl -s -D /tmp/enkrnett-c-headers -o /tmp/enkrnett-c-body -w '%{http_code}' --connect-timeout 5 -m 20 "$@" 2>/dev/null || true; }
jq_() { python3 -c "import json,sys; d=json.load(open('/tmp/enkrnett-c-body')); print(eval('d'+sys.argv[1]))" "$1" 2>/dev/null || echo "?"; }
code() { jq_ "['error']['code']"; }

./vm.sh start >/dev/null || true
python3 vmsh.py wait 120 >/dev/null 2>&1 || true
echo "## procd service (ENK-CFG-019; port 80 redirected to 8080 on the management interface)"
./vmssh 'sleep 5; ubus call service list "{\"name\":\"enkrnett\"}" | grep -E "running|pid" | tr -d "\t" | tr "\n" " "; echo; P=$(pidof enkrnettd); echo "uid: $(grep ^Uid /proc/$P/status | cut -f2) (1900 expected)"; grep -E "^CapEff" /proc/$P/status; netstat -tln 2>/dev/null | grep ":8080 " || echo "no listener on :8080"; nft list chain inet fw4 dstnat_enk_mgmt 2>/dev/null | grep -E "redirect|dport 80" | head -2; logread | grep -c enkrnett; tail -3 /etc/config/enkrnett' || true

ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o LogLevel=ERROR -i id_ed25519 -p 2222 -N -L "$PORT:192.168.207.1:8080" root@127.0.0.1 &
TUNNEL=$!
trap 'kill $TUNNEL 2>/dev/null || true' EXIT
sleep 2

api GET '' >/dev/null
if [ "$(jq_ "['state']")" = CLAIMED ]; then
  echo "## device is CLAIMED from an earlier run: erasing first"
  api1 POST login '' "{\"password\":\"$PASSWORD\"}" >/dev/null; O=$(jq_ "['token']")
  api POST erase "$O" "{\"password\":\"$PASSWORD\"}" >/dev/null
  kill $TUNNEL 2>/dev/null || true
  sleep 20; python3 vmsh.py wait 180 >/dev/null 2>&1 || true; sleep 25
  ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o LogLevel=ERROR -i id_ed25519 -p 2222 -N -L "$PORT:192.168.207.1:8080" root@127.0.0.1 &
  TUNNEL=$!
  sleep 3
fi

echo "## RFC 003: request security (ENK-SEC-009 to 012; ENK-API-016, 025)"
S=$(raw -H 'Host: evil.example' "http://127.0.0.1:$PORT/api/1/"); check "foreign Host refused with 421" "$S" 421
S=$(raw -H 'Host: 192.168.207.1' "http://127.0.0.1:$PORT/api/1/"); check "greeting 200" "$S" 200
for h in 'Content-Security-Policy' 'X-Content-Type-Options: nosniff' 'X-Frame-Options: DENY' 'Referrer-Policy: no-referrer' 'Cache-Control: no-store' 'Connection: close'; do
  grep -qi "$h" /tmp/enkrnett-c-headers && pass "header $h" || fail "header $h missing"
done
grep -qi 'Access-Control-Allow' /tmp/enkrnett-c-headers && fail "CORS header present" || pass "no CORS header (ENK-SEC-011)"
S=$(raw -X OPTIONS -H 'Host: 192.168.207.1' -H 'Origin: http://evil.example' "http://127.0.0.1:$PORT/api/1/login"); check "OPTIONS refused 405" "$S" 405
S=$(raw -H 'Host: 192.168.207.1' "http://127.0.0.1:$PORT/"); check "entry page 200" "$S" 200
grep -qi 'Cache-Control: no-store' /tmp/enkrnett-c-headers && pass "entry page no-store (ENK-API-024)" || fail "entry page cache"
S=$(raw -X POST -H 'Host: 192.168.207.1' -H 'Content-Type: text/plain' -d '{}' "http://127.0.0.1:$PORT/api/1/claim/start"); check "wrong content type 415" "$S" 415
S=$(api GET nope); check "unknown resource 404" "$S" 404

echo "## RFC 004: states and the claim (ENK-OWN-001..008, ENK-AUTH-007)"
api GET '' >/dev/null; check "state is SETUP after a fresh install" "$(jq_ "['state']")" SETUP
check "connection.network setup" "$(jq_ "['connection']['network']")" setup
S=$(api GET status); check "status without session in SETUP: wrong_state 403" "$S/$(code)" "403/wrong_state"
S=$(api POST login '' '{"password":"x"}'); check "login not offered in SETUP" "$S/$(code)" "403/wrong_state"
S=$(api GET claim); check "claim without claim session: no_session 401" "$S/$(code)" "401/no_session"
S=$(api POST claim/start); check "claim/start 200" "$S" 200; T=$(jq_ "['token']")
S=$(api POST claim/start); check "second client refused (ENK-OWN-005)" "$S/$(code)" "409/setup_in_progress_elsewhere"
S=$(api POST claim/check "$T" '{"guest":{"name":"Cafe Example","mode":"open_to_all"},"owner_password":"password123"}'); check "common password refused (ENK-OWN-010)" "$S/$(jq_ "['error']['fields'][0]['code']")" "400/common_password"
S=$(api POST claim/check "$T" '{"a":1,"a":2}'); check "duplicate JSON key refused (ENK-SEC-020)" "$S/$(code)" "400/invalid"
S=$(api POST claim/confirm "$T"); check "confirm before check: no_proposal (ENK-OWN-007)" "$S/$(code)" "409/no_proposal"
S=$(api POST claim/check "$T" "{\"guest\":{\"name\":\"Cafe Example\",\"mode\":\"open_to_all\"},\"owner_password\":\"$PASSWORD\"}"); check "check 200" "$S" 200
KEY=$(jq_ "['management']['key']"); NAME=$(jq_ "['management']['name']")
python3 -c "import re,sys; sys.exit(0 if re.fullmatch(r'[a-z2-9]{5}-[a-z2-9]{5}-[a-z2-9]{5}-[a-z2-9]{5}', '$KEY') else 1)" && pass "management key form (ENK-OWN-015): $KEY" || fail "key form $KEY"
S=$(api POST claim/confirm "$T"); check "confirm 200 -> PENDING" "$S/$(jq_ "['state']")" "200/PENDING"
sleep 12
api GET '' >/dev/null; check "greeting PENDING" "$(jq_ "['state']")" PENDING
S=$(api GET claim "$T"); check "claim resources gone in PENDING" "$S/$(code)" "403/wrong_state"
./vmssh 'ubus call hostapd.enk-mgmt get_status | grep -q ENABLED && echo "  ok   management network on the air" || echo "  FAIL management network"; grep -c enk_setup /etc/config/wireless | sed "s/^/  setup sections left: /"'

echo "## RFC 004: login, sessions, the delay (ENK-OWN-012, ENK-AUTH-001..005)"
for i in 1 2 3 4 5; do S=$(api1 POST login '' '{"password":"wrong wrong wrong"}'); done
check "fifth wrong password still 401" "$S/$(code)" "401/wrong_password"
S=$(api1 POST login '' "{\"password\":\"$PASSWORD\"}"); check "sixth attempt refused: try_later 429" "$S/$(code)" "429/try_later"
RETRY=$(jq_ "['error']['retry_in']")
# A second may pass between the fifth failure and this attempt: 29 or 30, never 0 or above 30.
[ "$RETRY" -ge 25 ] 2>/dev/null && [ "$RETRY" -le 30 ] && pass "retry_in $RETRY (30 s delay, chapter 4 §6)" || fail "retry_in '$RETRY'"
grep -qi "^Retry-After: $RETRY" /tmp/enkrnett-c-headers && pass "Retry-After header equals retry_in" || fail "Retry-After header"
echo "  waiting ${RETRY}s for the delay to end"; sleep $((RETRY + 1))
S=$(api1 POST login '' "{\"password\":\"$PASSWORD\"}"); check "login after the delay -> CLAIMED" "$S/$(jq_ "['state']")" "200/CLAIMED"; O1=$(jq_ "['token']")
TOKENS=("$O1")
for i in 2 3 4 5; do S=$(api POST login '' "{\"password\":\"$PASSWORD\"}"); TOKENS+=("$(jq_ "['token']")"); done
S=$(api GET status "${TOKENS[0]}"); check "the fifth session ended the first (ENK-AUTH-005)" "$S/$(code)" "401/no_session"
S=$(api GET status "${TOKENS[4]}"); check "the newest session works" "$S" 200
check "status: guest up" "$(jq_ "['guest']['up']")" True
check "status: management name" "$(jq_ "['management']['name']")" "$NAME"
S=$(api GET management-key "${TOKENS[4]}"); check "management key matches the proposal (ENK-OWN-016)" "$(jq_ "['key']")" "$KEY"
S=$(api GET device "${TOKENS[4]}"); check "installed_as package" "$(jq_ "['installed_as']")" package
S=$(api POST logout "${TOKENS[4]}"); check "logout 200" "$S" 200
S=$(api GET status "${TOKENS[4]}"); check "ended session refused" "$S/$(code)" "401/no_session"

echo "## RFC 004 §8: Erase through a real boot (ENK-OWN-017, 018)"
S=$(api POST login '' "{\"password\":\"$PASSWORD\"}"); O=$(jq_ "['token']")
S=$(api POST erase "$O" '{"password":"wrong wrong wrong"}'); check "erase with the wrong password 401" "$S/$(code)" "401/wrong_password"
S=$(api POST erase "$O" "{\"password\":\"$PASSWORD\"}"); check "erase accepted 202" "$S" 202
kill $TUNNEL 2>/dev/null || true
echo "  waiting for the device to go down and come back"
sleep 20; python3 vmsh.py wait 180 >/dev/null 2>&1 || true; sleep 15
./vmssh 'M=$(ls /etc/enkrnett/ERASE 2>/dev/null | wc -l); [ "$M" = 0 ] && echo "  ok   the erase marker was consumed by boot()" || echo "  FAIL erase marker still present"
O=$(grep -c "\"owner\": {" /etc/enkrnett/state.json 2>/dev/null || true); [ "$O" = 0 ] && echo "  ok   no owner record on flash after the erase (ENK-OWN-017)" || echo "  FAIL owner record survived the erase"
logread | grep -q "enkrnett: erase requested" && echo "  ok   the init script logged the erase" || echo "  FAIL no erase line in the log"
sleep 20; ubus list | grep -q "hostapd\.phy" && echo "  ok   the setup network is back on the air" || echo "  FAIL no BSS after the boot"' | tee /tmp/enkrnett-c-vm
FAILS=$((FAILS + $(grep -c '  FAIL' /tmp/enkrnett-c-vm || true)))
ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o LogLevel=ERROR -i id_ed25519 -p 2222 -N -L "$PORT:192.168.207.1:8080" root@127.0.0.1 &
TUNNEL=$!
sleep 3
api GET '' >/dev/null; check "after the erase and the boot: SETUP again (ENK-OWN-018, 001)" "$(jq_ "['state']")" SETUP
S=$(api POST login '' "{\"password\":\"$PASSWORD\"}"); check "the old password opens nothing" "$S/$(code)" "403/wrong_state"

echo
if [ "$FAILS" = 0 ]; then echo "contract on the real platform: every check passed"; else echo "contract on the real platform: $FAILS check(s) failed"; exit 1; fi
