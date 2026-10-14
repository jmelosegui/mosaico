//! Command lines as the Run key holds them.
//!
//! A Run value is one string. At logon Windows starts the program named
//! at its front, finding an unquoted one the way `CreateProcess` does,
//! and passes the rest to it as its arguments. So the program goes in
//! quotes, and the arguments follow the rules in `autostart_args.rs`.

#[path = "autostart_args.rs"]
mod args;

use std::path::{Path, PathBuf};

use mosaico_core::system::LaunchCommand;
use mosaico_core::{Error, Result};

/// The command line that runs `command`: the program in quotes, then
/// each argument.
///
/// Fails with [`Error::Unsupported`] for a command the value cannot hold
/// so that it reads back the same: an empty program, a program path that
/// is not valid Unicode or has a quote, which would end the quoted
/// program early, or a NUL character, which would end the value.
pub fn join(command: &LaunchCommand) -> Result<String> {
    let program = command.program.to_str().ok_or(Error::Unsupported(
        "an autostart program path that is not valid Unicode",
    ))?;
    if program.is_empty() {
        return Err(Error::Unsupported("an empty autostart program path"));
    }
    if program.contains('"') {
        return Err(Error::Unsupported("an autostart program path with a quote"));
    }
    if program.contains('\0') || command.args.iter().any(|arg| arg.contains('\0')) {
        return Err(Error::Unsupported(
            "an autostart command with a NUL character",
        ));
    }
    let mut line = format!("\"{program}\"");
    for arg in &command.args {
        line.push(' ');
        args::push(&mut line, arg);
    }
    Ok(line)
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
