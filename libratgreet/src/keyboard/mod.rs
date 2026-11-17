use std::{error::Error, sync::Arc};

use greetd_ipc::Request;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::RwLock;
use zeroize::Zeroize;

use crate::{
    Greeter, Mode,
    ipc::Ipc,
    model::{
        masked::MaskedString,
        sessions::{Session, SessionSource},
    },
    power::power,
};

// Act on keyboard events.
//
// This function will be called whenever a keyboard event was captured by the
// application. It takes a reference to the `Greeter` so it can be aware of the
// current state of the application and act accordingly; It also receives the
// `Ipc` interface so it is able to interact with `greetd` if necessary.
pub async fn handle(
    greeter: Arc<RwLock<Greeter>>,
    input: KeyEvent,
    ipc: Ipc,
) -> Result<(), Box<dyn Error>> {
    let mut greeter = greeter.write().await;

    if greeter.working {
        return Ok(());
    }

    match input {
        // ^U should erase the current buffer.
        KeyEvent {
            code: KeyCode::Char('u'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            match greeter.mode {
                Mode::Username => greeter.username = MaskedString::default(),
                Mode::Password | Mode::Command => greeter.buffer.zeroize(),
                _ => {}
            }
            greeter.cursor_offset = 0;
        }

        // In debug mode only, ^X will exit the application.
        #[cfg(debug_assertions)]
        KeyEvent {
            code: KeyCode::Char('x'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            use crate::{AuthStatus, Event};

            if let Some(ref sender) = greeter.events {
                let _ = sender.send(Event::Exit(AuthStatus::Cancel)).await;
            }
        }

        // Depending on the active screen, pressing Escape will either return to the
        // previous mode (close a popup, for example), or cancel the `greetd`
        // session.
        KeyEvent {
            code: KeyCode::Esc, ..
        } => match greeter.mode {
            Mode::Command => {
                greeter.mode = greeter.previous_mode;
                greeter.buffer = greeter.previous_buffer.take().unwrap_or_default();
                greeter.cursor_offset = 0;
            }

            Mode::Sessions | Mode::Power => {
                greeter.mode = greeter.previous_mode;
            }

            _ => {
                Ipc::cancel(&mut greeter).await;
                greeter.reset(false).await?;
            }
        },

        // Simple cursor directions in text fields, kept within the text.
        KeyEvent {
            code: KeyCode::Left,
            ..
        } => {
            let len = field(&greeter).map_or(0, |value| value.chars().count() as i16);
            greeter.cursor_offset = (greeter.cursor_offset - 1).max(-len);
        }
        KeyEvent {
            code: KeyCode::Right,
            ..
        } => greeter.cursor_offset = (greeter.cursor_offset + 1).min(0),

        // F2 will display the command entry prompt. If we are already in one of the
        // popup screens, we set the previous screen as being the current previous
        // screen.
        KeyEvent {
            code: KeyCode::F(i),
            ..
        } if i == greeter.kb_command => {
            greeter.previous_mode = match greeter.mode {
                Mode::Command | Mode::Sessions | Mode::Power => greeter.previous_mode,
                _ => greeter.mode,
            };

            // Set the edition buffer to the current command.
            greeter.previous_buffer = Some(greeter.buffer.clone());
            greeter.buffer = greeter
                .session_source
                .command(&greeter)
                .map(str::to_string)
                .unwrap_or_default();
            greeter.cursor_offset = 0;
            greeter.mode = Mode::Command;
        }

        // F3 will display the session selection menu. If we are already in one of
        // the popup screens, we set the previous screen as being the current
        // previous screen.
        KeyEvent {
            code: KeyCode::F(i),
            ..
        } if i == greeter.kb_sessions => {
            greeter.previous_mode = match greeter.mode {
                Mode::Command | Mode::Sessions | Mode::Power => greeter.previous_mode,
                _ => greeter.mode,
            };

            greeter.mode = Mode::Sessions;
        }

        // F12 will display the user selection menu. If we are already in one of the
        // popup screens, we set the previous screen as being the current previous
        // screen.
        KeyEvent {
            code: KeyCode::F(i),
            ..
        } if i == greeter.kb_power => {
            greeter.previous_mode = match greeter.mode {
                Mode::Command | Mode::Sessions | Mode::Power => greeter.previous_mode,
                _ => greeter.mode,
            };

            greeter.mode = Mode::Power;
        }

        // Handle moving up in menus.
        KeyEvent {
            code: KeyCode::Up, ..
        } => {
            if let Mode::Sessions = greeter.mode
                && greeter.sessions.selected > 0
            {
                greeter.sessions.selected -= 1;
            }

            if let Mode::Power = greeter.mode
                && greeter.powers.selected > 0
            {
                greeter.powers.selected -= 1;
            }
        }

        // Handle moving down in menus.
        KeyEvent {
            code: KeyCode::Down,
            ..
        } => {
            if let Mode::Sessions = greeter.mode
                && greeter.sessions.selected + 1 < greeter.sessions.options.len()
            {
                greeter.sessions.selected += 1;
            }

            if let Mode::Power = greeter.mode
                && greeter.powers.selected + 1 < greeter.powers.options.len()
            {
                greeter.powers.selected += 1;
            }
        }

        // ^A should go to the start of the current prompt
        KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            let value = {
                match greeter.mode {
                    Mode::Username => &greeter.username.value,
                    _ => &greeter.buffer,
                }
            };

            greeter.cursor_offset = -(value.chars().count() as i16);
        }

        // ^A should go to the end of the current prompt
        KeyEvent {
            code: KeyCode::Char('e'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => greeter.cursor_offset = 0,

        // Tab should validate the username entry (same as Enter).
        KeyEvent {
            code: KeyCode::Tab, ..
        } => match greeter.mode {
            Mode::Username if !greeter.username.value.is_empty() => {
                validate_username(&mut greeter, &ipc).await
            }
            _ => {}
        },

        // Enter validates the current entry, depending on the active mode.
        KeyEvent {
            code: KeyCode::Enter,
            ..
        } => match greeter.mode {
            Mode::Username if !greeter.username.value.is_empty() => {
                validate_username(&mut greeter, &ipc).await
            }

            Mode::Username => {}

            Mode::Password => {
                greeter.working = true;
                greeter.message = None;

                ipc.send(Request::PostAuthMessageResponse {
                    response: Some(greeter.buffer.clone()),
                })
                .await;

                greeter.buffer.zeroize();
                greeter.cursor_offset = 0;
            }

            Mode::Command => {
                greeter.sessions.selected = 0;
                greeter.session_source = SessionSource::Command(greeter.buffer.clone());

                greeter.buffer = greeter.previous_buffer.take().unwrap_or_default();
                greeter.mode = greeter.previous_mode;
                greeter.cursor_offset = 0;
            }

            Mode::Sessions => {
                let session = greeter
                    .sessions
                    .options
                    .get(greeter.sessions.selected)
                    .cloned();

                if let Some(Session { .. }) = session {
                    greeter.session_source = SessionSource::Session(greeter.sessions.selected);
                }

                greeter.mode = greeter.previous_mode;
            }

            Mode::Power => {
                let power_command = greeter.powers.options.get(greeter.powers.selected).cloned();

                if let Some(command) = power_command {
                    power(&mut greeter, command.action).await;
                }

                greeter.mode = greeter.previous_mode;
            }

            _ => {}
        },

        // Do not handle any other controls keybindings
        KeyEvent {
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {}

        // Handle free-form entry of characters.
        KeyEvent {
            code: KeyCode::Char(c),
            ..
        } => insert_key(&mut greeter, c).await,

        // Handle deletion of characters.
        KeyEvent {
            code: KeyCode::Backspace,
            ..
        }
        | KeyEvent {
            code: KeyCode::Delete,
            ..
        } => delete_key(&mut greeter, input.code).await,

        _ => {}
    }

    Ok(())
}

// The text field the current mode edits.
fn field(greeter: &Greeter) -> Option<&String> {
    match greeter.mode {
        Mode::Username => Some(&greeter.username.value),
        Mode::Password | Mode::Command => Some(&greeter.buffer),
        _ => None,
    }
}

// The cursor's position in `value`, as a character index from its start.
fn cursor_index(value: &str, cursor_offset: i16) -> usize {
    let len = value.chars().count() as i16;
    (len + cursor_offset).clamp(0, len) as usize
}

// Byte offset of character `index` in `value` (its length past the end).
fn byte_index(value: &str, index: usize) -> usize {
    value
        .char_indices()
        .nth(index)
        .map_or(value.len(), |(i, _)| i)
}

// Handle insertion of characters into the proper buffer, depending on the
// current mode and the position of the cursor. The string is edited in
// place, so no copies of a password are left behind in freed memory (unless
// it grows past its capacity).
async fn insert_key(greeter: &mut Greeter, c: char) {
    let offset = greeter.cursor_offset;
    let value = match greeter.mode {
        Mode::Username => &mut greeter.username.value,
        Mode::Password | Mode::Command => &mut greeter.buffer,
        _ => return,
    };

    let at = byte_index(value, cursor_index(value, offset));
    value.insert(at, c);
}

// Handle deletion of characters from a prompt into the proper buffer, depending
// on the current mode, whether Backspace or Delete was pressed and the position
// of the cursor.
async fn delete_key(greeter: &mut Greeter, key: KeyCode) {
    let offset = greeter.cursor_offset;
    let value = match greeter.mode {
        Mode::Username => &mut greeter.username.value,
        Mode::Password | Mode::Command => &mut greeter.buffer,
        _ => return,
    };

    let cursor = cursor_index(value, offset);
    let index = match key {
        KeyCode::Backspace if cursor > 0 => cursor - 1,
        KeyCode::Delete if cursor < value.chars().count() => cursor,
        _ => return,
    };

    value.remove(byte_index(value, index));

    // Delete removes the character after the cursor: the cursor keeps its
    // place, now one character closer to the end.
    if let KeyCode::Delete = key {
        greeter.cursor_offset += 1;
    }
}

// Creates a `greetd` session for the provided username.
async fn validate_username(greeter: &mut Greeter, ipc: &Ipc) {
    greeter.working = true;
    greeter.message = None;

    ipc.send(Request::CreateSession {
        username: greeter.username.value.clone(),
    })
    .await;
    greeter.buffer.zeroize();
    greeter.cursor_offset = 0;
}

#[cfg(test)]
mod tests;
