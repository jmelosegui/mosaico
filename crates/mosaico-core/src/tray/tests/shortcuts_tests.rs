use std::cell::RefCell;
use std::path::{Path, PathBuf};

use crate::system::{FileProbe, Opener, ProcessControl};
use crate::{Error, Result};

use super::*;

/// A world with `files` in it, that records what is started and opened.
#[derive(Default)]
struct World {
    files: Vec<PathBuf>,
    fails: bool,
    launched: RefCell<Vec<PathBuf>>,
    opened: RefCell<Vec<PathBuf>>,
}

impl World {
    fn outcome(&self) -> Result<()> {
        if self.fails {
            Err(Error::platform("open", "access denied"))
        } else {
            Ok(())
        }
    }
}

impl FileProbe for World {
    fn is_file(&self, path: &Path) -> Option<bool> {
        Some(self.files.iter().any(|f| f == path))
    }

    fn same_file(&self, a: &Path, b: &Path) -> bool {
        a == b
    }
}

impl ProcessControl for World {
    fn launch_daemon(&self) -> Result<u32> {
        unreachable!("the tray starts no daemon")
    }

    fn launch(&self, program: &Path) -> Result<u32> {
        self.launched.borrow_mut().push(program.to_path_buf());
        self.outcome().map(|()| 7)
    }

    fn is_alive(&self, _: u32) -> bool {
        false
    }

    fn kill(&self, _: u32) -> Result<()> {
        unreachable!("the tray kills nothing")
    }
}

impl Opener for World {
    fn open(&self, path: &Path) -> Result<()> {
        self.opened.borrow_mut().push(path.to_path_buf());
        self.outcome()
    }
}

fn paths() -> TrayPaths {
    TrayPaths {
        settings: PathBuf::from("bin/mosaico-settings"),
        config_dir: PathBuf::from("config"),
        log: PathBuf::from("logs/mosaico.log"),
    }
}

fn shortcuts<'a>(paths: &'a TrayPaths, world: &'a World) -> Shortcuts<'a> {
    Shortcuts {
        paths,
        files: world,
        processes: world,
        opener: world,
    }
}

#[test]
fn settings_start_the_installed_app() {
    // Arrange
    let paths = paths();
    let world = World {
        files: vec![paths.settings.clone()],
        ..World::default()
    };

    // Act
    let opened = shortcuts(&paths, &world).open_settings();

    // Assert
    assert_eq!(opened, Ok(()));
    assert_eq!(
        *world.launched.borrow(),
        std::slice::from_ref(&paths.settings)
    );
}

#[test]
fn settings_that_are_not_installed_say_how_to_install_them() {
    // Arrange
    let paths = paths();
    let world = World::default();

    // Act
    let notice = shortcuts(&paths, &world).open_settings().unwrap_err();

    // Assert
    assert_eq!(notice.title, "The settings app is not installed");
    assert!(notice.body.contains("mosaico update --force"));
    assert!(world.launched.borrow().is_empty());
}

#[test]
fn settings_that_cannot_start_give_the_reason() {
    // Arrange
    let paths = paths();
    let world = World {
        files: vec![paths.settings.clone()],
        fails: true,
        ..World::default()
    };

    // Act
    let notice = shortcuts(&paths, &world).open_settings().unwrap_err();

    // Assert
    assert_eq!(notice.title, "Could not open the settings");
    assert!(notice.body.contains("access denied"), "{}", notice.body);
}

#[test]
fn the_config_folder_and_an_existing_log_are_opened() {
    // Arrange
    let paths = paths();
    let world = World {
        files: vec![paths.log.clone()],
        ..World::default()
    };
    let shortcuts = shortcuts(&paths, &world);

    // Act
    let folder = shortcuts.open_config_folder();
    let log = shortcuts.open_log();

    // Assert
    assert_eq!((folder, log), (Ok(()), Ok(())));
    assert_eq!(
        *world.opened.borrow(),
        [paths.config_dir.clone(), paths.log.clone()]
    );
}

#[test]
fn a_missing_log_says_how_to_turn_logging_on() {
    // Arrange
    let paths = paths();
    let world = World::default();

    // Act
    let notice = shortcuts(&paths, &world).open_log().unwrap_err();

    // Assert
    assert_eq!(notice.title, "There is no log yet");
    assert!(notice.body.contains("[logging]"));
    assert!(world.opened.borrow().is_empty());
}

#[test]
fn a_folder_that_cannot_be_opened_gives_the_reason() {
    // Arrange
    let paths = paths();
    let world = World {
        fails: true,
        ..World::default()
    };

    // Act
    let notice = shortcuts(&paths, &world).open_config_folder().unwrap_err();

    // Assert
    assert_eq!(notice.title, "Could not open the config folder");
    assert!(notice.body.contains("access denied"));
}
