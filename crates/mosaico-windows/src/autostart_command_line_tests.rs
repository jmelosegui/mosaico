use std::path::PathBuf;

use mosaico_core::system::LaunchCommand;

use super::join;

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
    let line = join(&start);

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
    let line = join(&command);

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
    let line = join(&command);

    // Assert
    assert_eq!(line, r#""C:\mosaico.exe" C:\config\dir a\\b"#);
}
