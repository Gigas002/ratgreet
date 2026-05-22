# Changelog

## Unreleased

### Changed

- CI: wau-style workflows renamed for the `tuigreet` package (`build`, `test`, `fmt-clippy`, `doc`, `typos`, `deny`, `deploy`).
- Version output uses `CARGO_PKG_VERSION` (removed `build.rs` git script).
- UI strings are English-only (`src/ui/strings.rs`); removed Fluent/i18n embedding.
- `nix` replaced with `rustix` for `uname(2)`; `lazy_static` replaced with `std::sync::LazyLock` in `info.rs`.
- CI workflows use a single default build/test/clippy matrix (no optional feature flags).
- README screenshots live under `docs/images/`.

### Removed

- `build.rs`, `i18n.toml`, `examples/toc/` (wau leftovers).
- `contrib/` directory (locales, fixtures, man page, screenshots, helper scripts).
- `nsswrapper` Cargo feature and NSS-wrapper-based tests.
