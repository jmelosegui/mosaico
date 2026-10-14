use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;

use mosaico_core::Error;
use mosaico_core::system::LaunchCommand;

use super::{join, split};

fn command(program: &str, args: &[&str]) -> LaunchCommand {
    LaunchCommand {
        program: PathBuf::from(program),
        args: args.iter().map(|arg| arg.to_string()).collect(),
    }
}

#[test]
fn the_program_is_written_in_quotes_before_its_arguments() {
    // Arrange
    let start = LaunchCommand::autostart(r"C:\Program Files\mosaico\mosaico.exe");

    // Act
    let line = join(&start).unwrap();

    // Assert
    assert_eq!(line, r#""C:\Program Files\mosaico\mosaico.exe" start"#);
}

#[test]
fn arguments_with_spaces_or_quotes_are_quoted_and_escaped() {
    // Arrange
    let command = command(
        r"C:\mosaico.exe",
        &["two words", r#"say "hi""#, r"C:\with space\", ""],
    );

    // Act
    let line = join(&command).unwrap();

    // Assert
    assert_eq!(
        line,
        r#""C:\mosaico.exe" "two words" "say \"hi\"" "C:\with space\\" """#
    );
}

#[test]
fn backslashes_are_kept_as_they_are_outside_quotes() {
    // Arrange
    let command = command(r"C:\mosaico.exe", &[r"C:\config\dir", r"a\\b"]);

    // Act
    let line = join(&command).unwrap();

    // Assert
    assert_eq!(line, r#""C:\mosaico.exe" C:\config\dir a\\b"#);
}

#[test]
fn a_command_the_value_cannot_hold_is_refused() {
    // Arrange
    let not_unicode = PathBuf::from(OsString::from_wide(&[0x43, 0x3A, 0x5C, 0xD800]));
    let commands = [
        command(r#"C:\odd"name.exe"#, &["start"]),
        command("", &["start"]),
        LaunchCommand::autostart(not_unicode),
        command(r"C:\mosaico.exe", &["st\0art"]),
    ];

    for command in commands {
        // Act
        let line = join(&command);

        // Assert
        assert!(
            matches!(line, Err(Error::Unsupported(_))),
            "{command:?} gave {line:?}"
        );
    }
}

#[test]
fn a_written_command_splits_back_into_the_same_command() {
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
        let read_back = split(&join(&command).unwrap(), |_| false);

        // Assert
        assert_eq!(read_back, Some(command));
    }
}
