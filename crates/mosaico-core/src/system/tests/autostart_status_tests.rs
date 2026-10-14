use std::path::{Path, PathBuf};

use crate::Result;

use super::*;

/// An absolute path on any system, named after `name`. Nothing is
/// created there.
fn path(name: &str) -> PathBuf {
    std::env::temp_dir()
        .join("mosaico-autostart-status")
        .join(name)
}

/// What `of` says when `registered` is not what enabling from `exe`
/// registers.
fn other(registered: LaunchCommand, exe: &Path) -> AutostartStatus {
    AutostartStatus::Other {
        registered,
        expected: LaunchCommand::autostart(exe),
    }
}

/// A file system where `files` exist, each pair in `same` names one
/// file, and `far` cannot be checked without waiting.
#[derive(Default)]
struct Files {
    files: Vec<PathBuf>,
    same: Vec<(PathBuf, PathBuf)>,
    far: Vec<PathBuf>,
}

impl FileProbe for Files {
    fn is_file(&self, path: &Path) -> Option<bool> {
        if self.far.iter().any(|far| far == path) {
            return None;
        }
        Some(self.files.iter().any(|file| file == path))
    }

    fn same_file(&self, a: &Path, b: &Path) -> bool {
        self.same
            .iter()
            .any(|(x, y)| (x == a && y == b) || (x == b && y == a))
    }
}

/// A file system that must not be asked anything.
struct Untouched;

impl FileProbe for Untouched {
    fn is_file(&self, path: &Path) -> Option<bool> {
        panic!("asked whether {path:?} is a file");
    }

    fn same_file(&self, a: &Path, b: &Path) -> bool {
        panic!("asked whether {a:?} and {b:?} are one file");
    }
}

#[test]
fn nothing_registered_is_disabled() {
    // Act
    let status = AutostartStatus::of(None, &path("mosaico"), &Untouched);

    // Assert
    assert_eq!(status, AutostartStatus::Disabled);
}

#[test]
fn this_executable_started_the_way_enabling_does_is_enabled() {
    // Arrange
    let exe = path("mosaico");

    // Act
    let status = AutostartStatus::of(Some(LaunchCommand::autostart(&exe)), &exe, &Untouched);

    // Assert
    assert_eq!(status, AutostartStatus::Enabled);
}

#[test]
fn another_name_for_this_executable_is_enabled() {
    // Arrange
    let exe = path("mosaico");
    let link = path("link-to-mosaico");
    let files = Files {
        same: vec![(link.clone(), exe.clone())],
        ..Files::default()
    };

    // Act
    let status = AutostartStatus::of(Some(LaunchCommand::autostart(link)), &exe, &files);

    // Assert
    assert_eq!(status, AutostartStatus::Enabled);
}

#[test]
fn this_executable_with_other_arguments_is_another_command() {
    // Arrange
    let exe = path("mosaico");
    let registered = LaunchCommand {
        program: exe.clone(),
        args: vec!["daemon".into()],
    };

    // Act
    let status = AutostartStatus::of(Some(registered.clone()), &exe, &Untouched);

    // Assert
    assert_eq!(status, other(registered, &exe));
}

#[test]
fn another_program_that_exists_is_another_command() {
    // Arrange
    let another = path("another-mosaico");
    let files = Files {
        files: vec![another.clone()],
        ..Files::default()
    };
    let registered = LaunchCommand::autostart(another);

    // Act
    let status = AutostartStatus::of(Some(registered.clone()), &path("mosaico"), &files);

    // Assert
    assert_eq!(status, other(registered, &path("mosaico")));
}

#[test]
fn a_program_that_is_not_a_file_is_missing() {
    // Arrange
    let registered = LaunchCommand::autostart(path("moved-away"));

    // Act
    let status = AutostartStatus::of(
        Some(registered.clone()),
        &path("mosaico"),
        &Files::default(),
    );

    // Assert
    assert_eq!(status, AutostartStatus::Missing(registered));
}

#[test]
fn a_program_that_cannot_be_checked_without_waiting_is_another_command() {
    // Arrange
    let share = path("share-mosaico");
    let files = Files {
        far: vec![share.clone()],
        ..Files::default()
    };
    let registered = LaunchCommand::autostart(share);

    // Act
    let status = AutostartStatus::of(Some(registered.clone()), &path("mosaico"), &files);

    // Assert
    assert_eq!(status, other(registered, &path("mosaico")));
}

#[test]
fn a_relative_program_is_another_command_and_the_disk_is_not_asked() {
    // Arrange
    let registered = LaunchCommand::autostart("mosaico");

    // Act
    let status = AutostartStatus::of(Some(registered.clone()), &path("mosaico"), &Untouched);

    // Assert
    assert_eq!(status, other(registered, &path("mosaico")));
}

#[test]
fn an_entry_that_names_no_program_is_empty() {
    // Arrange
    let registered = LaunchCommand {
        program: PathBuf::new(),
        args: vec!["start".into()],
    };

    // Act
    let status = AutostartStatus::of(Some(registered), &path("mosaico"), &Untouched);

    // Assert
    assert_eq!(status, AutostartStatus::Empty);
}

/// A registration for the tests, turned off when `turned_off` is set.
struct Registered {
    command: Option<LaunchCommand>,
    turned_off: bool,
}

impl Autostart for Registered {
    fn registered_command(&self) -> Result<Option<LaunchCommand>> {
        Ok(self.command.clone())
    }

    fn is_turned_off(&self) -> Result<bool> {
        Ok(self.turned_off)
    }

    fn enable(&self, _: &LaunchCommand) -> Result<()> {
        unreachable!("reading does not enable")
    }

    fn disable(&self) -> Result<()> {
        unreachable!("reading does not disable")
    }
}

/// An executable lookup that must not be asked.
fn no_exe() -> Result<PathBuf> {
    panic!("asked for the running executable");
}

#[test]
fn reading_nothing_registered_needs_no_executable() {
    // Arrange
    let autostart = Registered {
        command: None,
        turned_off: false,
    };

    // Act
    let status = AutostartStatus::read(&autostart, &Untouched, no_exe).unwrap();

    // Assert
    assert_eq!(status, AutostartStatus::Disabled);
}

#[test]
fn an_entry_the_user_turned_off_is_turned_off_whatever_it_starts() {
    // Arrange
    let autostart = Registered {
        command: Some(LaunchCommand::autostart(path("another-mosaico"))),
        turned_off: true,
    };

    // Act
    let status = AutostartStatus::read(&autostart, &Untouched, no_exe).unwrap();

    // Assert
    assert_eq!(status, AutostartStatus::TurnedOff);
}

#[test]
fn an_entry_that_is_on_is_compared_with_the_running_executable() {
    // Arrange
    let exe = path("mosaico");
    let autostart = Registered {
        command: Some(LaunchCommand::autostart(&exe)),
        turned_off: false,
    };

    // Act
    let status = AutostartStatus::read(&autostart, &Untouched, || Ok(exe.clone())).unwrap();

    // Assert
    assert_eq!(status, AutostartStatus::Enabled);
}
