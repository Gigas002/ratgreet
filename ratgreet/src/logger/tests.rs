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

/// The only test that installs the global subscriber (it can be set once).
#[test]
fn debug_writes_both_crates_events_to_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let log = dir.path().join("ratgreet.log");
    let mut greeter = libratgreet::Greeter::default();
    greeter.debug = true;
    greeter.logfile = log.display().to_string();

    let guard = super::init(&greeter, LogLevel::Info).expect("a log file");
    tracing::info!(target: "libratgreet::ipc", "from the library");
    tracing::info!("from the greeter");
    tracing::debug!("below the level");
    drop(guard);

    use std::os::unix::fs::PermissionsExt;
    let mode = std::fs::metadata(&log).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o600);
    let text = std::fs::read_to_string(&log).unwrap();
    assert!(text.contains("from the library"), "{text}");
    assert!(text.contains("from the greeter"), "{text}");
    assert!(!text.contains("below the level"), "{text}");
}

/// A symlink at the log path (another user's, in /tmp) is not followed.
#[test]
fn a_symlinked_log_file_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("victim");
    std::fs::write(&target, "").unwrap();
    let link = dir.path().join("ratgreet.log");
    std::os::unix::fs::symlink(&target, &link).unwrap();
    let mut greeter = libratgreet::Greeter::default();
    greeter.debug = true;
    greeter.logfile = link.display().to_string();
    assert!(super::init(&greeter, LogLevel::Info).is_none());
}
