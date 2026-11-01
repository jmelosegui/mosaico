//! Windows autostart through a logon task in the Task Scheduler.
//!
//! `enable` registers a task that starts mosaico when the user logs on,
//! as that user, without elevation and at normal priority. Programs in
//! the Run registry key start later: Explorer waits before it runs them,
//! then starts them one at a time.
//!
//! The user can disable the task in the Task Scheduler, which turns
//! autostart off without removing it.
//!
//! Older mosaico wrote a `Mosaico` value under
//! `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`,
//! with a Startup apps switch of the same name under
//! `...\Explorer\StartupApproved\Run`. While there is no task, that value
//! is read as the registration, so status and doctor still see it, and
//! `enable` and `disable` remove it. Only mosaico's own values are read
//! or removed, never another program's.

#[path = "autostart_command_line.rs"]
mod command_line;
#[path = "autostart_registry.rs"]
mod registry;
#[path = "autostart_switch.rs"]
mod switch;
#[path = "autostart_task.rs"]
mod task;
#[path = "autostart_task_xml.rs"]
mod task_xml;
#[path = "autostart_user.rs"]
mod user;

use std::path::Path;

use mosaico_core::Result;
use mosaico_core::system::{Autostart, FileProbe, LaunchCommand};

use self::registry::Value;
use self::task::LogonTask;
use crate::files::WindowsFiles;

/// The command line older mosaico had logon run.
const RUN: Value = Value {
    key: r"Software\Microsoft\Windows\CurrentVersion\Run",
    name: "Mosaico",
};

/// The Startup apps switch of that command line.
const SWITCH: Value = Value {
    key: r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run",
    name: "Mosaico",
};

/// [`Autostart`] through a logon task of the current user.
pub struct LogonTaskAutostart;

impl Autostart for LogonTaskAutostart {
    /// Reads what the task runs. Without a task, reads the older Run
    /// value and splits it into the program and its arguments, finding
    /// an unquoted program the way logon does. That looks for files only
    /// when the program is not in quotes, and never on a network share.
    fn registered_command(&self) -> Result<Option<LaunchCommand>> {
        if let Some(task) = user_task()?.read()? {
            return Ok(Some(command_line::from_action(
                &task.program,
                &task.arguments,
            )));
        }
        let Some(line) = RUN.read_string("read the autostart entry")? else {
            return Ok(None);
        };
        let is_file = |path: &Path| WindowsFiles.is_file(path) == Some(true);
        Ok(Some(command_line::split(&line, is_file)))
    }

    /// Whether the task is disabled. Without a task, reads the Startup
    /// apps switch of the older Run value, and without that it is on.
    fn is_turned_off(&self) -> Result<bool> {
        if let Some(task) = user_task()?.read()? {
            return Ok(!task.enabled);
        }
        let setting = SWITCH.read_binary("read the startup apps setting")?;
        Ok(setting.is_some_and(|data| switch::is_off(&data)))
    }

    /// Registers the task that runs `command`, replacing the one there
    /// was, enabled. Then removes the older Run value and its switch, so
    /// mosaico is not started twice. A command the task cannot hold
    /// exactly is refused before anything is changed.
    fn enable(&self, command: &LaunchCommand) -> Result<()> {
        let action = command_line::join(command)?;
        let user = user::current_user()?;
        LogonTask::of(&user).register(&task_xml::definition(&user, &action))?;
        RUN.delete("remove the older autostart entry")?;
        SWITCH.delete("remove the older startup apps setting")
    }

    /// Deletes the task and the older Run value. Succeeds when they are
    /// not there. The Startup apps switch is left as it is.
    fn disable(&self) -> Result<()> {
        user_task()?.delete()?;
        RUN.delete("remove the older autostart entry")
    }
}

/// The logon task of the current user.
fn user_task() -> Result<LogonTask> {
    Ok(LogonTask::of(&user::current_user()?))
}
