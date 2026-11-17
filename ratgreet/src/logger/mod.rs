use std::fs::OpenOptions;

use libratgreet::Greeter;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::filter::{LevelFilter, Targets};

use crate::config::LogLevel;

#[cfg(test)]
mod tests;

/// With `--debug`, log ratgreet's and libratgreet's events at `level` to the
/// log file; without it, log nothing and leave the file alone.
pub fn init(greeter: &Greeter, level: LogLevel) -> Option<WorkerGuard> {
    use tracing_subscriber::prelude::*;

    if !greeter.debug {
        return None;
    }
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&greeter.logfile)
        .ok()?;
    let (appender, guard) = tracing_appender::non_blocking(file);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(appender)
                .with_line_number(true),
        )
        .with(targets(level))
        .init();

    Some(guard)
}

/// Both crates log: the greeter state machine and greetd IPC live in
/// libratgreet, whose target `ratgreet` alone doesn't match.
fn targets(level: LogLevel) -> Targets {
    let level = match level {
        LogLevel::Trace => LevelFilter::TRACE,
        LogLevel::Debug => LevelFilter::DEBUG,
        LogLevel::Info => LevelFilter::INFO,
        LogLevel::Warn => LevelFilter::WARN,
        LogLevel::Error => LevelFilter::ERROR,
    };
    Targets::new()
        .with_target("ratgreet", level)
        .with_target("libratgreet", level)
}
