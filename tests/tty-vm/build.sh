#!/bin/bash
# Build an initramfs that boots the host kernel into the real greetd, PAM and
# ratgreet on the VGA console's tty1 (see README.md).
# Usage: build.sh RATGREET_BINARY [config.toml]   (THEME=theme.toml optional)
set -euo pipefail
V=${TTY_VM_DIR:-$(git -C "$(dirname "$0")" rev-parse --show-toplevel)/target/tty-vm}
mkdir -p "$V"
BIN=$1; CONF=${2:-}
R=$V/root; rm -rf $R; mkdir -p $R/{usr/bin,usr/lib/security,etc/pam.d,etc/greetd,etc/ratgreet,proc,sys,dev,run,tmp,root,home/alice,var/lib/greeter,usr/share/wayland-sessions,usr/local/bin}
ln -s usr/bin $R/bin; ln -s usr/bin $R/sbin; ln -s usr/lib $R/lib; ln -s usr/lib $R/lib64
chmod 1777 $R/tmp
copy_bin() { cp -L "$1" $R/usr/bin/$(basename "${2:-$1}"); }
for b in bash greetd unix_chkpwd tty openvt mkdir rm mount sleep cat ls id stty setsid env date tail chvt fgconsole ps pgrep pkill true head grep tr sed; do copy_bin $(type -P $b); done
ln -s bash $R/usr/bin/sh
cp -L "$BIN" $R/usr/bin/ratgreet
for m in pam_unix pam_permit pam_deny pam_env pam_echo; do cp -L /usr/lib/security/$m.so $R/usr/lib/security/; done
# shared libraries of everything copied
for f in $R/usr/bin/* $R/usr/lib/security/*.so; do ldd "$f" 2>/dev/null | command grep -oE '/[^ ]+' || true; done | sort -u | while read -r l; do cp -L "$l" $R/usr/lib/ 2>/dev/null || true; done
cp -L /usr/lib/libnss_files.so.2 $R/usr/lib/ 2>/dev/null || true
HASH=$(openssl passwd -6 -salt vmsalt secret)
cat > $R/etc/passwd <<P
root:x:0:0:root:/root:/bin/bash
greeter:x:964:964:greeter:/var/lib/greeter:/bin/sh
alice:x:1000:1000:Alice:/home/alice:/bin/bash
P
cat > $R/etc/shadow <<P
root:!:19000::::::
greeter:!*:19000::::::
alice:$HASH:19000:0:99999:7:::
P
chmod 600 $R/etc/shadow
printf 'root:x:0:\ntty:x:5:\ngreeter:x:964:\nalice:x:1000:\n' > $R/etc/group
printf 'passwd: files\ngroup: files\nshadow: files\n' > $R/etc/nsswitch.conf
chown -R 1000:1000 $R/home/alice 2>/dev/null || true
printf 'auth optional pam_echo.so Welcome, %%u: this is a PAM info message\nauth required pam_unix.so\naccount required pam_unix.so\nsession required pam_unix.so\npassword required pam_unix.so\n' > $R/etc/pam.d/greetd
printf 'auth required pam_permit.so\naccount required pam_permit.so\nsession required pam_permit.so\n' > $R/etc/pam.d/greetd-greeter
cat > $R/etc/greetd/config.toml <<P
[terminal]
vt = 1
[default_session]
command = "${RATGREET_CMD:-ratgreet --debug /tmp/ratgreet.log}"
user = "greeter"
P
[ -n "${THEME:-}" ] && cp "$THEME" $R/etc/ratgreet/theme.toml
[ -n "$CONF" ] && cp "$CONF" $R/etc/ratgreet/config.toml
cp /etc/issue $R/etc/issue 2>/dev/null || true; cp -L /etc/os-release $R/etc/os-release
for s in Hyprland Sway; do printf '[Desktop Entry]\nName=%s\nExec=/usr/local/bin/fake-session %s\nType=Application\nDesktopNames=%s\n' $s $s $s > $R/usr/share/wayland-sessions/${s,,}.desktop; done
cat > $R/usr/local/bin/fake-session <<'P'
#!/bin/bash
echo "SESSION_START name=$1 user=$(id -un) XDG_SESSION_TYPE=$XDG_SESSION_TYPE XDG_CURRENT_DESKTOP=$XDG_CURRENT_DESKTOP tty=$(tty)" >> /tmp/session.log
echo "fake session $1 for $(id -un)"; sleep ${SESSION_SECS:-3}
P
cat > $R/usr/local/bin/fake-power <<'P'
#!/bin/bash
echo "POWER $*" >> /tmp/power.log
P
chmod +x $R/usr/local/bin/*
cat > $R/init <<'P'
#!/bin/bash
mount -t proc proc /proc; mount -t sysfs sys /sys; mount -t devtmpfs dev /dev
mkdir -p /dev/pts; mount -t devpts devpts /dev/pts
mount -t tmpfs -o mode=755 tmpfs /run; mount -t tmpfs -o mode=1777 tmpfs /tmp
echo ratgreet-vm > /proc/sys/kernel/hostname
export PATH=/usr/local/bin:/usr/bin
stty -F /dev/ttyS1 raw -echo
setsid bash -c 'exec bash < /dev/ttyS1 > /dev/ttyS1 2>&1' &
echo INIT_OK > /dev/ttyS0
while true; do
  greetd --config /etc/greetd/config.toml >> /dev/ttyS0 2>&1
  echo "GREETD_EXIT $?" > /dev/ttyS0; sleep 1
done
P
chmod +x $R/init
(cd $R && find . | cpio -o -H newc -R 0:0 --quiet) > $V/initrd.cpio
echo "built $(du -h $V/initrd.cpio | cut -f1)"
