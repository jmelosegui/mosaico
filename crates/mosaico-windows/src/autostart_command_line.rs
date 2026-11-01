//! Command lines as a logon task and the Run key hold them.
//!
//! A logon task's action holds the program and its arguments apart, the
//! arguments as one line. A Run value, which older mosaico registered
//! with, is one string: at logon Windows starts the program named at its
//! front, finding an unquoted one the way `CreateProcess` does, and
//! passes the rest to it as its arguments. Either way the arguments
//! follow the rules in `autostart_args.rs`.

#[path = "autostart_args.rs"]
mod args;

use std::path::{Path, PathBuf};

use mosaico_core::system::LaunchCommand;
use mosaico_core::{Error, Result};

/// What a logon task's action runs: a program, and its arguments as one
/// line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action {
    /// The path of the program, without quotes.
    pub program: String,
    /// The arguments, quoted where they need to be.
    pub arguments: String,
}

/// The action that runs `command`.
///
/// Fails with [`Error::Unsupported`] for a command the action cannot
/// hold so that it runs and reads back the same: an empty program, a
/// program path that is not valid Unicode or has a quote, a control
/// character other than a tab, which the task's XML cannot hold or turns
/// into another, or a `%`, which the Task Scheduler takes for the start
/// of an environment variable it expands.
pub fn join(command: &LaunchCommand) -> Result<Action> {
    let program = command.program.to_str().ok_or(Error::Unsupported(
        "an autostart program path that is not valid Unicode",
    ))?;
    if program.is_empty() {
        return Err(Error::Unsupported("an empty autostart program path"));
    }
    if program.contains('"') {
        return Err(Error::Unsupported("an autostart program path with a quote"));
    }
    let texts = || std::iter::once(program).chain(command.args.iter().map(String::as_str));
    if texts().any(|text| text.contains(|c: char| c.is_control() && c != '\t')) {
        return Err(Error::Unsupported(
            "an autostart command with a control character",
        ));
    }
    if texts().any(|text| text.contains('%')) {
        return Err(Error::Unsupported(
            "an autostart command with a percent sign",
        ));
    }
    let mut arguments = String::new();
    for arg in &command.args {
        if !arguments.is_empty() {
            arguments.push(' ');
        }
        args::push(&mut arguments, arg);
    }
    Ok(Action {
        program: program.to_string(),
        arguments,
    })
}

/// The command a logon task's action runs. A program in quotes, as the
/// Task Scheduler accepts it, is read without them.
pub fn from_action(program: &str, arguments: &str) -> LaunchCommand {
    let program = program
        .strip_prefix('"')
        .and_then(|program| program.strip_suffix('"'))
        .unwrap_or(program);
    LaunchCommand {
        program: PathBuf::from(program),
        args: args::split(arguments),
    }
}

/// Splits a command line into the program and its arguments. The
/// program is empty when the line names none.
///
/// A program in quotes ends at the closing quote. An unquoted one is
/// found the way logon finds it, see `find_unquoted`. `is_file` says
/// whether a file is at a path, and must answer without waiting.
pub fn split(line: &str, is_file: impl Fn(&Path) -> bool) -> LaunchCommand {
    let line = line.trim_start_matches([' ', '\t']);
    let (program, rest) = match line.strip_prefix('"') {
        Some(quoted) => {
            let (program, rest) = quoted.split_once('"').unwrap_or((quoted, ""));
            (PathBuf::from(program), rest)
        }
        None => find_unquoted(line, &is_file),
    };
    LaunchCommand {
        program,
        args: args::split(rest),
    }
}

/// The program at the front of an unquoted command line, and the rest.
///
/// Logon, like `CreateProcess`, tries ever longer runs of words, adding
/// `.exe` to a name without an extension, and runs the first that names
/// a file. For `C:\Program Files\Logi\Bolt.exe --startup` that is
/// `C:\Program.exe`, then `C:\Program Files\Logi\Bolt.exe`. When none
/// does, the program is the first word, which logon cannot start either.
/// A relative name is not looked up, since the file it names depends on
/// where it is run from.
fn find_unquoted<'a>(line: &'a str, is_file: &dyn Fn(&Path) -> bool) -> (PathBuf, &'a str) {
    let end_of_line = line.trim_end_matches([' ', '\t']).len();
    let word_ends = line[..end_of_line]
        .match_indices([' ', '\t'])
        .map(|(at, _)| at)
        .filter(|&at| !line[..at].ends_with([' ', '\t']))
        .chain([end_of_line]);
    for end in word_ends {
        if let Some(program) = runnable(&line[..end], is_file) {
            return (program, &line[end..]);
        }
    }
    let first_end = line.find([' ', '\t']).unwrap_or(line.len());
    (PathBuf::from(&line[..first_end]), &line[first_end..])
}

/// The file logon runs for `name`, if there is one: `name` itself, or
/// `name` with `.exe` added when it has no extension.
fn runnable(name: &str, is_file: &dyn Fn(&Path) -> bool) -> Option<PathBuf> {
    let path = Path::new(name);
    if !path.is_absolute() {
        return None;
    }
    let path = match path.extension() {
        Some(_) => path.to_path_buf(),
        None => PathBuf::from(format!("{name}.exe")),
    };
    is_file(&path).then_some(path)
}

#[cfg(test)]
#[path = "autostart_split_tests.rs"]
mod split_tests;
#[cfg(test)]
#[path = "autostart_command_line_tests.rs"]
mod tests;
