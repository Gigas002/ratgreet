use libgreetd_stub::SessionOptions;
use tempfile::tempdir;

use super::common::IntegrationRunner;

#[tokio::test]
async fn show_greet() {
    let opts = SessionOptions {
        username: "apognu".to_string(),
        password: "password".to_string(),
        mfa: false,
    };

    let mut runner = IntegrationRunner::new(
        opts,
        Some(|greeter| {
            greeter.greeting = Some("Lorem ipsum dolor sit amet".to_string());
        }),
    )
    .await;

    let events = tokio::task::spawn({
        let mut runner = runner.clone();

        async move {
            runner.wait_for_render().await;

            assert!(runner.output().await.contains("Lorem ipsum dolor sit amet"));
        }
    });

    runner.join_until_end(events).await;
}

#[tokio::test]
async fn show_wrapped_greet() {
    let opts = SessionOptions {
        username: "apognu".to_string(),
        password: "password".to_string(),
        mfa: false,
    };

    let mut runner = IntegrationRunner::new_with_size(
        opts,
        Some(|greeter| {
            greeter.greeting = Some("Lorem ipsum dolor sit amet".to_string());
        }),
        (20, 20),
    )
    .await;

    let events = tokio::task::spawn({
        let mut runner = runner.clone();

        async move {
            runner.wait_for_render().await;

            let output = runner.output().await;

            assert!(output.contains("┌ Authenticate into┐"));
            assert!(output.contains("│    Lorem ipsum   │"));
            assert!(output.contains("│  dolor sit amet  │"));
            assert!(output.contains("└──────────────────┘"));
        }
    });

    runner.join_until_end(events).await;
}

#[tokio::test]
async fn show_greeting_from_theme_file() {
    let dir = tempdir().unwrap();
    let theme_path = dir.path().join("theme.toml");
    std::fs::write(
        &theme_path,
        r#"
[ui]
greeting = "Configured greeting"
"#,
    )
    .unwrap();

    let opts = SessionOptions {
        username: "apognu".to_string(),
        password: "password".to_string(),
        mfa: false,
    };

    let mut runner =
        IntegrationRunner::new_with_config(opts, None, None, Some(theme_path), (200, 40)).await;

    let events = tokio::task::spawn({
        let mut runner = runner.clone();

        async move {
            runner.wait_for_render().await;
            assert!(runner.output().await.contains("Configured greeting"));
        }
    });

    runner.join_until_end(events).await;
}
