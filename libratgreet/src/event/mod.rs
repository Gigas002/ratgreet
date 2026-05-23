use std::time::Duration;

#[cfg(all(not(test), not(feature = "test-harness")))]
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
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

        tokio::task::spawn({
            let tx = tx.clone();

            async move {
                #[cfg(any(test, feature = "test-harness"))]
                {
                    let mut render_interval = tokio::time::interval(frame_duration);
                    loop {
                        render_interval.tick().await;
                        let _ = tx.send(Event::Render).await;
                    }
                }

                #[cfg(all(not(test), not(feature = "test-harness")))]
                loop {
                    // `ate_esc` is reset each outer iteration so that Char events typed
                    // *after* the current batch are never suppressed.
                    let mut ate_esc = false;

                    if crossterm::event::poll(frame_duration).unwrap_or(false) {
                        while crossterm::event::poll(Duration::ZERO).unwrap_or(false) {
                            match crossterm::event::read() {
                                Ok(crossterm::event::Event::Key(key)) => {
                                    match key.code {
                                        KeyCode::Esc => {
                                            // Mark that an ESC arrived in this batch.
                                            // Char events that follow immediately in the
                                            // same batch are discarded: on a Linux VT the
                                            // keyboard driver delivers F-key sequences
                                            // byte-by-byte, so crossterm may return a lone
                                            // Esc followed by the remaining bytes as
                                            // individual Char events.
                                            ate_esc = true;
                                            let _ = tx.send(Event::Key(key)).await;
                                        }
                                        KeyCode::Char(_) if ate_esc => {
                                            // Discard – almost certainly an escape-sequence
                                            // fragment, not deliberate user input.
                                        }
                                        _ => {
                                            ate_esc = false;
                                            let _ = tx.send(Event::Key(key)).await;
                                        }
                                    }
                                }
                                // Terminal resize: ratatui auto-detects size changes on
                                // each draw(), so just let the pending Render handle it.
                                Ok(crossterm::event::Event::Resize(..)) => {}
                                _ => {}
                            }
                        }
                    }
                    let _ = tx.send(Event::Render).await;
                }
            }
        });

        Events { rx, tx }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }

    pub fn sender(&self) -> Sender<Event> {
        self.tx.clone()
    }
}
