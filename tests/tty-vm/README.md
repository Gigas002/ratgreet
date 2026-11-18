# ratgreet on a real console

A ratgreet in a terminal emulator against `greetd-stub` is not the ratgreet
greetd starts on a VT: there the keys come from the Linux console's keymap,
the screen is 80×25 (or the framebuffer's size), the termios belong to the VT,
and PAM, the greeter user and session start are real. These scripts boot the
host's kernel in QEMU into an initramfs with the real `greetd`, PAM
(`pam_unix`), and the ratgreet under test on tty1, then drive it through the
QEMU monitor. The desktop's greetd and `/etc` are never touched.

Needs: `qemu-system-x86_64` with KVM, `cpio`, `openssl`, ImageMagick
(`magick`) for screenshots, and greetd installed on the host (its binary is
copied in).

```sh
cargo build --release -p ratgreet
tests/tty-vm/build.sh target/release/ratgreet            # [config.toml], THEME=theme.toml
tests/tty-vm/vmctl.py start                              # [kernel args, e.g. vga=0x318]
tests/tty-vm/vmctl.py keys alice "<ret>"                 # text, or <f2> <esc> <ctrl-u> <caps_lock> ...
tests/tty-vm/vmctl.py keys secret "<ret>"
tests/tty-vm/vmctl.py shot login                         # target/tty-vm/login.png
tests/tty-vm/vmctl.py sh "cat /tmp/session.log /var/log/ratgreet/ratgreet.log"
tests/tty-vm/vmctl.py stop
```

In the VM: user `alice`, password `secret`; PAM first sends an info message;
two sessions (Hyprland, Sway) run `/usr/local/bin/fake-session`, which records
the user, `XDG_*` and the tty in `/tmp/session.log`; with a config whose
`[power]` commands call `/usr/local/bin/fake-power`, power actions land in
`/tmp/power.log`. ratgreet runs with `--debug`, logging to `/var/log/ratgreet/ratgreet.log`
(`RATGREET_CMD` overrides the command line). `sh` runs a command as root over
a second serial port; `log` prints the console log (greetd's output).

`KERNEL` picks the kernel (default: the first `/boot/vmlinuz-*`) and
`TTY_VM_DIR` the work directory (default `target/tty-vm`). QEMU runs with
`-cpu host`, since the copied binaries may need the host's CPU level.
