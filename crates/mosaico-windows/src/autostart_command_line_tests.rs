use std::path::PathBuf;

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
        let read_back = split(&join(&command).unwrap());

        // Assert
        assert_eq!(read_back, Some(command));
    }
}

#[test]
fn a_quoted_program_ends_at_its_closing_quote() {
    // Act
    let parsed = split(r#"  "C:\Program Files\mosaico\mosaico.exe"   start"#);

    // Assert
    assert_eq!(
        parsed,
        Some(LaunchCommand::autostart(
            r"C:\Program Files\mosaico\mosaico.exe"
        ))
    );
}

#[test]
fn an_unquoted_program_ends_at_the_first_space() {
    // Act
    let parsed = split(r"C:\Program Files\mosaico\mosaico.exe start");

    // Assert
    assert_eq!(
        parsed,
        Some(command(
            r"C:\Program",
            &[r"Files\mosaico\mosaico.exe", "start"]
        ))
    );
}

#[test]
fn arguments_split_as_windows_programs_read_them() {
    // Act
    let parsed = split(r#""C:\mosaico.exe" C:\a\b "C:\c d\\" a\\\"b "x""y" ab"c d"e"#);

    // Assert
    assert_eq!(
        parsed,
        Some(command(
            r"C:\mosaico.exe",
            &[r"C:\a\b", r"C:\c d\", r#"a\"b"#, r#"x"y"#, "abc de"]
        ))
    );
}

#[test]
fn a_line_without_a_program_names_nothing() {
    for line in ["", " \t ", r#""""#, r#""" start"#] {
        // Act
        let parsed = split(line);

        // Assert
        assert_eq!(parsed, None, "{line:?}");
    }
}
