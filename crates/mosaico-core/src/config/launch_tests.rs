use crate::config::Modifier::{Alt, Shift};
use crate::config::rules::KeybindingsFile;

use super::*;

const FILE: &str = r#"
[[keybinding]]
action = "retile"
key = "R"
modifiers = ["alt"]

[[launch]]
command = "wt.exe"
key = "Enter"
modifiers = ["alt"]

[[launch]]
command = "msedge.exe"
args = ["--new-window", "https://example.com"]
key = "B"
modifiers = ["alt", "shift"]
focus = true
match_path = "msedge.exe"
"#;

fn parse(content: &str) -> Vec<Result<LaunchBinding>> {
    parse_launches(Path::new("keybindings.toml"), content).expect("the file is TOML")
}

fn launch(command: &str) -> LaunchBinding {
    LaunchBinding {
        command: command.into(),
        args: Vec::new(),
        key: "B".into(),
        modifiers: vec![Alt],
        focus: true,
        windows: WindowMatch::default(),
    }
}

#[test]
fn every_field_is_read_and_left_out_ones_take_their_defaults() {
    // Act
    let entries = parse(FILE);

    // Assert
    let launches: Vec<LaunchBinding> = entries.into_iter().map(|e| e.expect("valid")).collect();
    assert_eq!(
        launches,
        [
            LaunchBinding {
                command: "wt.exe".into(),
                args: Vec::new(),
                key: "Enter".into(),
                modifiers: vec![Alt],
                focus: false,
                windows: WindowMatch::default(),
            },
            LaunchBinding {
                command: "msedge.exe".into(),
                args: vec!["--new-window".into(), "https://example.com".into()],
                key: "B".into(),
                modifiers: vec![Alt, Shift],
                focus: true,
                windows: WindowMatch {
                    match_path: Some("msedge.exe".into()),
                    ..WindowMatch::default()
                },
            },
        ]
    );
}

#[test]
fn a_file_without_launch_tables_has_no_launch_bindings() {
    // Act
    let entries = parse("[[keybinding]]\naction = \"retile\"\nkey = \"R\"\nmodifiers = []\n");

    // Assert
    assert!(entries.is_empty());
}

#[test]
fn a_bad_entry_fails_alone_and_names_its_position() {
    // Arrange
    let content = r#"
[[launch]]
command = "notepad.exe"
key = "N"
modifiers = ["alt"]

[[launch]]
key = "B"
modifiers = ["alt"]

[[launch]]
command = "  "
key = "C"
modifiers = ["alt"]

[[launch]]
command = "x"
args = "--one-string"
key = "D"
modifiers = ["alt"]
"#;

    // Act
    let entries = parse(content);

    // Assert
    assert_eq!(entries.len(), 4);
    assert!(entries[0].is_ok());
    let errors: Vec<String> = entries[1..]
        .iter()
        .map(|e| e.as_ref().expect_err("invalid").to_string())
        .collect();
    assert!(
        errors[0].contains("launch binding 2") && errors[0].contains("command"),
        "{}",
        errors[0]
    );
    assert!(
        errors[1].contains("launch binding 3: `command` is empty"),
        "{}",
        errors[1]
    );
    assert!(errors[2].contains("launch binding 4"), "{}", errors[2]);
}

#[test]
fn launch_not_being_a_list_of_tables_fails_the_whole_file() {
    // Act
    let parsed = parse_launches(Path::new("keybindings.toml"), "launch = \"wt.exe\"\n");

    // Assert
    assert!(parsed.is_err());
}

#[test]
fn keybindings_are_read_as_before_from_a_file_with_launch_tables() {
    // Act
    let file: KeybindingsFile = toml::from_str(FILE).expect("parses");

    // Assert
    assert_eq!(file.keybinding.len(), 1);
}

#[test]
fn without_match_fields_windows_match_on_the_name_of_the_command() {
    // Arrange
    let edge = launch("msedge.exe");
    let by_path = launch(r"C:\Tools\Code.exe");
    let no_extension = launch("firefox");

    // Act
    let matched = [
        edge.matches(
            "Chrome_WidgetWin_1",
            "New tab",
            r"C:\Program Files\Edge\MSEDGE.EXE",
        ),
        edge.matches(
            "Chrome_WidgetWin_1",
            "msedge",
            r"C:\Program Files\Edge\msedge_proxy.exe",
        ),
        edge.matches("Chrome_WidgetWin_1", "msedge", ""),
        by_path.matches("", "", r"D:\Other\code.exe"),
        no_extension.matches("", "", "/usr/lib/firefox/firefox"),
        no_extension.matches("", "", r"C:\Mozilla\firefox.exe"),
    ];

    // Assert
    assert_eq!(matched, [true, false, false, true, true, true]);
}

#[test]
fn match_fields_replace_the_command_name_and_mean_what_they_mean_in_rules() {
    // Arrange
    let mut terminal = launch("wt.exe");
    terminal.windows.match_path = Some("WindowsTerminal.exe".into());
    let mut notes = launch("notepad.exe");
    notes.windows.match_class = Some("Notepad".into());
    notes.windows.match_title = Some("todo".into());

    // Act
    let matched = [
        terminal.matches("CASCADIA", "pwsh", r"C:\Apps\WindowsTerminal.exe"),
        terminal.matches("CASCADIA", "pwsh", r"C:\Apps\wt.exe"),
        notes.matches("notepad", "TODO.txt - Notepad", r"C:\Windows\notepad.exe"),
        notes.matches("notepad", "other.txt - Notepad", r"C:\Windows\notepad.exe"),
    ];

    // Assert
    assert_eq!(matched, [true, false, true, false]);
}

#[test]
fn the_combo_is_checked_as_for_keybindings() {
    // Arrange
    let mut bad = launch("x");
    bad.key = "NotAKey".into();

    // Act
    let good = launch("x").combo();
    let bad = bad.combo();

    // Assert
    assert_eq!(good.expect("a key").to_string(), "Alt+B");
    assert!(bad.is_err());
}
