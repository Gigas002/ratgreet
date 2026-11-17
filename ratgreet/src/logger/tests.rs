use tracing::Level;

use super::targets;
use crate::config::LogLevel;

#[test]
fn library_events_are_logged() {
    let info = targets(LogLevel::Info);
    assert!(info.would_enable("libratgreet::ipc", &Level::INFO));
    assert!(info.would_enable("ratgreet::app", &Level::INFO));
}

#[test]
fn the_configured_level_applies() {
    let warn = targets(LogLevel::Warn);
    assert!(!warn.would_enable("libratgreet::ipc", &Level::INFO));
    assert!(warn.would_enable("libratgreet::ipc", &Level::WARN));
    assert!(targets(LogLevel::Trace).would_enable("ratgreet::ui", &Level::TRACE));
}

#[test]
fn nothing_is_written_without_debug() {
    let dir = tempfile::tempdir().unwrap();
    let mut greeter = libratgreet::Greeter::default();
    greeter.logfile = dir.path().join("ratgreet.log").display().to_string();
    assert!(super::init(&greeter, LogLevel::Info).is_none());
    assert!(!dir.path().join("ratgreet.log").exists());
}
