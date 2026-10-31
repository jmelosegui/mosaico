use super::*;

fn labels(state: &TrayState) -> Vec<String> {
    state
        .menu()
        .into_iter()
        .map(|item| match item {
            TrayItem::Command { label, .. } => label,
            TrayItem::Separator => "-".to_string(),
        })
        .collect()
}

#[test]
fn a_running_daemon_offers_to_pause_and_shows_its_version() {
    // Arrange
    let state = TrayState::new("0.12.0");

    // Act
    let labels = labels(&state);

    // Assert
    assert_eq!(state.tooltip(), "mosaico 0.12.0");
    assert_eq!(
        labels,
        [
            "Settings\u{2026}",
            "-",
            "Pause hotkeys",
            "Reload config",
            "-",
            "Open config folder",
            "Open log",
            "-",
            "Quit mosaico",
        ]
    );
}

#[test]
fn a_paused_daemon_offers_to_resume_and_says_so_in_the_tooltip() {
    // Arrange
    let state = TrayState {
        version: "0.12.0".into(),
        paused: true,
    };

    // Act
    let labels = labels(&state);

    // Assert
    assert_eq!(state.tooltip(), "mosaico 0.12.0 (hotkeys paused)");
    assert_eq!(labels[2], "Resume hotkeys");
}

#[test]
fn the_default_command_is_the_first_entry_of_the_menu() {
    // Act
    let first = TrayState::new("1.0.0").menu().remove(0);

    // Assert
    assert!(matches!(
        first,
        TrayItem::Command { command, .. } if command == TrayCommand::DEFAULT
    ));
}

#[test]
fn an_invalid_config_notice_names_the_file_and_the_reason() {
    // Act
    let notice = TrayNotice::invalid_config("bar.toml", "expected `=` at line 3");

    // Assert
    assert_eq!(notice.title, "bar.toml was not loaded");
    assert!(notice.body.starts_with("expected `=` at line 3\n"));
}

#[test]
fn an_invalid_config_notice_keeps_where_and_what_of_a_long_reason() {
    // Arrange
    let reason = "TOML parse error at line 131, column 6\n    |\n131 | a = = b\n    |      ^\nstring values must be quoted\n";

    // Act
    let notice = TrayNotice::invalid_config("config.toml", reason);

    // Assert
    assert_eq!(
        notice.body,
        "TOML parse error at line 131, column 6: string values must be quoted\n\
         The settings in use stay until the file is fixed."
    );
}
