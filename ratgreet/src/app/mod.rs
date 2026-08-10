use std::{error::Error, io, sync::Arc};

use libratgreet::{
    event::{Event, Events},
    greeter::{AuthStatus, Greeter},
    ipc::Ipc,
    keyboard,
    power::PowerPostAction,
};
use ratatui::crossterm::{
    execute,
    terminal::{LeaveAlternateScreen, disable_raw_mode},
};
use ratatui::{Terminal, backend::Backend};
use tokio::sync::RwLock;

#[cfg(all(not(test), not(feature = "test-harness")))]
use ratatui::crossterm::terminal::{EnterAlternateScreen, enable_raw_mode};

use crate::ui::common::style::Theme;

mod greeter_init;

pub use greeter_init::init_greeter;

/// Puts the terminal into raw mode and switches to the alternate screen.
///
/// This must run *before* anything else touches stdin — in particular before
/// [`libratgreet::event::Events::new`] spawns its keyboard reader, and before any other
/// startup work (settings/session loading, connecting to greetd) that could give the user
/// time to type. While the terminal is still in cooked/line-buffered mode, keystrokes are
/// echoed straight to the (not-yet-alternate) screen and only delivered a full line at a
/// time; once raw mode kicks in mid-typing, buffered input can be replayed out of order or
/// misinterpreted by the reader. Real greetd/VT sessions are especially prone to this,
/// since there is often stale input already queued on the tty (from boot messages, a
/// previous session, or an impatient user) by the time ratgreet starts, whereas a manual
/// `cargo run` in a desktop terminal rarely has any pending input at startup.
#[cfg(all(not(test), not(feature = "test-harness")))]
pub fn prepare_terminal() -> io::Result<()> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;

    // Discard whatever was already buffered on the tty before we started reading for
    // real, so it cannot be misinterpreted as keystrokes once the main loop starts.
    while ratatui::crossterm::event::poll(std::time::Duration::ZERO).unwrap_or(false) {
        let _ = ratatui::crossterm::event::read();
    }

    Ok(())
}

#[cfg(any(test, feature = "test-harness"))]
pub fn prepare_terminal() -> io::Result<()> {
    Ok(())
}

pub async fn run<B>(
    backend: B,
    greeter: Greeter,
    theme: Theme,
    mut events: Events,
) -> Result<(), Box<dyn Error>>
where
    B: Backend,
    <B as Backend>::Error: 'static,
{
    tracing::info!("ratgreet started");

    register_panic_handler();

    let mut terminal = Terminal::new(backend)?;

    #[cfg(all(not(test), not(feature = "test-harness")))]
    terminal.clear()?;

    let ipc = Ipc::new();

    let greeter = Arc::new(RwLock::new(greeter));

    tokio::task::spawn({
        let greeter = greeter.clone();
        let mut ipc = ipc.clone();

        async move {
            loop {
                let _ = ipc.handle(greeter.clone()).await;
            }
        }
    });

    loop {
        if let Some(status) = greeter.read().await.exit {
            tracing::info!("exiting main loop");

            return Err(status.into());
        }

        match events.next().await {
            Some(Event::Render) => crate::ui::draw(greeter.clone(), &theme, &mut terminal).await?,
            Some(Event::Key(key)) => keyboard::handle(greeter.clone(), key, ipc.clone()).await?,

            Some(Event::Exit(status)) => {
                exit(&mut *greeter.write().await, status).await;
            }

            Some(Event::PowerCommand(command)) => {
                if let PowerPostAction::ClearScreen =
                    libratgreet::power::run(&greeter, command).await
                {
                    execute!(io::stdout(), LeaveAlternateScreen)?;
                    terminal.set_cursor_position((1, 1))?;
                    terminal.clear()?;
                    disable_raw_mode()?;

                    break;
                }
            }

            _ => {}
        }
    }

    Ok(())
}

pub async fn exit(greeter: &mut Greeter, status: AuthStatus) {
    tracing::info!("preparing exit with status {}", status);

    match status {
        AuthStatus::Success => {}
        AuthStatus::Cancel | AuthStatus::Failure => Ipc::cancel(greeter).await,
    }

    #[cfg(all(not(test), not(feature = "test-harness")))]
    clear_screen();

    let _ = execute!(io::stdout(), LeaveAlternateScreen);
    let _ = disable_raw_mode();

    greeter.exit = Some(status);
}

fn register_panic_handler() {
    let hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |info| {
        #[cfg(all(not(test), not(feature = "test-harness")))]
        clear_screen();

        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        let _ = disable_raw_mode();

        hook(info);
    }));
}

#[cfg(all(not(test), not(feature = "test-harness")))]
fn clear_screen() {
    use ratatui::backend::CrosstermBackend;

    let backend = CrosstermBackend::new(io::stdout());

    if let Ok(mut terminal) = Terminal::new(backend) {
        let _ = terminal.hide_cursor();
        let _ = terminal.clear();
    }
}
