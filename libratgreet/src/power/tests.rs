use crate::{Greeter, event::Event, model::power_item::Power};

use super::{PowerOption, power};

async fn command_for(configured: &str, setsid: bool) -> Vec<String> {
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    let mut greeter = Greeter::default();
    greeter.events = Some(tx);
    greeter.power_setsid = setsid;
    greeter.powers.options = vec![Power {
        action: PowerOption::Shutdown,
        label: "Shut down".into(),
        command: Some(configured.into()),
    }];
    power(&mut greeter, PowerOption::Shutdown).await;
    let Some(Event::PowerCommand(command)) = rx.recv().await else {
        panic!("no power command");
    };
    let command = command.as_std();
    std::iter::once(command.get_program())
        .chain(command.get_args())
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect()
}

/// Extra spaces in the configured command used to become empty arguments.
#[tokio::test]
async fn commands_split_on_any_whitespace() {
    assert_eq!(
        command_for("  systemctl   poweroff\t-i ", false).await,
        ["systemctl", "poweroff", "-i"]
    );
    assert_eq!(
        command_for("systemctl  poweroff", true).await,
        ["setsid", "systemctl", "poweroff"]
    );
}
