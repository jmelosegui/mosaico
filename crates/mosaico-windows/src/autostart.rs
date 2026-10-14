//! Windows autostart registration via the HKCU Run registry key.
//!
//! Writes a `Mosaico` value under
//! `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`
//! holding the command line Windows runs on user logon. No elevation is
//! required since HKCU is per-user.

#[path = "autostart_command_line.rs"]
mod command_line;
#[path = "autostart_registry.rs"]
mod registry;

use std::path::Path;

use mosaico_core::system::{Autostart, FileProbe, LaunchCommand};
use mosaico_core::{Error, Result};

use crate::files::WindowsFiles;

/// [`Autostart`] through the per-user Run registry key.
pub struct RegistryAutostart;

impl Autostart for RegistryAutostart {
    /// Reads the Mosaico value and splits it into the program and its
    /// arguments, finding an unquoted program the way logon does. That
    /// looks for files only when the program is not in quotes, as mosaico
    /// never writes it, and never on a network share.
    fn registered_command(&self) -> Result<Option<LaunchCommand>> {
        let Some(line) = registry::read()? else {
            return Ok(None);
        };
        let is_file = |path: &Path| WindowsFiles.is_file(path) == Some(true);
        command_line::split(&line, is_file)
            .map(Some)
            .ok_or_else(|| {
                Error::platform("read the autostart entry", "the entry names no program")
            })
    }

    /// Writes the command line that runs `command` to the Mosaico value,
    /// replacing what was there. A command the value cannot hold exactly
    /// is refused before anything is written.
    fn enable(&self, command: &LaunchCommand) -> Result<()> {
        registry::write(&command_line::join(command)?)
    }

    /// Deletes the Mosaico value. Succeeds if it did not exist.
    fn disable(&self) -> Result<()> {
        registry::delete()
    }
}
