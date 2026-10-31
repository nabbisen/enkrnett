#!/bin/sh
# The claim (chapter 4) against `enkrnettd --mock`, with curl. Start the daemon first:
#
#   cargo run -q -p enkrnettd -- --mock --listen 127.0.0.1:8080 --state-dir /tmp/enkrnett-demo --act-delay-ms 0
#
# then run this script. It prints every request and the device's answer.
set -eu
BASE="${BASE:-http://127.0.0.1:8080/api/1}"
PASSWORD="${PASSWORD:-correct horse battery staple}"

call() {
  method=$1; path=$2; shift 2
  echo
  echo "### $method $path $*"
  if [ "$method" = GET ]; then
    curl -sS "$@" "$BASE/$path"
  else
    curl -sS -X POST -H 'Content-Type: application/json' "$@" "$BASE/$path"
  fi
  echo
}

call GET ""                                   # the greeting: SETUP, connection.security
SESSION=$(curl -sS -X POST -H 'Content-Type: application/json' -d '{}' "$BASE/claim/start" | sed -n 's/.*"token":"\([^"]*\)".*/\1/p')
echo "claim session: $SESSION"
call GET claim -H "Enkrnett-Session: $SESSION"
call POST claim/identify -H "Enkrnett-Session: $SESSION" -d '{}'
call POST claim/check -H "Enkrnett-Session: $SESSION" \
  -d "{\"country\":\"JP\",\"guest\":{\"name\":\"Cafe Example\",\"mode\":\"open_to_all\"},\"owner_password\":\"$PASSWORD\"}"
call POST claim/confirm -H "Enkrnett-Session: $SESSION" -d '{}'
sleep 1
call GET ""                                   # PENDING
OWNER=$(curl -sS -X POST -H 'Content-Type: application/json' -d "{\"password\":\"$PASSWORD\"}" "$BASE/login" | sed -n 's/.*"token":"\([^"]*\)".*/\1/p')
echo "owner session: $OWNER"
call GET status -H "Enkrnett-Session: $OWNER"
call GET management-key -H "Enkrnett-Session: $OWNER"
call POST settings -H "Enkrnett-Session: $OWNER" \
  -d '{"version":1,"guest":{"name":"Cafe Example","mode":"encrypted_only","enabled":true},"management":{"name":"Enkrnett-3F2A"}}'
call GET device -H "Enkrnett-Session: $OWNER"
echo
echo "To erase: curl -X POST -H 'Content-Type: application/json' -H \"Enkrnett-Session: $OWNER\" -d '{\"password\":\"...\"}' $BASE/erase"
