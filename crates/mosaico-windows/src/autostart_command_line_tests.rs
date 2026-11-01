use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;

use mosaico_core::Error;
use mosaico_core::system::LaunchCommand;

use super::{Action, from_action, join};

fn command(program: &str, args: &[&str]) -> LaunchCommand {
    LaunchCommand {
        program: PathBuf::from(program),
        args: args.iter().map(|arg| arg.to_string()).collect(),
    }
}

#[test]
fn the_program_is_kept_apart_from_its_arguments() {
    // Arrange
    let start = LaunchCommand::autostart(r"C:\Program Files\mosaico\mosaico.exe");

    // Act
    let action = join(&start).unwrap();

    // Assert
    assert_eq!(
        action,
        Action {
            program: r"C:\Program Files\mosaico\mosaico.exe".into(),
            arguments: "start --quiet".into(),
        }
    );
}

#[test]
fn arguments_with_spaces_or_quotes_are_quoted_and_escaped() {
    // Arrange
    let command = command(
        r"C:\mosaico.exe",
        &["two words", r#"say "hi""#, r"C:\with space\", "", "a\tb"],
    );

    // Act
    let action = join(&command).unwrap();

    // Assert
    assert_eq!(
        action.arguments,
        "\"two words\" \"say \\\"hi\\\"\" \"C:\\with space\\\\\" \"\" \"a\tb\""
    );
}

#[test]
fn backslashes_are_kept_as_they_are_outside_quotes() {
    // Arrange
    let command = command(r"C:\mosaico.exe", &[r"C:\config\dir", r"a\\b"]);

    // Act
    let action = join(&command).unwrap();

    // Assert
    assert_eq!(action.arguments, r"C:\config\dir a\\b");
}

#[test]
fn a_command_the_action_cannot_hold_is_refused() {
    // Arrange
    let not_unicode = PathBuf::from(OsString::from_wide(&[0x43, 0x3A, 0x5C, 0xD800]));
    let commands = [
        command(r#"C:\odd"name.exe"#, &["start"]),
        command("", &["start"]),
        LaunchCommand::autostart(not_unicode),
        command(r"C:\mosaico.exe", &["st\0art"]),
        command(r"C:\mosaico.exe", &["two\nlines"]),
        command(r"C:\mosaico.exe", &["%APPDATA%"]),
        command(r"C:\%USERNAME%\mosaico.exe", &["start"]),
    ];

    for command in commands {
        // Act
        let action = join(&command);

        // Assert
        assert!(
            matches!(action, Err(Error::Unsupported(_))),
            "{command:?} gave {action:?}"
        );
    }
}

#[test]
fn a_written_command_reads_back_as_the_same_command() {
    // Arrange
    let commands = [
        LaunchCommand::autostart(r"C:\Program Files\mosaico\mosaico.exe"),
        command(r"C:\mosaico.exe", &[]),
        command(
            r"C:\mosaico.exe",
            &[
                "two words",
                r#"say "hi""#,
                r#"a\"b"#,
                r"C:\with space\",
                "",
                r"a\b",
            ],
        ),
    ];

    for command in commands {
        // Act
        let action = join(&command).unwrap();
        let read_back = from_action(&action.program, &action.arguments);

        // Assert
        assert_eq!(read_back, command);
    }
}

#[test]
fn a_program_in_quotes_is_read_without_them() {
    // Act
    let command = from_action(r#""C:\Program Files\mosaico.exe""#, "start --quiet");

    // Assert
    assert_eq!(
        command,
        LaunchCommand::autostart(r"C:\Program Files\mosaico.exe")
    );
}
