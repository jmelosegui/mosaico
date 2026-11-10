//! Every running process, from a snapshot the system takes with Toolhelp.

use std::mem::size_of;
use std::ops::ControlFlow;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::path::Path;

use mosaico_core::system::RunningProcess;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};

/// Shows `visit` each process in a snapshot until it breaks, and returns
/// what it broke with: `None` when every process was seen or the snapshot
/// could not be taken.
pub(crate) fn walk<T>(mut visit: impl FnMut(&PROCESSENTRY32W) -> ControlFlow<T>) -> Option<T> {
    // SAFETY: the snapshot is a new handle that nothing else owns, so the
    // OwnedHandle closes it.
    let snapshot = unsafe {
        OwnedHandle::from_raw_handle(CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).ok()?.0)
    };
    let snapshot = HANDLE(snapshot.as_raw_handle());
    let mut entry = PROCESSENTRY32W {
        dwSize: size_of::<PROCESSENTRY32W>() as u32,
        ..PROCESSENTRY32W::default()
    };
    let mut step: Step = Process32FirstW;
    loop {
        // SAFETY: `entry` is a PROCESSENTRY32W with its size set, as both
        // steps require, and the snapshot is open.
        unsafe { step(snapshot, &mut entry) }.ok()?;
        if let ControlFlow::Break(found) = visit(&entry) {
            return Some(found);
        }
        step = Process32NextW;
    }
}

/// `Process32FirstW` or `Process32NextW`.
type Step = unsafe fn(HANDLE, *mut PROCESSENTRY32W) -> windows::core::Result<()>;

/// Every process the caller may see, empty when the snapshot fails.
pub(crate) fn processes() -> Vec<RunningProcess> {
    let mut found = Vec::new();
    walk::<()>(|entry| {
        found.push(RunningProcess {
            pid: entry.th32ProcessID,
            program: program_of(&entry.szExeFile),
        });
        ControlFlow::Continue(())
    });
    found
}

/// The program's file name in `exe_file`, a wide string ended by a NUL,
/// without its extension.
fn program_of(exe_file: &[u16]) -> String {
    let len = exe_file
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(exe_file.len());
    let name = String::from_utf16_lossy(&exe_file[..len]);
    match Path::new(&name).file_stem() {
        Some(stem) => stem.to_string_lossy().into_owned(),
        None => name,
    }
}

#[cfg(test)]
mod tests {
    use super::program_of;

    fn wide(text: &str) -> Vec<u16> {
        let mut wide: Vec<u16> = text.encode_utf16().collect();
        wide.resize(260, 0);
        wide
    }

    #[test]
    fn a_program_is_named_without_its_extension_and_the_padding() {
        let names = [
            wide("yasb.exe"),
            wide("Code - Insiders.exe"),
            wide("[System Process]"),
        ];

        let programs = names.map(|name| program_of(&name));

        assert_eq!(programs, ["yasb", "Code - Insiders", "[System Process]"]);
    }
}
