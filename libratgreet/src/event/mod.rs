use std::time::Duration;

use ratatui::crossterm::event::KeyEvent;
use tokio::{
    process::Command,
    sync::mpsc::{self, Sender},
};

use crate::AuthStatus;

const FRAME_RATE: f64 = 2.0;

pub enum Event {
    Key(KeyEvent),
    Render,
    PowerCommand(Command),
    Exit(AuthStatus),
}

pub struct Events {
    rx: mpsc::Receiver<Event>,
    tx: mpsc::Sender<Event>,
}

impl Events {
    pub async fn new() -> Events {
        let (tx, rx) = mpsc::channel(10);
        let frame_duration = Duration::from_secs_f64(1.0 / FRAME_RATE);

        #[cfg(any(test, feature = "test-harness"))]
        {
            let tx = tx.clone();

            tokio::task::spawn(async move {
                let mut render_interval = tokio::time::interval(frame_duration);
                loop {
                    render_interval.tick().await;
                    let _ = tx.send(Event::Render).await;
                }
            });
        }

        // `crossterm::event::poll`/`read` are blocking syscalls (they call `poll(2)`/
        // `read(2)` directly on the tty). Running them inside a plain `tokio::task::spawn`
        // async task occupies a runtime worker thread for the whole duration of the call.
        // On a machine with few CPU cores (e.g. the real console ratgreet actually runs
        // on via greetd, as opposed to a many-core desktop used for local testing), this
        // can starve every other task — rendering, the greetd IPC loop and keyboard
        // dispatch — making the whole UI look unresponsive or "stuck", even though input
        // is technically still being read correctly. `spawn_blocking` runs this on a
        // dedicated blocking-task thread pool instead, so it can never block the async
        // runtime.
        #[cfg(all(not(test), not(feature = "test-harness")))]
        {
            let tx = tx.clone();

            tokio::task::spawn_blocking(move || {
                loop {
                    if ratatui::crossterm::event::poll(frame_duration).unwrap_or(false) {
                        while ratatui::crossterm::event::poll(Duration::ZERO).unwrap_or(false) {
                            if let Ok(ratatui::crossterm::event::Event::Key(key)) =
                                ratatui::crossterm::event::read()
                            {
                                if tx.blocking_send(Event::Key(key)).is_err() {
                                    return;
                                }
                            }
                        }
                    }

                    if tx.blocking_send(Event::Render).is_err() {
                        return;
                    }
                }
            });
        }

        Events { rx, tx }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }

    pub fn sender(&self) -> Sender<Event> {
        self.tx.clone()
    }
}
