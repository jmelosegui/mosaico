//! Starting applications the user named in a launch binding.

use std::ffi::OsString;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;

use mosaico_core::system::Launcher;
use mosaico_core::{Error, Result};
use windows::Win32::UI::Shell::{
    SEE_MASK_FLAG_NO_UI, SEE_MASK_NOASYNC, SHELLEXECUTEINFOW, ShellExecuteExW,
};
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
use windows::core::PCWSTR;

use crate::apartment::Apartment;

#[path = "launcher_find.rs"]
mod find;

use find::Lookup;

/// Starts applications through the shell, which finds them the way the
/// Run box does: on `PATH`, under App Paths (`msedge.exe`), by full path,
/// or as a URI with a registered handler (`ms-settings:`).
pub struct ShellLauncher {
    /// The folder programs start in, or `None` for the daemon's own.
    home: Option<PathBuf>,
}

impl ShellLauncher {
    /// A launcher that starts programs in the current user's profile
    /// folder.
    pub fn current() -> Self {
        Self {
            home: std::env::var_os("USERPROFILE")
                .filter(|home| !home.is_empty())
                .map(PathBuf::from),
        }
    }
}

impl Launcher for ShellLauncher {
    fn launch(&self, command: &str, args: &[String]) -> Result<()> {
        let var = |name: &str| std::env::var_os(name);
        let command = expand_vars(command, var);
        let lookup = Lookup {
            var: &var,
            registered: &find::registered,
            home: self.home.as_deref(),
        };
        if !find::can_start(&command, &lookup) {
            let reason = format!("no program, file or URI named \"{command}\" was found");
            return Err(Error::platform("start an application", reason));
        }
        let file = wide(&command);
        let args: Vec<String> = args.iter().map(|arg| expand_vars(arg, var)).collect();
        let parameters = wide(&quote_args(&args));
        let directory = self.home.as_ref().map(|home| {
            let mut wide: Vec<u16> = home.as_os_str().encode_wide().collect();
            wide.push(0);
            wide
        });

        let mut info = SHELLEXECUTEINFOW {
            cbSize: size_of::<SHELLEXECUTEINFOW>() as u32,
            // The call returns only once the program has been started, and
            // reports a failure instead of showing a message box for it.
            fMask: SEE_MASK_NOASYNC | SEE_MASK_FLAG_NO_UI,
            lpFile: PCWSTR(file.as_ptr()),
            lpParameters: PCWSTR(parameters.as_ptr()),
            lpDirectory: directory
                .as_ref()
                .map_or(PCWSTR::null(), |dir| PCWSTR(dir.as_ptr())),
            nShow: SW_SHOWNORMAL.0,
            ..Default::default()
        };
        // The shell may hand the program to a COM server, as it does for
        // URIs and packaged apps, which needs an apartment on this thread.
        let _apartment = Apartment::enter();
        // SAFETY: `info` is initialized with its size, and every string it
        // points to is nul-terminated and outlives the call.
        unsafe { ShellExecuteExW(&mut info) }
            .map_err(|reason| Error::platform("start an application", reason))
    }
}

/// Replaces `%NAME%` with the value of the environment variable `NAME`,
/// as the Windows shell does. A name that is not set is left as written,
/// percent signs included.
fn expand_vars(text: &str, var: impl Fn(&str) -> Option<OsString>) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find('%') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let value = after.find('%').and_then(|end| {
            let name = &after[..end];
            let value = if name.is_empty() { None } else { var(name) }?;
            Some((value, end))
        });
        match value {
            Some((value, end)) => {
                out.push_str(&value.to_string_lossy());
                rest = &after[end + 1..];
            }
            None => {
                out.push('%');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

/// Joins `args` into one command line that `CommandLineToArgvW`, and the
/// C runtime every program built with it, splits back into the same
/// arguments.
fn quote_args(args: &[String]) -> String {
    let mut line = String::new();
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            line.push(' ');
        }
        quote_into(&mut line, arg);
    }
    line
}

fn quote_into(line: &mut String, arg: &str) {
    let plain = !arg.is_empty() && !arg.contains([' ', '\t', '\n', '\u{b}', '"']);
    if plain {
        line.push_str(arg);
        return;
    }
    line.push('"');
    let mut backslashes = 0;
    for c in arg.chars() {
        match c {
            '\\' => backslashes += 1,
            '"' => {
                // Backslashes before a quote are halved when read, and the
                // quote itself needs one more to stay a quote.
                line.extend(std::iter::repeat_n('\\', backslashes * 2 + 1));
                line.push('"');
                backslashes = 0;
            }
            _ => {
                line.extend(std::iter::repeat_n('\\', backslashes));
                line.push(c);
                backslashes = 0;
            }
        }
    }
    // Backslashes before the closing quote are halved too.
    line.extend(std::iter::repeat_n('\\', backslashes * 2));
    line.push('"');
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain([0]).collect()
}

#[cfg(test)]
#[path = "launcher_tests.rs"]
mod tests;
