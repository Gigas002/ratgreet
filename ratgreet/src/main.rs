use std::{io, process};

use clap::Parser;
use libratgreet::{event::Events, greeter::AuthStatus};
use ratatui::backend::CrosstermBackend;
use ratgreet::{app, cli::Cli, settings};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // `cargo build --workspace` unifies the `test-harness` feature (enabled by
    // the integration tests) into this binary: it would then never connect to
    // greetd nor read the keyboard, and nobody could log in.
    if cfg!(feature = "test-harness") {
        eprintln!(
            "this ratgreet was built with the test-harness feature and cannot greet; \
             build it with `cargo build --release -p ratgreet`"
        );
        process::exit(1);
    }

    let settings = match settings::Settings::load(&(&cli).into()) {
        Ok(settings) => settings,
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    };

    app::register_panic_handler();
    if let Err(err) = app::prepare_terminal() {
        app::restore_terminal();
        eprintln!("failed to initialize terminal: {err}");
        process::exit(1);
    }

    let theme = settings.theme.clone();
    let backend = CrosstermBackend::new(io::stdout());
    let events = Events::new().await;
    let greeter = match app::init_greeter(events.sender(), &settings).await {
        Ok(greeter) => greeter,
        Err(err) => {
            app::restore_terminal();
            eprintln!("ratgreet: {err}");
            process::exit(1);
        }
    };

    if let Err(error) = app::run(backend, greeter, theme, events).await {
        match error.downcast_ref::<AuthStatus>() {
            Some(AuthStatus::Success) => return,
            Some(_) => {}
            None => {
                app::restore_terminal();
                eprintln!("ratgreet: {error}");
            }
        }

        process::exit(1);
    }
}
