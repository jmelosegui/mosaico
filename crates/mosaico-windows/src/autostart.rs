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

use mosaico_core::Result;
use mosaico_core::system::{Autostart, LaunchCommand};

/// [`Autostart`] through the per-user Run registry key.
pub struct RegistryAutostart;

impl Autostart for RegistryAutostart {
    /// Checks whether the Mosaico value exists in the Run key.
    fn is_enabled(&self) -> bool {
        registry::exists()
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
