//! Command lines as the Run key holds them.
//!
//! A Run value is one string. Windows starts the program named at its
//! front and passes the rest on, and the program splits that into
//! arguments with the rules every Windows program follows (those of
//! `CommandLineToArgvW`). So the program goes in quotes, and an argument
//! is quoted and escaped for those rules when it needs to be.

use std::iter::repeat_n;

use mosaico_core::system::LaunchCommand;

/// The command line that runs `command`: the program in quotes, then
/// each argument.
pub fn join(command: &LaunchCommand) -> String {
    let mut line = format!("\"{}\"", command.program.display());
    for arg in &command.args {
        line.push(' ');
        push_arg(&mut line, arg);
    }
    line
}

/// Appends `arg` as it is, or in quotes when it is empty or has spaces,
/// tabs or quotes, escaping its quotes and the backslashes before them
/// so the program reads it back unchanged.
fn push_arg(line: &mut String, arg: &str) {
    if !arg.is_empty() && !arg.contains([' ', '\t', '"']) {
        line.push_str(arg);
        return;
    }
    line.push('"');
    let mut backslashes = 0;
    for c in arg.chars() {
        if c == '\\' {
            backslashes += 1;
            continue;
        }
        let escapes = if c == '"' {
            backslashes * 2 + 1
        } else {
            backslashes
        };
        line.extend(repeat_n('\\', escapes));
        line.push(c);
        backslashes = 0;
    }
    // Backslashes before the closing quote would escape it.
    line.extend(repeat_n('\\', backslashes * 2));
    line.push('"');
}

#[cfg(test)]
#[path = "autostart_command_line_tests.rs"]
mod tests;
