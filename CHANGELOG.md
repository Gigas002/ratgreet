# Changelog

## Unreleased

### Changed

- Dates and times are formatted with `jiff` instead of `chrono`, which is unmaintained (RUSTSEC-2026-0543). `[ui].time_format` still takes strftime formats; a format jiff can't print is rejected when the theme loads.
- `libratgreet`: `Greeter::connect` and `Greeter::reset` return `io::Result<()>` instead of exiting the process; `init_greeter` returns `Result<Greeter, String>`.

### Fixed

- Drawing no longer overflows when the theme's `width` or paddings don't fit the terminal: debug builds panicked, release builds drew the "Please wait..." screen off screen when `width` was wider than the console (80 columns on a VGA tty) and crashed in the session and power menus for `width` below 4.
- A theme color with a non-ASCII character after `#` (`#aé123`) crashed ratgreet at start-up, leaving no login prompt; it is now rejected like any invalid color.
- `--debug` logs libratgreet's events too (greetd requests and responses, sessions, power commands); only ratgreet's own few lines were written. `[logging].level` is now applied; it was ignored. Without `--debug` the log file is no longer created.
- Problems with `config.toml` and `theme.toml` (a skipped invalid file, an unusable merged result) were logged before logging was set up, so never shown; they are now written to the `--debug` log, along with keys ratgreet doesn't know.
- `cargo build --release` at the workspace root produced a greeter with the `test-harness` feature (unified in from the integration tests): it never connected to greetd nor read the keyboard, so nobody could log in. The tests crate is no longer a default member, and a binary built with `test-harness` refuses to start with an explanation.
- When ratgreet can't reach greetd (`GREETD_SOCK` unset, nothing listening) it restores the terminal and says so (`cannot connect to greetd at …`), instead of exiting with a bare `No such file or directory` and leaving a terminal emulator raw and in the alternate screen. Panics before the main loop restore the terminal too.
- The cursor position no longer carries over from one field to the next or moves past the text: typed before a redraw, a password after moving the cursor in the username was entered scrambled (`secret` as `cretse`). Down in an empty session menu no longer underflows.

### Security

- Typing and deleting edit the username, password and command in place, and cleared fields are zeroized, so fewer copies of a password are left in freed memory.

## 0.1.0 - 2026-09-04

### Changed

- **Rename tuigreet → ratgreet**: binary and crate names (`ratgreet`, `libratgreet`, `ratgreet-tests`), default paths (`/etc/ratgreet/`, `~/.config/ratgreet/`, `/tmp/ratgreet.log`), and CI/release artifacts.
- Workspace layout (wau-style): `libratgreet` (greetd core + unit tests), `ratgreet` (CLI/config/theme/settings/logger/UI + binary), `ratgreet-tests` (greetd-stub integration). Root `src/` removed.
- Crate split: TOML parsing, settings, app loop, logger, and ratatui UI live in **`ratgreet`**; greetd IPC, greeter state, and keyboard live in **`libratgreet`**.
- CLI is minimal (`clap`): `--help`, `--version`, `--config`, `--theme`, `--debug` only. All former greeter flags live in `config.toml` / `theme.toml` (see migration table below).

- CI: wau-style workflows renamed for the `ratgreet` package (`build`, `test`, `fmt-clippy`, `doc`, `typos`, `deny`, `deploy`).
- Version output uses `CARGO_PKG_VERSION` (removed `build.rs` git script).
- UI strings are English-only (`src/ui/strings.rs`); removed Fluent/i18n embedding.
- `nix` replaced with `rustix` 1.x for `uname(2)`; `lazy_static` replaced with `std::sync::LazyLock` in `info.rs`.
- Rust edition **2024** (workspace); dependency bumps: **ratatui** 0.30, **crossterm** 0.29, **toml** 1.1, **thiserror** 2, **unicode-width** 0.2.
- Theme colors: CSS-like hex (`#rgb`, `#rrggbb`, `#rrggbbaa`) plus named ANSI colors; **ansi-to-tui** and **rand** removed from the CLI crate.
- `[secrets]`: `display` (`hidden` / `plain` / `masked`, default **masked**) replaces `mask`; `mask_char` is a single character when masked.
- **libratgreet**: dropped `smart-default`, `futures`, `uzers`, `utmp-rs`; user menu reads `/etc/passwd` via **rustix**; event loop uses crossterm `poll`/`read`; `/etc/issue` `\U` is no longer live-counted.
- CI workflows use a single default build/test/clippy matrix (no optional feature flags).
- Removed bundled README screenshots (`docs/images/`).

### Removed

- User picker (`--user-menu` / `[user_menu]`): login is manual username + password only.
- Remember / cache (`--remember*`, `/var/cache/ratgreet/`): no saved username, session, or autologin on startup.
- `build.rs`, `i18n.toml`, `examples/toc/` (wau leftovers).
- `contrib/` directory (locales, fixtures, man page, screenshots, helper scripts).
- `nsswrapper` Cargo feature and NSS-wrapper-based tests.
- `getopts` and the legacy long-option CLI surface (`Greeter::options()`, `parse_options()`).

### CLI → config migration

| Former CLI                                                                                      | New location                                                      |
| ----------------------------------------------------------------------------------------------- | ----------------------------------------------------------------- |
| `--cmd`, `--env`                                                                                | `[session]`                                                       |
| `--sessions`, `--xsessions`, `--session-wrapper`, `--xsession-wrapper`, `--no-xsession-wrapper` | `[session]`                                                       |
| `--width`, `--window-padding`, `--container-padding`, `--prompt-padding`, `--greet-align`       | `theme.toml` `[ui]`                                               |
| `--issue`, `--greeting`, `--time`, `--time-format`                                              | `theme.toml` `[ui]`                                               |
| `--remember`, `--remember-session`, `--remember-user-session`                                   | removed (manual login every time)                                 |
| `--user-menu`, `--user-menu-min-uid`, `--user-menu-max-uid`                                     | removed (type username manually)                                  |
| `--asterisks`, `--asterisks-char`                                                               | `[secrets].display` (`plain` / `hidden` / `masked`) + `mask_char` |
| `--theme` (inline colors)                                                                       | `theme.toml` / `--theme PATH`                                     |
| `--power-shutdown`, `--power-reboot`, `--power-no-setsid`                                       | `[power]`                                                         |
| `--kb-command`, `--kb-sessions`, `--kb-power`                                                   | `[keybindings]`                                                   |
| `-d` / `--debug`                                                                                | `[logging]` + CLI `--debug [FILE]`                                |
