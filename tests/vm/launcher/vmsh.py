#!/usr/bin/env python3
"""Run shell commands in the test VM over its serial console (tests/vm/image.sh builds it).

Usage:
  vmsh.py wait [seconds]        wait until a shell prompt answers
  vmsh.py run 'cmd' ['cmd' ...] run commands, print their output
"""
import os, re, socket, sys, time

SOCK = os.path.join(os.path.dirname(os.path.abspath(__file__)), "serial.sock")
PROMPT = re.compile(rb"root@[\w-]+:[^\n]*# $")


def connect():
    s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    s.connect(SOCK)
    s.settimeout(0.5)
    return s


def read_until_prompt(s, timeout):
    buf = b""
    end = time.time() + timeout
    while time.time() < end:
        try:
            chunk = s.recv(65536)
            if not chunk:
                break
            buf += chunk
            if PROMPT.search(buf[-200:]):
                return buf, True
        except socket.timeout:
            if PROMPT.search(buf[-200:]):
                return buf, True
    return buf, False


def wait(seconds):
    end = time.time() + seconds
    while time.time() < end:
        try:
            s = connect()
        except OSError:
            time.sleep(1)
            continue
        s.sendall(b"\n")
        buf, ok = read_until_prompt(s, 3)
        s.close()
        if ok:
            print("prompt ready")
            return 0
        time.sleep(2)
    print("no prompt", file=sys.stderr)
    return 1


def run(cmds, timeout=120):
    s = connect()
    s.sendall(b"\n")
    read_until_prompt(s, 3)
    rc = 0
    for c in cmds:
        marker = b"__END_%d__" % int(time.time() * 1000)
        s.sendall(c.encode() + b"; echo " + marker + b"$?\n")
        buf = b""
        end = time.time() + timeout
        while time.time() < end:
            try:
                chunk = s.recv(65536)
                if chunk:
                    buf += chunk
            except socket.timeout:
                pass
            m = re.search(marker + rb"(\d+)", buf)
            if m and PROMPT.search(buf[-200:]):
                break
        text = buf.decode("utf-8", "replace")
        # drop the echoed command line and the marker line
        lines = text.split("\n")
        out = []
        for ln in lines[1:]:
            if marker.decode() in ln or PROMPT.search(ln.encode()):
                continue
            out.append(ln.rstrip("\r"))
        m = re.search(marker + rb"(\d+)", buf)
        code = int(m.group(1)) if m else -1
        print("$ " + c)
        print("\n".join(out).rstrip())
        print("[exit %d]" % code)
        rc = rc or code
    s.close()
    return 0 if rc == 0 else 1


if __name__ == "__main__":
    if len(sys.argv) >= 2 and sys.argv[1] == "wait":
        sys.exit(wait(int(sys.argv[2]) if len(sys.argv) > 2 else 120))
    if len(sys.argv) >= 3 and sys.argv[1] == "run":
        sys.exit(run(sys.argv[2:]))
    print(__doc__)
    sys.exit(2)
