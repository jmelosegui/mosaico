//! Whether the shell would find a program, checked before asking it to
//! start one.
//!
//! When the shell finds nothing by a name, it guesses a web address from
//! it and opens that in the browser, so a typo in a launch binding would
//! open a web search instead of failing. This looks the name up the way
//! the shell does, and the launcher only hands over names that are found.

use std::ffi::OsString;
use std::path::Path;

use windows::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, RegCloseKey, RegOpenKeyExW,
};
use windows::core::PCWSTR;

const APP_PATHS: &str = r"Software\Microsoft\Windows\CurrentVersion\App Paths";

/// Extensions tried for a name without one when `PATHEXT` is not set.
const DEFAULT_PATHEXT: &str = ".COM;.EXE;.BAT;.CMD";

/// Where the shell finds programs.
pub(super) struct Lookup<'a> {
    /// Reads an environment variable.
    pub var: &'a dyn Fn(&str) -> Option<OsString>,
    /// Whether a program is registered with the system under this file
    /// name, as `msedge.exe` is.
    pub registered: &'a dyn Fn(&str) -> bool,
    /// The folder a relative path is read from.
    pub home: Option<&'a Path>,
}

/// Whether the shell can start `command`: a URI, a path to something that
/// exists, or a name found on `PATH` or registered with the system.
pub(super) fn can_start(command: &str, lookup: &Lookup) -> bool {
    if command.is_empty() {
        return false;
    }
    if is_uri(command) {
        return true;
    }
    let path = Path::new(command);
    if command.contains(['\\', '/']) {
        return (path.is_absolute() && exists(path))
            || lookup.home.is_some_and(|home| exists(&home.join(path)));
    }
    let names = names_for(command, lookup);
    names.iter().any(|name| (lookup.registered)(name)) || on_path(&names, lookup)
}

/// Whether the name has a scheme, like `ms-settings:` or `https://...`.
/// A drive letter is not one.
fn is_uri(command: &str) -> bool {
    let Some(colon) = command.find(':') else {
        return false;
    };
    let scheme = &command[..colon];
    scheme.len() > 1
        && scheme.starts_with(|c: char| c.is_ascii_alphabetic())
        && scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
}

/// The file names the shell tries for `command`: the name itself when it
/// has an extension, or the name with each of `PATHEXT`'s.
fn names_for(command: &str, lookup: &Lookup) -> Vec<String> {
    if Path::new(command).extension().is_some() {
        return vec![command.to_string()];
    }
    let pathext = (lookup.var)("PATHEXT")
        .map(|ext| ext.to_string_lossy().into_owned())
        .filter(|ext| !ext.is_empty())
        .unwrap_or_else(|| DEFAULT_PATHEXT.to_string());
    pathext
        .split(';')
        .filter(|ext| !ext.is_empty())
        .map(|ext| format!("{command}{ext}"))
        .collect()
}

fn on_path(names: &[String], lookup: &Lookup) -> bool {
    let Some(path) = (lookup.var)("PATH") else {
        return false;
    };
    std::env::split_paths(&path)
        .filter(|dir| !dir.as_os_str().is_empty())
        .any(|dir| names.iter().any(|name| exists(&dir.join(name))))
}

/// Whether something is at `path`. Not following links, so the aliases
/// packaged apps put on `PATH`, which cannot be opened as files, count.
fn exists(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok()
}

/// Whether `name` has an App Paths key for the current user or the
/// machine.
pub(super) fn registered(name: &str) -> bool {
    let key: Vec<u16> = format!(r"{APP_PATHS}\{name}")
        .encode_utf16()
        .chain([0])
        .collect();
    [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE]
        .into_iter()
        .any(|root| {
            let mut opened = HKEY::default();
            // SAFETY: `key` is nul-terminated and outlives the call, and
            // `opened` receives the key only when this succeeds.
            let status =
                unsafe { RegOpenKeyExW(root, PCWSTR(key.as_ptr()), None, KEY_READ, &mut opened) };
            if status.is_err() {
                return false;
            }
            // SAFETY: `opened` was opened above and is closed once.
            let _ = unsafe { RegCloseKey(opened) };
            true
        })
}

#[cfg(test)]
#[path = "launcher_find_tests.rs"]
mod tests;
