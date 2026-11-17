use std::sync::Arc;

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::RwLock;

use crate::{
    greeter::{Greeter, Mode},
    ipc::Ipc,
    model::{masked::MaskedString, sessions::SessionSource},
};

use super::handle;

#[tokio::test]
async fn ctrl_u() {
    let greeter = Arc::new(RwLock::new(Greeter::default()));

    {
        let mut greeter = greeter.write().await;
        greeter.mode = Mode::Username;
        greeter.username = MaskedString::from("apognu".to_string(), None);
    }

    let result = handle(
        greeter.clone(),
        KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL),
        Ipc::new(),
    )
    .await;

    {
        let status = greeter.read().await;

        assert!(result.is_ok());
        assert_eq!(status.username.value, "".to_string());
    }

    {
        let mut greeter = greeter.write().await;
        greeter.mode = Mode::Password;
        greeter.buffer = "password".to_string();
    }

    let result = handle(
        greeter.clone(),
        KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL),
        Ipc::new(),
    )
    .await;

    {
        let status = greeter.read().await;

        assert!(result.is_ok());
        assert_eq!(status.buffer, "".to_string());
    }

    {
        let mut greeter = greeter.write().await;
        greeter.mode = Mode::Command;
        greeter.buffer = "newcommand".to_string();
    }

    let result = handle(
        greeter.clone(),
        KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL),
        Ipc::new(),
    )
    .await;

    {
        let status = greeter.read().await;

        assert!(result.is_ok());
        assert_eq!(status.buffer, "".to_string());
    }
}

#[tokio::test]
async fn escape() {
    let greeter = Arc::new(RwLock::new(Greeter::default()));

    {
        let mut greeter = greeter.write().await;
        greeter.previous_mode = Mode::Username;
        greeter.mode = Mode::Command;
        greeter.previous_buffer = Some("apognu".to_string());
        greeter.buffer = "newcommand".to_string();
        greeter.cursor_offset = 2;
    }

    let result = handle(
        greeter.clone(),
        KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()),
        Ipc::new(),
    )
    .await;

    {
        let status = greeter.read().await;

        assert!(result.is_ok());
        assert_eq!(status.mode, Mode::Username);
        assert_eq!(status.buffer, "apognu".to_string());
        assert!(status.previous_buffer.is_none());
        assert_eq!(status.cursor_offset, 0);
    }

    for mode in [Mode::Sessions, Mode::Power] {
        {
            let mut greeter = greeter.write().await;
            greeter.previous_mode = Mode::Username;
            greeter.mode = mode;
        }

        let result = handle(
            greeter.clone(),
            KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()),
            Ipc::new(),
        )
        .await;

        {
            let status = greeter.read().await;

            assert!(result.is_ok());
            assert_eq!(status.mode, Mode::Username);
        }
    }
}

#[tokio::test]
async fn left_right() {
    let greeter = Arc::new(RwLock::new(Greeter::default()));
    greeter.write().await.username = MaskedString::from("ab".to_string(), None);

    let result = handle(
        greeter.clone(),
        KeyEvent::new(KeyCode::Left, KeyModifiers::empty()),
        Ipc::new(),
    )
    .await;

    {
        let status = greeter.read().await;

        assert!(result.is_ok());
        assert_eq!(status.cursor_offset, -1);
    }

    let _ = handle(
        greeter.clone(),
        KeyEvent::new(KeyCode::Right, KeyModifiers::empty()),
        Ipc::new(),
    )
    .await;
    let result = handle(
        greeter.clone(),
        KeyEvent::new(KeyCode::Right, KeyModifiers::empty()),
        Ipc::new(),
    )
    .await;

    {
        let status = greeter.read().await;

        assert!(result.is_ok());
        // Clamped at the end of the text.
        assert_eq!(status.cursor_offset, 0);
    }
}

#[tokio::test]
async fn f2() {
    let greeter = Arc::new(RwLock::new(Greeter::default()));

    {
        let mut greeter = greeter.write().await;
        greeter.mode = Mode::Username;
        greeter.buffer = "apognu".to_string();
        greeter.session_source = SessionSource::Command("thecommand".to_string());
    }

    let result = handle(
        greeter.clone(),
        KeyEvent::new(KeyCode::F(2), KeyModifiers::empty()),
        Ipc::new(),
    )
    .await;

    {
        let status = greeter.read().await;

        assert!(result.is_ok());
        assert_eq!(status.mode, Mode::Command);
        assert_eq!(status.previous_buffer, Some("apognu".to_string()));
        assert_eq!(status.buffer, "thecommand".to_string());
    }

    for mode in [Mode::Sessions, Mode::Power] {
        {
            let mut greeter = greeter.write().await;
            greeter.previous_mode = Mode::Username;
            greeter.mode = mode;
        }

        let result = handle(
            greeter.clone(),
            KeyEvent::new(KeyCode::F(2), KeyModifiers::empty()),
            Ipc::new(),
        )
        .await;

        {
            let status = greeter.read().await;

            assert!(result.is_ok());
            assert_eq!(status.mode, Mode::Command);
            assert_eq!(status.previous_mode, Mode::Username);
        }
    }
}

#[tokio::test]
async fn f_menu() {
    let greeter = Arc::new(RwLock::new(Greeter::default()));

    for (key, mode) in [
        (KeyCode::F(3), Mode::Sessions),
        (KeyCode::F(12), Mode::Power),
    ] {
        {
            let mut greeter = greeter.write().await;
            greeter.mode = Mode::Username;
            greeter.buffer = "apognu".to_string();
        }

        let result = handle(
            greeter.clone(),
            KeyEvent::new(key, KeyModifiers::empty()),
            Ipc::new(),
        )
        .await;

        {
            let status = greeter.read().await;

            assert!(result.is_ok());
            assert_eq!(status.mode, mode);
            assert_eq!(status.buffer, "apognu".to_string());
        }

        for mode in [Mode::Sessions, Mode::Power] {
            {
                let mut greeter = greeter.write().await;
                greeter.previous_mode = Mode::Username;
                greeter.mode = mode;
            }

            let result = handle(
                greeter.clone(),
                KeyEvent::new(KeyCode::F(2), KeyModifiers::empty()),
                Ipc::new(),
            )
            .await;

            {
                let status = greeter.read().await;

                assert!(result.is_ok());
                assert_eq!(status.mode, Mode::Command);
                assert_eq!(status.previous_mode, Mode::Username);
            }
        }
    }
}

#[tokio::test]
async fn f_menu_rebinded() {
    let greeter = Arc::new(RwLock::new(Greeter::default()));

    for (key, mode) in [
        (KeyCode::F(1), Mode::Sessions),
        (KeyCode::F(11), Mode::Power),
    ] {
        {
            let mut greeter = greeter.write().await;
            greeter.kb_command = 3;
            greeter.kb_sessions = 1;
            greeter.kb_power = 11;
            greeter.mode = Mode::Username;
            greeter.buffer = "apognu".to_string();
        }

        let result = handle(
            greeter.clone(),
            KeyEvent::new(key, KeyModifiers::empty()),
            Ipc::new(),
        )
        .await;

        {
            let status = greeter.read().await;

            assert!(result.is_ok());
            assert_eq!(status.mode, mode);
            assert_eq!(status.buffer, "apognu".to_string());
        }

        for mode in [Mode::Sessions, Mode::Power] {
            {
                let mut greeter = greeter.write().await;
                greeter.previous_mode = Mode::Username;
                greeter.mode = mode;
            }

            let result = handle(
                greeter.clone(),
                KeyEvent::new(KeyCode::F(3), KeyModifiers::empty()),
                Ipc::new(),
            )
            .await;

            {
                let status = greeter.read().await;

                assert!(result.is_ok());
                assert_eq!(status.mode, Mode::Command);
                assert_eq!(status.previous_mode, Mode::Username);
            }
        }
    }
}

#[tokio::test]
async fn ctrl_a_e() {
    let greeter = Arc::new(RwLock::new(Greeter::default()));

    {
        let mut greeter = greeter.write().await;
        greeter.mode = Mode::Command;
        greeter.buffer = "123456789".to_string();
    }

    let result = handle(
        greeter.clone(),
        KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL),
        Ipc::new(),
    )
    .await;

    {
        let status = greeter.read().await;

        assert!(result.is_ok());
        assert_eq!(status.cursor_offset, -9);
    }

    let result = handle(
        greeter.clone(),
        KeyEvent::new(KeyCode::Char('e'), KeyModifiers::CONTROL),
        Ipc::new(),
    )
    .await;

    {
        let status = greeter.read().await;

        assert!(result.is_ok());
        assert_eq!(status.cursor_offset, 0);
    }
}

async fn press(greeter: &Arc<RwLock<Greeter>>, ipc: &Ipc, code: KeyCode) {
    handle(
        greeter.clone(),
        KeyEvent::new(code, KeyModifiers::empty()),
        ipc.clone(),
    )
    .await
    .unwrap();
}

async fn type_text(greeter: &Arc<RwLock<Greeter>>, ipc: &Ipc, text: &str) {
    for c in text.chars() {
        press(greeter, ipc, KeyCode::Char(c)).await;
    }
}

/// The cursor moved within the username used to carry over to the password
/// prompt: typed without a redraw in between (a fast typist, or a password
/// manager), the password's characters were inserted out of order.
#[tokio::test]
async fn the_password_starts_with_the_cursor_at_its_end() {
    let greeter = Arc::new(RwLock::new(Greeter::default()));
    let ipc = Ipc::new();
    type_text(&greeter, &ipc, "alice").await;
    press(&greeter, &ipc, KeyCode::Left).await;
    press(&greeter, &ipc, KeyCode::Left).await;
    press(&greeter, &ipc, KeyCode::Enter).await;
    {
        // greetd asks for the password.
        let mut greeter = greeter.write().await;
        assert_eq!(greeter.username.value, "alice");
        greeter.working = false;
        greeter.mode = Mode::Password;
        greeter.set_prompt("Password:");
    }
    type_text(&greeter, &ipc, "secret").await;
    assert_eq!(greeter.read().await.buffer, "secret");
}

/// Left past the start, or Right past the end, doesn't move the insertion
/// point out of the text.
#[tokio::test]
async fn the_cursor_stays_within_the_text() {
    let greeter = Arc::new(RwLock::new(Greeter::default()));
    let ipc = Ipc::new();
    type_text(&greeter, &ipc, "bc").await;
    for _ in 0..5 {
        press(&greeter, &ipc, KeyCode::Left).await;
    }
    type_text(&greeter, &ipc, "a").await;
    for _ in 0..5 {
        press(&greeter, &ipc, KeyCode::Right).await;
    }
    type_text(&greeter, &ipc, "d").await;
    assert_eq!(greeter.read().await.username.value, "abcd");
    press(&greeter, &ipc, KeyCode::Left).await;
    press(&greeter, &ipc, KeyCode::Backspace).await;
    press(&greeter, &ipc, KeyCode::Delete).await;
    assert_eq!(greeter.read().await.username.value, "ab");
}

/// No sessions installed: Down in the (empty) session menu used to
/// underflow `len() - 1`.
#[tokio::test]
async fn down_in_an_empty_menu() {
    let greeter = Arc::new(RwLock::new(Greeter::default()));
    let ipc = Ipc::new();
    for mode in [Mode::Sessions, Mode::Power] {
        greeter.write().await.mode = mode;
        press(&greeter, &ipc, KeyCode::Down).await;
        press(&greeter, &ipc, KeyCode::Enter).await;
    }
    assert_eq!(greeter.read().await.sessions.selected, 0);
    assert_eq!(greeter.read().await.powers.selected, 0);
}

/// In-place edits find byte positions for multi-byte characters.
#[tokio::test]
async fn editing_between_multi_byte_characters() {
    let greeter = Arc::new(RwLock::new(Greeter::default()));
    let ipc = Ipc::new();
    type_text(&greeter, &ipc, "jérme").await;
    press(&greeter, &ipc, KeyCode::Left).await;
    press(&greeter, &ipc, KeyCode::Left).await;
    type_text(&greeter, &ipc, "ô").await;
    assert_eq!(greeter.read().await.username.value, "jérôme");
    press(&greeter, &ipc, KeyCode::Left).await;
    press(&greeter, &ipc, KeyCode::Backspace).await;
    press(&greeter, &ipc, KeyCode::Delete).await;
    assert_eq!(greeter.read().await.username.value, "jéme");
}
