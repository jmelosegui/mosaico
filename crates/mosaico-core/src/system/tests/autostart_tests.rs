use std::fs;
use std::path::{Path, PathBuf};

use super::*;

/// A fresh directory for `test` under the system temp directory.
fn temp_dir(test: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("mosaico-autostart-tests")
        .join(format!("{test}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Creates a file at `path` to stand in for an executable.
fn program_at(path: PathBuf) -> PathBuf {
    fs::write(&path, "").unwrap();
    path
}

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

#[test]
fn nothing_registered_is_disabled() {
    // Act
    let status = AutostartStatus::of(None, Path::new("/apps/mosaico"));

    // Assert
    assert_eq!(status, AutostartStatus::Disabled);
}

#[test]
fn this_executable_started_the_way_enabling_does_is_enabled() {
    // Arrange
    let exe = Path::new("/apps/mosaico");

    // Act
    let status = AutostartStatus::of(Some(LaunchCommand::autostart(exe)), exe);

    // Assert
    assert_eq!(status, AutostartStatus::Enabled);
}

#[test]
fn another_spelling_of_this_executable_is_enabled() {
    // Arrange
    let dir = temp_dir("spelling");
    let exe = program_at(dir.join("mosaico"));
    fs::create_dir(dir.join("bin")).unwrap();
    let spelled = dir.join("bin").join("..").join("mosaico");

    // Act
    let status = AutostartStatus::of(Some(LaunchCommand::autostart(spelled)), &exe);

    // Assert
    assert_eq!(status, AutostartStatus::Enabled);
}

#[test]
fn this_executable_with_other_arguments_is_another_command() {
    // Arrange
    let exe = Path::new("/apps/mosaico");
    let registered = LaunchCommand {
        program: exe.to_path_buf(),
        args: vec!["daemon".into()],
    };

    // Act
    let status = AutostartStatus::of(Some(registered.clone()), exe);

    // Assert
    assert_eq!(status, AutostartStatus::Other(registered));
}

#[test]
fn another_program_that_exists_is_another_command() {
    // Arrange
    let dir = temp_dir("other");
    let registered = LaunchCommand::autostart(program_at(dir.join("other-mosaico")));

    // Act
    let status = AutostartStatus::of(Some(registered.clone()), &dir.join("mosaico"));

    // Assert
    assert_eq!(status, AutostartStatus::Other(registered));
}

#[test]
fn a_program_that_does_not_exist_is_missing() {
    // Arrange
    let dir = temp_dir("missing");
    let registered = LaunchCommand::autostart(dir.join("moved-away").join("mosaico"));

    // Act
    let status = AutostartStatus::of(Some(registered.clone()), &dir.join("mosaico"));

    // Assert
    assert_eq!(status, AutostartStatus::Missing(registered));
}
