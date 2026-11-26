#!/bin/bash
# Separation on the x86-64 VM with simulated stations (chapter 2; measurement M-8 made
# repeatable): ENK-ISO-001 to 006, ENK-NET-008, ENK-NET-010, ENK-CFG-023, ENK-CFG-024,
# ENK-CFG-025, ENK-TOPO-001, ENK-HARD-005, ENK-HARD-008; the client counts of chapter 7
# (ENK-STAT-*); the connection line of RFC 016 §10 (ENK-OWN-022).
#
# Needs a CLAIMED device from tests/vm/claim.sh and mac80211_hwsim with 5 radios on the VM
# (phy0 is the device). Four stations in network namespaces: g1 OWE, g2 open BSS, g3 OWE, m1
# WPA2 on the management network. Prints a report; exits non-zero when a row does not hold.
#
#   tests/vm/isolation.sh VM_DIR
set -euo pipefail
VM=${1:?VM_DIR}
PASSWORD="correct horse battery staple"
PORT=18084
cd "$VM"
FAILS=0
pass() { echo "  ok   $*"; }
fail() { echo "  FAIL $*"; FAILS=$((FAILS + 1)); }
check() { if [ "$2" = "$3" ]; then pass "$1"; else fail "$1 (got '$2', expected '$3')"; fi; }
api() { # method path [token] [body] -> status; body in /tmp/enkrnett-i-body
  local method=$1 path=$2 token=${3:-} body=${4:-}
  local args=(-s -o /tmp/enkrnett-i-body -w '%{http_code}' --connect-timeout 5 -m 40 --retry 6 --retry-connrefused --retry-all-errors --retry-delay 2 -X "$method" "http://127.0.0.1:$PORT/api/1/$path" -H "Host: 192.168.207.1")
  [ -n "$token" ] && args+=(-H "Enkrnett-Session: $token")
  [ "$method" = POST ] && args+=(-H 'Content-Type: application/json' -d "${body:-{\}}")
  curl "${args[@]}" 2>/dev/null || true
}
jq_() { python3 -c "import json,sys; d=json.load(open('/tmp/enkrnett-i-body')); print(eval('d'+sys.argv[1]))" "$1" 2>/dev/null || echo "?"; }

./vm.sh start >/dev/null || true
python3 vmsh.py wait 120 >/dev/null 2>&1 || true
ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o LogLevel=ERROR -i id_ed25519 -p 2222 -N -L "$PORT:192.168.207.1:8080" root@127.0.0.1 &
TUNNEL=$!
trap 'kill $TUNNEL 2>/dev/null || true' EXIT
sleep 2

echo "## the device: CLAIMED, guest network open to every device (Transition Mode)"
api GET '' >/dev/null; check "state CLAIMED" "$(jq_ "['state']")" CLAIMED
S=$(api POST login '' "{\"password\":\"$PASSWORD\"}"); check "login" "$S" 200; T=$(jq_ "['token']")
api GET status "$T" >/dev/null; V=$(jq_ "['settings']['version']"); GUEST=$(jq_ "['settings']['guest']['name']"); MGMT=$(jq_ "['settings']['management']['name']")
if [ "$(jq_ "['settings']['guest']['mode']")" != open_to_all ]; then
  S=$(api POST settings "$T" "{\"version\":$V,\"guest\":{\"name\":\"$GUEST\",\"mode\":\"open_to_all\",\"enabled\":true},\"management\":{\"name\":\"$MGMT\"}}"); check "switch to open_to_all accepted" "$S" 202
  for i in $(seq 1 90); do api GET change "$T" >/dev/null; case "$(jq_ "['state']")" in settled|undone) break ;; esac; sleep 1; done
  check "the change settled" "$(jq_ "['state']")" settled
fi
api GET management-key "$T" >/dev/null; KEY=$(jq_ "['key']")
echo "  guest \"$GUEST\" (open BSS) + \"${GUEST}OWE\" (hidden OWE BSS); management \"$MGMT\""

echo "## stations in network namespaces (hwsim phy1..phy4)"
TMP=$(mktemp -d); trap 'kill $TUNNEL 2>/dev/null || true; rm -rf "$TMP"' EXIT
cat > "$TMP/enk-stations.sh" <<EOS
#!/bin/sh
GUEST='$GUEST'; MGMT='$MGMT'; KEY='$KEY'
ns() { n=\$1; shift; ip netns exec \$n "\$@" 2>&1; }
phy_of() { for p in /sys/class/ieee80211/*; do case "\$(readlink -f "\$p/device")" in *"/\$1") basename "\$p";; esac; done; }
killall wpa_supplicant 2>/dev/null; killall udhcpc 2>/dev/null; for n in g1 g2 g3 m1; do ip netns del \$n 2>/dev/null; done
# The radios return to the root namespace a moment after their namespaces go.
for i in \$(seq 1 30); do [ "\$(ls /sys/class/ieee80211 | wc -l)" -ge 5 ] && break; sleep 1; done
echo "radios: \$(ls /sys/class/ieee80211 | tr '\n' ' ')"
start_sta() { # name phy ssid keymgmt [psk]
  n=\$1; phy=\$2; ssid=\$3; km=\$4; psk=\$5
  ip netns add \$n; iw phy \$phy set netns name \$n
  ns \$n iw phy \$phy interface add \${n}w type managed >/dev/null
  ns \$n ip link set \${n}w up
  { echo "ctrl_interface=/var/run/wpa_\$n"; echo "network={"; echo " ssid=\\"\$ssid\\""; echo " key_mgmt=\$km";
    [ "\$km" = OWE ] && echo " ieee80211w=2"; [ -n "\$psk" ] && echo " psk=\\"\$psk\\""; echo "}"; } > /tmp/\$n.conf
  ns \$n wpa_supplicant -B -D nl80211 -i \${n}w -c /tmp/\$n.conf -P /tmp/\$n.pid >/dev/null
  for i in \$(seq 1 25); do ns \$n wpa_cli -p /var/run/wpa_\$n -i \${n}w status 2>/dev/null | grep -q COMPLETED && break; sleep 1; done
  ns \$n udhcpc -i \${n}w -q -n -t 6 >/dev/null 2>&1 || true
  echo "\$n: \$(ns \$n wpa_cli -p /var/run/wpa_\$n -i \${n}w status | grep -E '^(bssid|key_mgmt|wpa_state)=' | tr '\\n' ' ') ip=\$(ns \$n ip -4 addr show \${n}w | awk '/inet /{print \$2}')"
}
start_sta g1 \$(phy_of hwsim1) "\$GUEST" OWE
start_sta g2 \$(phy_of hwsim2) "\$GUEST" NONE
start_sta g3 \$(phy_of hwsim3) "\$GUEST" OWE
start_sta m1 \$(phy_of hwsim4) "\$MGMT" WPA-PSK "\$KEY"
EOS
./vmscp "$TMP/enk-stations.sh" /root/enk-stations.sh >/dev/null
./vmssh 'sh /root/enk-stations.sh' | sed 's/^/  /'
cat > "$TMP/enk-rows.sh" <<'EOS'
#!/bin/sh
# One row per line: "<id>|<observed>"; the host compares.
ns() { n=$1; shift; ip netns exec $n "$@" 2>&1; }
ipof() { ns $1 ip -4 addr show ${1}w | awk '/inet /{print $2}' | cut -d/ -f1; }
tcp() { ns $1 nmap -Pn -n -p $3 --host-timeout 8s $2 2>/dev/null | awk -v p="$3" '$1 ~ "^"p"/" {print $2}'; }
pings() { ns $1 ping -c2 -W2 $2 2>/dev/null | grep -c 'bytes from' || true; }
G1=$(ipof g1); G2=$(ipof g2); G3=$(ipof g3); M1=$(ipof m1)
echo "addresses|g1=$G1 g2=$G2 g3=$G3 m1=$M1"
echo "ISO-001 guest->site host tcp|$(tcp g1 10.0.2.3 53)"
echo "ISO-001 guest->site router ping|$(pings g1 10.0.2.2)"
echo "TOPO-001 guest->Internet tcp|$(tcp g1 1.1.1.1 80)"
echo "ISO-003 guest scan of the device tcp|$(ns g1 nmap -Pn -n -p 1-1024,8080,8443 --host-timeout 150s 10.207.0.1 2>/dev/null | grep -E '^[0-9]+/' | awk '{print $1":"$2}' | tr '\n' ' ')"
echo "ISO-003 guest scan of the device udp|$(ns g1 nmap -Pn -n -sU -p 53,67 10.207.0.1 2>/dev/null | grep -E '^[0-9]+/' | awk '{print $1":"$2}' | tr '\n' ' ')"
echo "CFG-025 guest->management address 80|$(tcp g1 192.168.207.1 80)"
echo "CFG-025 guest->management address 8080|$(tcp g1 192.168.207.1 8080)"
echo "ISO-002 g1->g3 same OWE BSS ping|$(pings g1 $G3)"
echo "ISO-002 g1 (OWE)->g2 (open BSS) ping|$(pings g1 $G2)"
echo "ISO-002 g2 (open)->g1 (OWE) ping|$(pings g2 $G1)"
AP_MAC=$(cat /sys/class/net/br-guest/address)
ns g1 ip neigh replace $G3 lladdr $AP_MAC dev g1w nud permanent
echo "ISO-002 g1->g3 through the device MAC ping|$(pings g1 $G3)"
ns g1 ip neigh del $G3 dev g1w 2>/dev/null
G3LL=$(ns g3 ip -6 addr show g3w | awk '/inet6 fe80/{print $2}' | cut -d/ -f1)
echo "NET-008 device ipv6 on br-guest|$(ip -6 addr show br-guest 2>/dev/null | grep -c inet6 || true)"
echo "NET-008 g1->g3 ipv6 link-local ping|$(ns g1 ping6 -c2 -W2 ${G3LL}%g1w 2>/dev/null | grep -c 'bytes from' || true)"
echo "ISO-004 mgmt->UI 80|$(tcp m1 192.168.207.1 80)"
echo "ISO-004 mgmt->Internet|$(tcp m1 1.1.1.1 80)"
echo "ISO-004 mgmt->site host tcp|$(tcp m1 10.0.2.3 53)"
echo "ISO-004 mgmt->guest ping|$(pings m1 $G1)"
echo "ISO-004 mgmt->guest address 53|$(tcp m1 10.207.0.1 53)"
echo "OWN-022 greeting from the management station|$(ns m1 uclient-fetch -q -T 5 -O - http://192.168.207.1/api/1/ 2>/dev/null | tr -d '\n' | head -c 400)"
echo "OWN-022 greeting from a guest station|$(ns g1 uclient-fetch -q -T 5 -O - http://192.168.207.1/api/1/ 2>&1 | tr -d '\n' | head -c 60)"
echo "ISO-006 bridges|$(ls -d /sys/class/net/br-guest /sys/class/net/br-mgmt 2>/dev/null | wc -l)"
for br in br-guest br-mgmt; do for p in /sys/class/net/$br/brif/*; do echo "CFG-023 $br/$(basename $p) isolated|$(cat $p/isolated)"; done; done
echo "CFG-024 dns listeners|$(netstat -uln 2>/dev/null | awk '$4 ~ /:53$/ {print $4}' | sort | tr '\n' ' ')"
echo "HARD-005 listeners|$(netstat -tuln 2>/dev/null | awk 'NR>2 {print $1" "$4}' | sort -u | tr '\n' ';')"
echo "HARD-008 wps in hostapd|$(grep -c '^wps_state=[12]' /var/run/hostapd-phy0.conf 2>/dev/null || echo 0)"
echo "NET-010 management beacon hidden|$(grep -A30 "ssid=$1" /var/run/hostapd-phy0.conf 2>/dev/null | grep -m1 ignore_broadcast_ssid || echo none)"
EOS
./vmscp "$TMP/enk-rows.sh" /root/enk-rows.sh >/dev/null
./vmssh "sh /root/enk-rows.sh '$MGMT'" > /tmp/enkrnett-i-rows
row() { grep -F "$1|" /tmp/enkrnett-i-rows | head -1 | cut -d'|' -f2-; }
echo "  $(row addresses)"

echo "## chapter 2 §2: what may reach what"
check "ENK-ISO-001 guest cannot open the site host" "$(row 'ISO-001 guest->site host tcp')" closed
check "ENK-ISO-001 guest cannot ping the site router" "$(row 'ISO-001 guest->site router ping')" 0
check "ENK-TOPO-001 guest reaches the Internet" "$(row 'TOPO-001 guest->Internet tcp')" open
check "ENK-ISO-003 the device answers a guest on DNS only (tcp)" "$(row 'ISO-003 guest scan of the device tcp')" "53/tcp:open "
R=$(row 'ISO-003 guest scan of the device udp'); [[ "$R" == *"53/udp:open"* && "$R" == *"67/udp:open"* ]] && pass "ENK-ISO-003 udp: DNS and DHCP" || fail "ENK-ISO-003 udp: $R"
check "ENK-CFG-025 the UI is not reachable from a guest (80)" "$(row 'CFG-025 guest->management address 80')" closed
check "ENK-CFG-025 the UI is not reachable from a guest (daemon port)" "$(row 'CFG-025 guest->management address 8080')" closed
check "ENK-ISO-002 guest to guest, same BSS" "$(row 'ISO-002 g1->g3 same OWE BSS ping')" 0
check "ENK-ISO-002 guest to guest across the Transition Mode pair (M-8 row 6)" "$(row 'ISO-002 g1 (OWE)->g2 (open BSS) ping')" 0
check "ENK-ISO-002 and back" "$(row 'ISO-002 g2 (open)->g1 (OWE) ping')" 0
check "ENK-ISO-002 routed through the device" "$(row 'ISO-002 g1->g3 through the device MAC ping')" 0
check "ENK-NET-008 the device has no IPv6 address on the guest bridge" "$(row 'NET-008 device ipv6 on br-guest')" 0
check "ENK-NET-008 no IPv6 between guests" "$(row 'NET-008 g1->g3 ipv6 link-local ping')" 0
check "ENK-ISO-004 management client reaches the UI" "$(row 'ISO-004 mgmt->UI 80')" open
check "ENK-ISO-004 management client reaches the Internet" "$(row 'ISO-004 mgmt->Internet')" open
check "ENK-ISO-004 management client cannot open the site host" "$(row 'ISO-004 mgmt->site host tcp')" closed
check "ENK-ISO-004 management client cannot ping a guest" "$(row 'ISO-004 mgmt->guest ping')" 0
check "ENK-ISO-004 management client gets nothing from the guest address" "$(row 'ISO-004 mgmt->guest address 53')" closed
G=$(row 'OWN-022 greeting from the management station'); [[ "$G" == *'"network":"management"'* && "$G" == *'"security":"wpa2"'* ]] && pass "ENK-OWN-022 the greeting names the management network and WPA2 for a WPA2 station" || fail "ENK-OWN-022 greeting: $G"
G=$(row 'OWN-022 greeting from a guest station'); [[ "$G" != *'"state"'* ]] && pass "the API gives a guest station nothing" || fail "a guest station got the greeting"

echo "## chapter 6: the kernel's view the daemon reads back"
check "ENK-ISO-006 two bridges" "$(row 'ISO-006 bridges')" 2
N=0; while IFS= read -r l; do case "$l" in CFG-023*) N=$((N+1)); check "${l%%|*}" "${l#*|}" 1 ;; esac; done < /tmp/enkrnett-i-rows
[ "$N" -ge 3 ] && pass "ENK-CFG-023 $N bridge ports read" || fail "ENK-CFG-023 only $N bridge ports"
R=$(row 'CFG-024 dns listeners'); [[ "$R" == *"10.207.0.1:53"* && "$R" == *"192.168.207.1:53"* ]] && pass "ENK-CFG-024 one DNS listener per network: $R" || fail "ENK-CFG-024 DNS listeners: $R"
echo "  listening sockets (ENK-HARD-005; dropbear is the VM's): $(row 'HARD-005 listeners')"
check "ENK-HARD-008 no WPS in the generated hostapd configuration" "$(row 'HARD-008 wps in hostapd')" 0
check "ENK-NET-010 the management network is not hidden" "$(row 'NET-010 management beacon hidden')" none

echo "## chapter 7: the counts the status reports"
api GET status "$T" >/dev/null
check "guest clients encrypted (g1, g3 on the OWE BSS)" "$(jq_ "['guest']['clients_encrypted']")" 2
check "guest clients unencrypted (g2 on the open BSS)" "$(jq_ "['guest']['clients_unencrypted']")" 1
check "management clients (m1)" "$(jq_ "['management']['clients']")" 1
check "hardening report honest about the VM's dropbear" "$(jq_ "['hardening_met']")" False
api POST logout "$T" >/dev/null
./vmssh 'killall wpa_supplicant 2>/dev/null; killall udhcpc 2>/dev/null; for n in g1 g2 g3 m1; do ip netns del $n 2>/dev/null; done; true' >/dev/null 2>&1 || true

echo
if [ "$FAILS" = 0 ]; then echo "separation on the VM: every row holds"; else echo "separation on the VM: $FAILS row(s) do not hold"; exit 1; fi
