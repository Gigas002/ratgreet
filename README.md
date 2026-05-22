# tuigreet

Terminal greeter for [greetd](https://git.sr.ht/~kennylevinsen/greetd). Built with Rust and [ratatui](https://ratatui.rs/).

## Overview

tuigreet connects to greetd over a Unix socket, draws a TUI login prompt, and starts the configured session. Behavior is defined in **`config.toml`**; colors in **`theme.toml`**. The binary only accepts **`--config`**, **`--theme`**, and **`--debug`** (plus `--help` / `--version`).

Invalid or missing config/theme files are skipped; the greeter falls back to built-in defaults and keeps running (warnings appear when tracing is enabled).

**Reference docs (commented examples):**

| File                                           | Contents                                   |
| ---------------------------------------------- | ------------------------------------------ |
| [`examples/config.toml`](examples/config.toml) | Sessions, UI, secrets, keybindings, power  |
| [`examples/theme.toml`](examples/theme.toml)   | Color roles, ANSI names, hex syntax        |
| [`examples/cli.md`](examples/cli.md)           | CLI flags, file resolution, greetd snippet |

## Development

Workspace layout: **`libtuigreet/`** (greetd core), **`tuigreet/`** (config, UI, binary), **`tests/`** (greetd-stub integration). Details in [`docs/PLAN.md`](docs/PLAN.md).

```bash
cargo test --workspace
```

### Run locally (normal terminal)

The binary needs **`GREETD_SOCK`**. Use [greetd-stub](https://github.com/apognu/greetd-stub) — **do not** build with `test-harness` (that mode is for automated tests only).

**Terminal 1:**

```bash
cargo install greetd-stub   # once
greetd-stub -s /tmp/greetd.sock --user alice:secret
```

**Terminal 2:**

```bash
GREETD_SOCK=/tmp/greetd.sock cargo run -p tuigreet -- \
  --config examples/config.toml \
  --theme examples/theme.toml
```

Debug builds run `true` after login; release builds need `[session] cmd` in config. See [`examples/cli.md`](examples/cli.md).

The `test-harness` Cargo feature is enabled only by the `tuigreet-tests` crate for in-memory integration tests — not for packagers or manual runs.

## Migrating from older tuigreet

Long CLI flags moved to TOML. See [`CHANGELOG.md`](CHANGELOG.md). Removed: user picker (`--user-menu`), remember/cache (`--remember*`).

## License

GPL-3.0-or-later. See `LICENSE`.
