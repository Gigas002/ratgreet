#!/usr/bin/env python3
"""Drive the ratgreet test VM: vmctl.py start [append] | keys TEXT... | shot NAME | sh CMD | log | stop"""
import glob, os, socket, subprocess, sys, time
HERE = os.path.dirname(os.path.abspath(__file__))
V = os.environ.get("TTY_VM_DIR") or os.path.join(HERE, "..", "..", "target", "tty-vm")
# Unix socket paths are limited to 108 bytes: keep them short.
SOCKS = f"/tmp/rgvm-{os.getuid()}"
MON, CMD = f"{SOCKS}/mon.sock", f"{SOCKS}/cmd.sock"
KERNEL = os.environ.get("KERNEL") or sorted(glob.glob("/boot/vmlinuz-*"))[0]
LOG, PID = os.path.join(V, "serial.log"), os.path.join(V, "qemu.pid")

def mon(cmd):
    s = socket.socket(socket.AF_UNIX); s.connect(MON); s.settimeout(5)
    time.sleep(0.05); s.recv(65536)
    s.sendall((cmd + "\n").encode()); time.sleep(0.15)
    try: out = s.recv(65536).decode(errors="replace")
    except socket.timeout: out = ""
    s.close(); return out

KEYS = {" ": "spc", "\n": "ret", "\t": "tab", "-": "minus", "=": "equal", ".": "dot", ",": "comma",
        "/": "slash", ";": "semicolon", "'": "apostrophe", "[": "bracket_left", "]": "bracket_right",
        "\\": "backslash", "`": "grave_accent"}
SHIFTED = {"_": "minus", "+": "equal", ">": "dot", "<": "comma", "?": "slash", ":": "semicolon",
           "\"": "apostrophe", "!": "1", "@": "2", "#": "3", "$": "4", "%": "5", "^": "6", "&": "7", "*": "8", "(": "9", ")": "0"}

def keys(*args):
    """Each arg: '<name>' sends a qemu key combo (e.g. <f2>, <ctrl-u>, <esc>); other text is typed."""
    for a in args:
        if a.startswith("<") and a.endswith(">") and len(a) > 2:
            mon("sendkey " + a[1:-1]); time.sleep(0.12); continue
        for c in a:
            if c.isalpha() and c.isupper(): k = "shift-" + c.lower()
            elif c.isalnum(): k = c
            elif c in KEYS: k = KEYS[c]
            elif c in SHIFTED: k = "shift-" + SHIFTED[c]
            else: raise SystemExit(f"no key for {c!r}")
            mon("sendkey " + k); time.sleep(0.08)

def shot(name):
    ppm = os.path.join(V, name + ".ppm"); png = os.path.join(V, name + ".png")
    if os.path.exists(ppm): os.remove(ppm)
    mon("screendump " + ppm)
    for _ in range(50):
        if os.path.exists(ppm) and os.path.getsize(ppm) > 0: break
        time.sleep(0.1)
    time.sleep(0.2)
    subprocess.run(["magick", ppm, png], check=True); print(png)

def sh(cmd, timeout=10):
    s = socket.socket(socket.AF_UNIX); s.connect(CMD); s.settimeout(timeout)
    s.sendall((cmd + " 2>&1; echo __END__\n").encode()); buf = b""
    while b"__END__" not in buf: buf += s.recv(65536)
    s.close(); return buf.decode(errors="replace").split("__END__")[0]

def kill_previous():
    try: pid = int(open(PID).read())
    except (OSError, ValueError): return
    try:
        if b"qemu" not in open(f"/proc/{pid}/cmdline", "rb").read(): return
        os.kill(pid, 15)
        for i in range(100):
            if not os.path.exists(f"/proc/{pid}"): return
            if i == 50: os.kill(pid, 9)
            time.sleep(0.05)
    except OSError: pass

def start(append=""):
    kill_previous()
    os.makedirs(SOCKS, mode=0o700, exist_ok=True)
    for p in (MON, CMD, LOG):
        if os.path.exists(p): os.remove(p)
    subprocess.run(["qemu-system-x86_64", "-enable-kvm", "-cpu", "host", "-m", "1024", "-smp", "1",
        "-kernel", KERNEL, "-initrd", os.path.join(V, "initrd.cpio"),
        "-append", ("console=ttyS0 loglevel=3 panic=1 rdinit=/init " + append).strip(),
        "-display", "none", "-vga", "std", "-nic", "none", "-no-reboot",
        "-serial", "file:" + LOG,
        "-chardev", f"socket,id=cmd,path={CMD},server=on,wait=off", "-device", "isa-serial,chardev=cmd",
        "-monitor", f"unix:{MON},server,nowait", "-daemonize", "-pidfile", PID], check=True)
    for _ in range(300):
        if os.path.exists(LOG) and b"INIT_OK" in open(LOG, "rb").read(): print("booted"); return
        time.sleep(0.1)
    raise SystemExit("no INIT_OK; serial log:\n" + open(LOG, errors="replace").read()[-3000:])

def stop():
    try: mon("quit")
    except OSError: pass
    time.sleep(0.5); kill_previous()

if __name__ == "__main__":
    c, a = sys.argv[1], sys.argv[2:]
    if c == "start": start(" ".join(a))
    elif c == "keys": keys(*a)
    elif c == "shot": shot(a[0])
    elif c == "sh": print(sh(" ".join(a)), end="")
    elif c == "log": print(open(LOG, errors="replace").read()[-int(a[0]) if a else -4000:], end="")
    elif c == "stop": stop()
