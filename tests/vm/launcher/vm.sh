#!/bin/sh
# The x86-64 OpenWrt test VM under QEMU (tests/vm/image.sh builds the disk; chapter 14 §2).
# Usage: [VM_WAN_NET=10.207.0.0/24] ./vm.sh start | stop | status
#   eth0 = LAN (slirp 192.168.100.0/24, unused), eth1 = WAN (slirp, Internet,
#   host 127.0.0.1:2222 -> guest 22, host 127.0.0.1:8080 -> guest 8080).
#   VM_WAN_NET: the uplink subnet, for the ENK-TOPO collision tests (default 10.0.2.0/24).
# KVM is used when /dev/kvm is writable (GitHub-hosted runners after the udev rule in the
# workflow); otherwise QEMU falls back to software emulation, which is slower but works.
# The SMBIOS product string is pinned: OpenWrt derives the board name from the DMI vendor and
# product, and the daemon picks its profile by that name (qemu-standard-pc-i440fx-piix-1996 in
# openwrt/profiles/x86_64_generic). Distribution builds of QEMU change the default product string
# (Ubuntu's says "Ubuntu 26.04 PC ..."), which left the daemon without a profile on the runner.
cd "$(dirname "$0")" || exit 1
case "$1" in
start)
  [ -f qemu.pid ] && kill -0 "$(cat qemu.pid)" 2>/dev/null && { echo "already running"; exit 0; }
  rm -f serial.sock
  if [ -w /dev/kvm ]; then ACCEL="-enable-kvm -cpu host"; else ACCEL="-accel tcg -cpu max"; echo "no /dev/kvm: software emulation"; fi
  # shellcheck disable=SC2086
  qemu-system-x86_64 $ACCEL -m 256 -smp 2 \
    -machine pc -smbios 'type=1,manufacturer=QEMU,product=Standard PC (i440FX + PIIX,, 1996)' \
    -drive file=disk.img,format=raw,if=virtio \
    -nic user,model=virtio-net-pci,net=192.168.100.0/24 \
    -nic user,model=virtio-net-pci,net="${VM_WAN_NET:-10.0.2.0/24}",hostfwd=tcp:127.0.0.1:2222-:22,hostfwd=tcp:127.0.0.1:8080-:8080 \
    -display none -serial unix:serial.sock,server,nowait -monitor none \
    -daemonize -pidfile qemu.pid
  echo "started pid $(cat qemu.pid)"
  ;;
stop)
  [ -f qemu.pid ] && kill "$(cat qemu.pid)" 2>/dev/null; rm -f qemu.pid serial.sock; echo stopped ;;
status)
  [ -f qemu.pid ] && kill -0 "$(cat qemu.pid)" 2>/dev/null && echo running || echo "not running" ;;
*) echo "usage: $0 start|stop|status"; exit 2 ;;
esac
