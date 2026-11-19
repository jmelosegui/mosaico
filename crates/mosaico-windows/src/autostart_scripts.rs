//! Scripts started at logon, each through a logon task of its own named
//! after the script and the user, beside mosaico's own task.

use mosaico_core::Result;
use mosaico_core::system::{LaunchCommand, ScriptAutostart};

use super::task::LogonTask;
use super::{command_line, task_xml, user};

/// [`ScriptAutostart`] through logon tasks of the current user.
pub struct LogonTaskScripts;

impl ScriptAutostart for LogonTaskScripts {
    fn is_enabled(&self, name: &str) -> Result<bool> {
        Ok(script_task(name)?.read()?.is_some())
    }

    /// Registers the script's task, replacing the one there was. A
    /// command the task cannot hold exactly is refused before anything
    /// is changed.
    fn enable(&self, name: &str, command: &LaunchCommand) -> Result<()> {
        let action = command_line::join(command)?;
        let user = user::current_user()?;
        let description = format!(
            "Starts the mosaico script {name} when you sign in. \
             Set up by `mosaico script autostart enable {name}`."
        );
        LogonTask::named(&task_name(name, &user)).register(&task_xml::described(
            &user,
            &action,
            &description,
        ))
    }

    fn disable(&self, name: &str) -> Result<()> {
        script_task(name)?.delete()
    }
}

/// The logon task of the script called `name` for the current user.
fn script_task(name: &str) -> Result<LogonTask> {
    Ok(LogonTask::named(&task_name(name, &user::current_user()?)))
}

/// What the task of the script called `name` is called for the user with
/// the security identifier `user`.
fn task_name(name: &str, user: &str) -> String {
    format!("Mosaico script {name} for {user}")
}
