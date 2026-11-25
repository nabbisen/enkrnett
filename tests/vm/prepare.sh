#!/bin/bash
# Prepares the x86-64 measurement VM for the daemon (what the package's post-install and init
# script do on a device): the user, the ubusd ACL, ownership of the four configuration files,
# the sticky directory, the state directory, and a clean base configuration without the
# measurement leftovers. Then reboots the VM so that ubusd reads the ACL.
#
#   tests/vm/prepare.sh VM_DIR     VM_DIR holds vm.sh, vmssh, vmscp (the M-6 virtual machine)
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
VM=${1:?VM_DIR}
cd "$VM"
./vm.sh start >/dev/null || true
python3 vmsh.py wait 120 >/dev/null 2>&1 || true
./vmscp "$ROOT/openwrt/package/enkrnett/files/enkrnett.acl.json" /tmp/enkrnett.acl.json >/dev/null
./vmssh 'set -e
grep -q "^enkrnett:" /etc/passwd || echo "enkrnett:x:1900:1900:enkrnett daemon:/var/run/enkrnett:/bin/false" >> /etc/passwd
grep -q "^enkrnett:" /etc/group || echo "enkrnett:x:1900:" >> /etc/group
grep -q "^enkrnett:" /etc/shadow || echo "enkrnett:*:0:0:99999:7:::" >> /etc/shadow
cp /tmp/enkrnett.acl.json /usr/share/acl.d/enkrnett.json
# A base configuration as an Enkrnett image would ship it: no measurement sections, no stock
# dnsmasq instance (the daemon owns every instance), lan without an address, wan by DHCP.
uci -q batch <<EOF
delete network.guest
delete network.mgmt
delete network.br_guest
delete network.br_mgmt
set network.lan.proto=none
delete network.lan.ipaddr
delete network.lan.netmask
delete network.lan.ip6assign
EOF
uci -q commit network
for s in $(uci -q show wireless | sed -n "s/^wireless\.\(enk_[a-z0-9_]*\)=.*/\1/p; s/^wireless\.\(default_radio[0-9]*\)=.*/\1/p" | sort -u); do uci -q delete "wireless.$s"; done
uci -q commit wireless
cat > /etc/config/dhcp <<EOF
config odhcpd "odhcpd"
	option maindhcp "0"
	option leasefile "/tmp/hosts/odhcpd"
	option leasetrigger "/usr/sbin/odhcpd-update"
	option loglevel "4"
EOF
cat > /etc/config/firewall <<EOF
config defaults
	option input "REJECT"
	option output "ACCEPT"
	option forward "REJECT"
	option synflood_protect "1"

config zone "wan"
	option name "wan"
	list network "wan"
	list network "wan6"
	option input "REJECT"
	option output "ACCEPT"
	option forward "REJECT"
	option masq "1"
	option mtu_fix "1"

config rule "measurement_ssh"
	option name "measurement only: ssh from the uplink"
	option src "wan"
	option proto "tcp"
	option dest_port "22"
	option target "ACCEPT"
EOF
chown enkrnett:enkrnett /etc/config/wireless /etc/config/network /etc/config/dhcp /etc/config/firewall
chmod 0600 /etc/config/wireless /etc/config/network /etc/config/dhcp /etc/config/firewall
chgrp enkrnett /etc/config && chmod 1775 /etc/config
rm -rf /etc/enkrnett; mkdir -p /etc/enkrnett && chown enkrnett:enkrnett /etc/enkrnett && chmod 0750 /etc/enkrnett
mkdir -p /usr/share/enkrnett/profiles /usr/share/enkrnett/ui
echo prepared; (sleep 1; reboot) >/dev/null 2>&1 &'
echo "VM prepared; rebooting"
