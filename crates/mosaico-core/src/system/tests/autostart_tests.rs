use std::path::PathBuf;

use super::*;

#[test]
fn autostart_runs_the_executable_with_start() {
    // Act
    let command = LaunchCommand::autostart("/apps/mosaico");

    // Assert
    assert_eq!(command.program, PathBuf::from("/apps/mosaico"));
    assert_eq!(command.args, ["start"]);
}

#[test]
fn a_command_shows_its_program_in_quotes_then_its_arguments() {
    // Arrange
    let command = LaunchCommand {
        program: PathBuf::from("/my apps/mosaico"),
        args: vec!["start".into(), "two words".into()],
    };

    // Act
    let shown = command.to_string();

    // Assert
    assert_eq!(shown, "\"/my apps/mosaico\" start \"two words\"");
}
