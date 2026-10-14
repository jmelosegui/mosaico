//! The autostart registration and the program files of the fake world.

use std::io;
use std::path::{Path, PathBuf};

use mosaico_core::Result;
use mosaico_core::system::{Autostart, FileProbe, LaunchCommand};

use super::{World, failure};

/// The mosaico executable commands run from in the fake world.
pub const EXE: &str = r"C:\apps\mosaico\mosaico.exe";

/// A mosaico executable that is no longer there, as after it moved.
pub const MOVED_AWAY: &str = r"C:\moved-away\mosaico.exe";

impl World {
    /// The executable commands run from: [`EXE`], or an error when
    /// [`World::exe_missing`] is set.
    pub fn current_exe(&self) -> io::Result<PathBuf> {
        if self.exe_missing.get() {
            return Err(io::Error::other("no executable"));
        }
        Ok(PathBuf::from(EXE))
    }

    /// Fails the way the registration does when an autostart error is set.
    fn autostart_fails(&self) -> Result<()> {
        match self.autostart_error.borrow().clone() {
            Some(message) => Err(failure(message)),
            None => Ok(()),
        }
    }
}

impl Autostart for World {
    fn registered_command(&self) -> Result<Option<LaunchCommand>> {
        self.autostart_fails()?;
        Ok(self.autostart.borrow().clone())
    }

    fn enable(&self, command: &LaunchCommand) -> Result<()> {
        self.autostart_fails()?;
        *self.autostart.borrow_mut() = Some(command.clone());
        Ok(())
    }

    fn disable(&self) -> Result<()> {
        self.autostart_fails()?;
        *self.autostart.borrow_mut() = None;
        Ok(())
    }
}

/// Only the files listed in [`World::files`] exist, and no two paths
/// name one file.
impl FileProbe for World {
    fn is_file(&self, path: &Path) -> Option<bool> {
        Some(self.files.borrow().iter().any(|file| file == path))
    }

    fn same_file(&self, _a: &Path, _b: &Path) -> bool {
        false
    }
}
