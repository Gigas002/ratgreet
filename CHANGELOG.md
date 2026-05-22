# Changelog

## Unreleased

### Changed

- CI: wau-style workflows renamed for the `tuigreet` package (`build`, `test`, `fmt-clippy`, `doc`, `typos`, `deny`, `deploy`).
- Version output uses `CARGO_PKG_VERSION` (removed `build.rs` git script).
- UI strings are English-only (`src/ui/strings.rs`); removed Fluent/i18n embedding and `contrib/locales/`.
- `nix` replaced with `rustix` for `uname(2)`; `lazy_static` replaced with `std::sync::LazyLock` in `info.rs`.

### Removed

- `build.rs`, `i18n.toml`, `examples/toc/` (wau leftovers).
