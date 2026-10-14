//! Command lines as the Run key holds them.
//!
//! A Run value is one string. Windows starts the program named at its
//! front and passes the rest on, and the program splits that into
//! arguments with the rules every Windows program follows (those of
//! `CommandLineToArgvW`). So the program goes in quotes, an argument is
//! quoted and escaped for those rules when it needs to be, and a value is
//! read back with the same rules.

use std::iter::repeat_n;
use std::path::PathBuf;

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

/// Splits a command line into the program and its arguments, or `None`
/// when it names no program.
///
/// A program in quotes ends at the closing quote, any other at the first
/// space or tab. Logon is more forgiving with an unquoted path that has
/// spaces, such as `C:\Program Files\mosaico\mosaico.exe start` typed in
/// by hand: it may still find and run the program, but this splits it at
/// the first space, so such an entry reads as another command.
/// `autostart enable` writes it back in quotes.
pub fn split(line: &str) -> Option<LaunchCommand> {
    let line = line.trim_start_matches([' ', '\t']);
    let (program, rest) = match line.strip_prefix('"') {
        Some(quoted) => quoted.split_once('"').unwrap_or((quoted, "")),
        None => line.split_once([' ', '\t']).unwrap_or((line, "")),
    };
    if program.is_empty() {
        return None;
    }
    Some(LaunchCommand {
        program: PathBuf::from(program),
        args: split_args(rest),
    })
}

/// Splits what follows the program into arguments. Spaces and tabs
/// outside quotes separate them and quotes group them. Backslashes are
/// literal unless a quote follows: then each pair is one backslash and
/// an odd one makes the quote literal. `""` inside quotes is a quote.
fn split_args(rest: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut chars = rest.chars().peekable();
    loop {
        while chars.next_if(|c| matches!(c, ' ' | '\t')).is_some() {}
        if chars.peek().is_none() {
            return args;
        }
        let mut arg = String::new();
        let mut quoted = false;
        while let Some(c) = chars.next_if(|&c| quoted || !matches!(c, ' ' | '\t')) {
            match c {
                '\\' => {
                    let mut backslashes = 1;
                    while chars.next_if_eq(&'\\').is_some() {
                        backslashes += 1;
                    }
                    if chars.peek() == Some(&'"') {
                        arg.extend(repeat_n('\\', backslashes / 2));
                        if backslashes % 2 == 1 {
                            chars.next();
                            arg.push('"');
                        }
                    } else {
                        arg.extend(repeat_n('\\', backslashes));
                    }
                }
                '"' => {
                    if quoted && chars.next_if_eq(&'"').is_some() {
                        arg.push('"');
                    } else {
                        quoted = !quoted;
                    }
                }
                c => arg.push(c),
            }
        }
        args.push(arg);
    }
}

#[cfg(test)]
#[path = "autostart_command_line_tests.rs"]
mod tests;
