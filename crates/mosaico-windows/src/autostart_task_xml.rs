//! The definition of the logon task, in the Task Scheduler's XML.
//!
//! The task starts at the user's logon, as that user and without
//! elevation, so registering it needs no administrator. Unlike a Run
//! entry it does not wait for Explorer's startup delay, nor in line
//! behind the other startup programs. Its priority is 4, which the Task
//! Scheduler gives interactive programs: its default, 7, is for
//! background work and runs the program below normal priority.

use super::command_line::Action;

/// The XML of a task that runs `action` when the user with the security
/// identifier `user` logs on.
///
/// The trigger names the user, since only an administrator may register
/// a task that starts at anyone's logon. The task may run on battery,
/// has no time limit, and a second start while it runs is dropped.
pub fn definition(user: &str, action: &Action) -> String {
    let user = escape(user);
    let program = escape(&action.program);
    let arguments = escape(&action.arguments);
    format!(
        r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo>
    <Description>Starts mosaico when you sign in. Set up by `mosaico autostart enable`.</Description>
  </RegistrationInfo>
  <Triggers>
    <LogonTrigger>
      <Enabled>true</Enabled>
      <UserId>{user}</UserId>
    </LogonTrigger>
  </Triggers>
  <Principals>
    <Principal id="Author">
      <UserId>{user}</UserId>
      <LogonType>InteractiveToken</LogonType>
      <RunLevel>LeastPrivilege</RunLevel>
    </Principal>
  </Principals>
  <Settings>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
    <ExecutionTimeLimit>PT0S</ExecutionTimeLimit>
    <Priority>4</Priority>
    <Enabled>true</Enabled>
  </Settings>
  <Actions Context="Author">
    <Exec>
      <Command>{program}</Command>
      <Arguments>{arguments}</Arguments>
    </Exec>
  </Actions>
</Task>
"#
    )
}

/// `text` with the characters XML gives a meaning written as entities.
fn escape(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            c => escaped.push(c),
        }
    }
    escaped
}

#[cfg(test)]
#[path = "autostart_task_xml_tests.rs"]
mod tests;
