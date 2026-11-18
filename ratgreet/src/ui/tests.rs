use std::sync::Arc;

use libratgreet::{
    Greeter, Mode,
    model::{menu::Menu, power_item::Power, sessions::Session},
};
use ratatui::{Terminal, backend::TestBackend};
use tokio::sync::RwLock;

use super::{common::style::Theme, draw};

const MODES: [Mode; 7] = [
    Mode::Username,
    Mode::Password,
    Mode::Action,
    Mode::Command,
    Mode::Sessions,
    Mode::Power,
    Mode::Processing,
];

fn greeter(mode: Mode, width: u16, padding: u16, text: bool) -> Greeter {
    let mut greeter = Greeter::default();
    greeter.mode = mode;
    greeter.width = width;
    greeter.window_padding = padding;
    greeter.container_padding = padding + 1;
    greeter.prompt_padding = padding;
    greeter.time = true;
    if mode == Mode::Password {
        greeter.set_prompt("Password:");
        greeter.buffer = "secret".into();
    }
    if text {
        greeter.greeting = Some("Welcome to a host with a long banner line\n".repeat(3));
        greeter.message = Some("Authentication failed, please try again.".into());
    }
    greeter.username.value = "alice".into();
    greeter.sessions = Menu {
        title: "Change session".into(),
        options: (0..5)
            .map(|i| Session {
                name: format!("Session {i}"),
                ..Session::default()
            })
            .collect(),
        selected: 0,
    };
    greeter.powers = Menu {
        title: "Power options".into(),
        options: vec![Power::default(), Power::default()],
        selected: 1,
    };
    greeter
}

/// Every screen, at terminal sizes from tiny to large and with theme widths
/// and paddings smaller and larger than the terminal: drawing must never
/// panic, since a panicking greeter leaves nobody able to log in.
#[test]
fn drawing_never_panics() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let theme = Theme::default();
    let mut failures = Vec::new();

    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    for (cols, rows) in [
        (1, 1),
        (5, 3),
        (20, 5),
        (40, 12),
        (80, 25),
        (81, 24),
        (240, 67),
    ] {
        for width in [1, 8, 30, 80, 100, 400] {
            for padding in [0, 2] {
                for text in [false, true] {
                    for mode in MODES {
                        let result = std::panic::catch_unwind(|| {
                            runtime.block_on(async {
                                let greeter =
                                    Arc::new(RwLock::new(greeter(mode, width, padding, text)));
                                let mut terminal =
                                    Terminal::new(TestBackend::new(cols, rows)).unwrap();
                                draw(greeter, &theme, &mut terminal).await.unwrap();
                            })
                        });
                        if result.is_err() {
                            failures.push(format!(
                                "{cols}x{rows} width={width} padding={padding} text={text} {mode:?}"
                            ));
                        }
                    }
                }
            }
        }
    }
    std::panic::set_hook(previous_hook);

    assert!(
        failures.is_empty(),
        "{} draws panicked:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// A theme wider than the terminal (80 columns on a VGA console) used to
/// put the "please wait" box after a login off screen.
#[test]
fn processing_fits_a_narrow_terminal() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let greeter = Arc::new(RwLock::new(greeter(Mode::Processing, 100, 0, false)));
    let mut terminal = Terminal::new(TestBackend::new(80, 25)).unwrap();
    runtime
        .block_on(draw(greeter, &Theme::default(), &mut terminal))
        .unwrap();
    let screen: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect();
    assert!(screen.contains("Please wait..."), "{screen}");
}
