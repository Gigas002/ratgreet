# Audit

A whole-application audit of ratgreet and libratgreet (2026-11-18) in five
parts: bugs (§2), performance (§3), security and robustness (§4), usability
(§5), and code quality, tests and project hygiene (§6). §7 lists what was left
open, and §8 is the checklist to re-run the audit.

Each finding says what was found, how serious it is, and what was done:
**fixed** (with the commit), **accepted** (with the reason), or **OK**
(checked, nothing to do). Every fix is its own commit, with a regression test
where one can be written.

Severity: **high** (nobody can log in, or a password leaks), **medium** (a
crash or wrong behavior in a reachable case, an ignored setting), **low**
(hardening, polish, or needs an unusual setup).

## 1. Method

Every source file was read in full. Findings were then reproduced in two
ways:

- **In a terminal**, with the integration tests' in-process greetd stub and
  with the release binary in a pty.
- **On a real console.** A greeter on a VT differs from one in a terminal
  emulator: keys come through the console keymap, the screen is 80×25 (or the
  framebuffer's size), termios belong to the VT, and PAM, the greeter user and
  the session start are real. `tests/tty-vm` (added by this audit) boots the
  host kernel in QEMU into an initramfs with the real greetd, `pam_unix` and
  the ratgreet under test on tty1, types through the QEMU monitor and takes
  screenshots.

The desktop's greetd, `/etc/greetd` and `/etc/ratgreet` were never touched.
Your `/etc/ratgreet/theme.toml` was copied into the VM, where it loads without
warnings.

On the console these were checked with the final build:

- login;
- a wrong password, then a retry;
- a PAM info message (`pam_echo`) before the password prompt;
- F2 to change the command to `bash`, then logging in to a shell;
- F3 to pick a session, which sets `XDG_SESSION_TYPE` and `XDG_CURRENT_DESKTOP`;
- F12 power, with and without `[power]` commands configured;
- Esc;
- CAPS LOCK;
- `/etc/issue`;
- an 80×25 VGA console and a 128×48 framebuffer console;
- a theme wider than the console;
- an unreachable greetd.

Builds checked: `-p ratgreet` (the AUR package and release workflow),
`--workspace`, `--no-default-features` and `--all-features`. Static checks:
`cargo clippy` at the three gate levels, `cargo deny check`, `typos` and
`cargo llvm-cov`.

## 2. Bugs

| Area | Finding | Status |
| ---- | ------- | ------ |
| Test harness in the binary | The integration tests enable `test-harness`, and Cargo unifies features across a build. So `cargo build --release` at the workspace root produced a ratgreet that never connected to greetd, set raw mode or read the keyboard. On tty1 it drew the login screen while the console echoed keystrokes underneath, and nobody could log in. The AUR package and the release workflow build `-p ratgreet` and were not affected. The tests crate's dependencies are dev-dependencies now, and a test-harness binary refuses to start. | Fixed, high (a4dfc3b, 5dce244) |
| Layout overflow | Layout arithmetic wasn't saturating, so a theme `width` or padding that doesn't fit the terminal caused problems. Debug builds panicked in 630 of the swept size combinations. Release builds crashed in the session and power menus for `width` below 4. They also drew "Please wait..." off screen when `width` exceeded the console: 80 columns on a VGA tty, where login then looked hung. | Fixed, medium (4e0a8e3) |
| Non-ASCII hex color | `#aé123` panicked while the theme loaded, so there was no login prompt. | Fixed, medium (83b4b24) |
| Scrambled password | `cursor_offset` was clamped only while drawing. It survived Enter, so after moving the cursor in the username, a password typed before the next redraw went in out of order (`secret` → `cretse`). Left and Right also moved it without bound. Checked on the console after the fix. | Fixed, medium (99f2bac) |
| Empty menu | Down in an empty session or power menu computed `len() - 1`. | Fixed, low (99f2bac) |
| Logging | `--debug` logged only ratgreet's few lines. libratgreet's events (greetd requests and responses, sessions, power) were filtered out, and `[logging].level` was ignored. Without `--debug`, an empty log file was created anyway. | Fixed, medium (2cf5a5c) |
| Silent config problems | A skipped invalid config or theme was reported before logging started, so the report was never seen. Unknown (misspelled) keys did nothing silently. Both now go to the `--debug` log. | Fixed, medium (e63d114) |
| Early exit | With `GREETD_SOCK` unset or greetd down, the library printed a bare `No such file or directory` and called `process::exit`. That happened after raw mode and the alternate screen were set, and before the panic hook was installed. In a terminal emulator the terminal stayed raw. The error is now returned, the terminal restored and the message names greetd's socket. On a VT the kernel resets termios anyway, so only the message was wrong there. | Fixed, low (6f3ec8d) |
| Power commands | Commands were split on single spaces, so `systemctl  poweroff` passed an empty argument and a leading space made the program name empty. A command that couldn't start was reported without its name. | Fixed, low (23dbe21, c41b359) |
| Login flow | Everything listed in §1 behaves on the console. | OK |

## 3. Performance

| Area | Finding | Status |
| ---- | ------- | ------ |
| CAPS LOCK | `kbdinfo gkbled capslock` was spawned before every draw, and an idle greeter draws twice a second. That is about 60 fork/execs a minute for as long as the login screen is up (62 PIDs in 30 s on the VM), and no indicator at all without `kbd`. It is now the same `KDGKBLED` ioctl on stdin, checked on the console with `kbdinfo` absent. | Fixed, medium (9097ec3) |
| Idle redraw | Two draws a second keep the clock current; each is a diff against the previous frame. | Accepted |

## 4. Security and robustness

| Area | Finding | Status |
| ---- | ------- | ------ |
| chrono | RUSTSEC-2026-0543 (unmaintained) failed `cargo deny`. It is replaced by `jiff`. `[ui].time_format` keeps strftime syntax, and a format jiff can't print is rejected at load instead of panicking while drawing. | Fixed, medium (30a2722) |
| Debug log | The log records every `CreateSession` username, which is sometimes a password typed one field early. It defaulted to `/tmp/ratgreet.log`, was created 0644, and was opened through any symlink another user planted at that path. New files are now 0600 and opened with `O_NOFOLLOW`. The default is now `/var/log/ratgreet/ratgreet.log`, in a directory that `dist/tmpfiles.conf` creates for `greeter` only; checked on the VM. | Fixed, low (99c58d8, a643792) |
| Password copies | Each keystroke built a new `String`, leaving the previous copy of the password in freed memory. Edits now happen in place, and cleared buffers are zeroized. | Fixed, low (99f2bac) |
| Passwords in the log | `PostAuthMessageResponse` is logged without its content. This had no test until this audit; checked on the VM too. | OK, test (31c7b00) |
| Password on screen | Masked with the configured character. | OK |
| Power commands | Run as argv (optionally under `setsid`), never through a shell. | OK |
| Dependencies | `cargo deny check` passes; manifests are at the newest majors and the lockfile is current. | OK (6ddadc2) |

## 5. Usability

| Area | Finding | Status |
| ---- | ------- | ------ |
| Error messages | greetd connection failures and power commands that fail to start now say which socket or command was involved. | Fixed (6f3ec8d, c41b359) |
| Build docs | The README says that only `cargo test` builds enable `test-harness`. | Fixed, docs (a4dfc3b, 5dce244) |
| Config mistakes | The README says to run with `--debug` to see skipped files and unknown keys. | Fixed, docs (e63d114) |

## 6. Code quality, tests and hygiene

| Area | Finding | Status |
| ---- | ------- | ------ |
| Host-dependent tests | `load_defaults_without_files` read the host's `/etc/ratgreet`, so it failed on any machine with a config there, yours included. | Fixed (f28ac61) |
| Flaky test | `display::show_time` compared the clock to the screen within the same second and failed about once in 30 runs; its own TODO noted the race. Removed. Time formats are still covered by the theme tests. | Fixed (efe0af9) |
| Cargo warnings | Three "unused dependency" warnings on every build of the tests crate. | Fixed (5dce244) |
| Console testing | There was no way to see ratgreet on a VT. `tests/tty-vm` (see its README) is one. | Fixed (ccdb577) |
| Coverage | 82.1 % → 81.6 % of lines (`cargo llvm-cov --workspace --all-features`). New tests cover the layout sweep, keyboard, power, logger, config warnings and `safe_debug` (0 % → 100 %). The new error paths in `main.rs` (0 %) run only as the real binary and were checked on the VM. | Accepted |
| Gates | fmt, typos, deny, clippy and tests at the three feature levels, and docs, pass on every commit. | OK |

## 7. Left open

- **Password growth.** In-place edits leave a copy behind only when the
  string grows past its capacity. Reserving, say, 128 bytes for the password
  buffer would close that too.
- **Multi-prompt PAM.** Checked with an info message followed by a password.
  Real MFA (TOTP after the password) runs through the same code path and is
  covered by the stub tests, but was not exercised on the console.
- **`main.rs` coverage.** It could be split into a testable `run()` that takes
  the terminal, at the cost of a less "slim" main (ARCHITECTURE §1.2).

## 8. Re-running the audit

```sh
# Gates (docs/ARCHITECTURE.md §8)
cargo fmt --all -- --check && typos && cargo deny check
for f in --no-default-features "" --all-features; do
  cargo clippy --workspace --all-targets $f -- -D warnings
  cargo test --workspace $f
done
cargo doc --workspace --no-deps

# Coverage (system LLVM tools on Arch: LLVM_COV / LLVM_PROFDATA)
LLVM_COV=/usr/bin/llvm-cov LLVM_PROFDATA=/usr/bin/llvm-profdata \
  cargo llvm-cov --workspace --all-features --summary-only

# On a real console (QEMU + KVM, greetd installed): see tests/tty-vm/README.md
cargo build --release -p ratgreet
THEME=/etc/ratgreet/theme.toml tests/tty-vm/build.sh target/release/ratgreet
tests/tty-vm/vmctl.py start
tests/tty-vm/vmctl.py keys alice "<ret>"; tests/tty-vm/vmctl.py keys secret "<ret>"
tests/tty-vm/vmctl.py sh "cat /tmp/session.log"; tests/tty-vm/vmctl.py stop
```
