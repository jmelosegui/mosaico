//! Windows autostart registration via the HKCU Run registry key.
//!
//! Writes a `Mosaico` value under
//! `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`
//! holding the command line Windows runs on user logon. No elevation is
//! required since HKCU is per-user.
//!
//! The Startup apps switch in Settings and Task Manager is a value of the
//! same name under `...\Explorer\StartupApproved\Run`, and logon skips a
//! Run value whose switch is off. Only mosaico's own switch is read or
//! removed, never another program's.

#[path = "autostart_command_line.rs"]
mod command_line;
#[path = "autostart_registry.rs"]
mod registry;
#[path = "autostart_switch.rs"]
mod switch;

use std::path::Path;

use mosaico_core::Result;
use mosaico_core::system::{Autostart, FileProbe, LaunchCommand};

use self::registry::Value;
use crate::files::WindowsFiles;

/// The command line logon runs for mosaico.
const RUN: Value = Value {
    key: r"Software\Microsoft\Windows\CurrentVersion\Run",
    name: "Mosaico",
};

/// The Startup apps switch of that command line.
const SWITCH: Value = Value {
    key: r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run",
    name: "Mosaico",
};

/// [`Autostart`] through the per-user Run registry key.
pub struct RegistryAutostart;

impl Autostart for RegistryAutostart {
    /// Reads the Mosaico value and splits it into the program and its
    /// arguments, finding an unquoted program the way logon does. That
    /// looks for files only when the program is not in quotes, as mosaico
    /// never writes it, and never on a network share.
    fn registered_command(&self) -> Result<Option<LaunchCommand>> {
        let Some(line) = RUN.read_string("read the autostart entry")? else {
            return Ok(None);
        };
        let is_file = |path: &Path| WindowsFiles.is_file(path) == Some(true);
        Ok(Some(command_line::split(&line, is_file)))
    }

    /// Reads mosaico's Startup apps switch. Without one the entry is on.
    fn is_turned_off(&self) -> Result<bool> {
        let setting = SWITCH.read_binary("read the startup apps setting")?;
        Ok(setting.is_some_and(|data| switch::is_off(&data)))
    }

    /// Writes the command line that runs `command` to the Mosaico value,
    /// replacing what was there, then removes mosaico's own Startup apps
    /// switch, which turns the entry on. A command the value cannot hold
    /// exactly is refused before anything is written.
    fn enable(&self, command: &LaunchCommand) -> Result<()> {
        let line = command_line::join(command)?;
        RUN.write_string(&line, "write the autostart entry")?;
        SWITCH.delete("turn the autostart entry on")
    }

    /// Deletes the Mosaico value. Succeeds if it did not exist. The
    /// Startup apps switch is left as it is.
    fn disable(&self) -> Result<()> {
        RUN.delete("remove the autostart entry")
    }
}
