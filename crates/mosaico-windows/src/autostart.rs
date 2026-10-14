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

use mosaico_core::system::{Autostart, LaunchCommand};
use mosaico_core::{Error, Result};

/// [`Autostart`] through the per-user Run registry key.
pub struct RegistryAutostart;

impl Autostart for RegistryAutostart {
    /// Reads the Mosaico value and splits it into the program and its
    /// arguments, see [`command_line::split`].
    fn registered_command(&self) -> Result<Option<LaunchCommand>> {
        let Some(line) = registry::read()? else {
            return Ok(None);
        };
        command_line::split(&line).map(Some).ok_or_else(|| {
            Error::platform("read the autostart entry", "the entry names no program")
        })
    }

    /// Writes the command line that runs `command` to the Mosaico value,
    /// replacing what was there.
    fn enable(&self, command: &LaunchCommand) -> Result<()> {
        registry::write(&command_line::join(command))
    }

    /// Deletes the Mosaico value. Succeeds if it did not exist.
    fn disable(&self) -> Result<()> {
        registry::delete()
    }
}
