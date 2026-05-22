use std::path::Path;

use super::*;

const EXAMPLE_THEME: &str = include_str!("../../../examples/theme.toml");

#[test]
fn parse_example_theme() {
    let theme = parse(EXAMPLE_THEME).unwrap();
    assert_eq!(theme.colors.container.as_deref(), Some("blue"));
    assert_eq!(theme.colors.title.as_deref(), Some("cyan"));
    assert_eq!(theme.colors.button.as_deref(), Some("yellow"));
}

#[test]
fn example_theme_converts_to_ui_theme() {
    let theme = parse(EXAMPLE_THEME).unwrap();
    let ui = theme.to_ui_theme().unwrap();
    let style = ui.of(&[crate::ui::common::style::Themed::Container]);
    assert!(style.bg.is_some());
}

#[test]
fn accepts_css_hex_with_alpha() {
    let theme = parse(
        r##"
[colors]
container = "#ffffff00"
"##,
    )
    .unwrap();
    let ui = theme.to_ui_theme().unwrap();
    let style = ui.of(&[crate::ui::common::style::Themed::Container]);
    assert_eq!(style.bg, Some(ratatui::style::Color::Rgb(255, 255, 255)));
}

#[test]
fn rejects_invalid_color() {
    let err = parse(
        r#"
[colors]
container = "not-a-color"
"#,
    )
    .unwrap_err();
    assert!(matches!(err, ThemeError::InvalidColor { .. }));
}

#[test]
fn example_theme_file_on_disk() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples/theme.toml");
    let theme = load(&path).unwrap();
    theme.to_ui_theme().unwrap();
}

#[test]
fn load_layered_missing_override_uses_defaults() {
    let theme = load_layered(Some(Path::new("/nonexistent/tuigreet/theme.toml")));
    assert!(theme.colors.container.is_none());
}

#[test]
fn load_layered_invalid_override_uses_defaults() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("theme.toml");
    std::fs::write(&path, "[colors]\ncontainer = \"not-a-color\"\n").unwrap();

    let theme = load_layered(Some(&path));
    assert!(theme.colors.container.is_none());
}
