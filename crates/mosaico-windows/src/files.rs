//! [`FileProbe`](mosaico_core::system::FileProbe) on Windows.
//!
//! Looking at a path on a network share blocks until the server answers
//! or the request times out, which can take tens of seconds when it is
//! out of reach. So paths on a share (`\\server\share\...`, also in its
//! `\\?\UNC\` form) and on drives mapped to one are not looked at.

use std::path::{Component, Path, Prefix};

use windows::Win32::Storage::FileSystem::GetDriveTypeW;
use windows::core::PCWSTR;

use mosaico_core::system::FileProbe;

/// What `GetDriveTypeW` returns for a drive mapped to a network share
/// (`DRIVE_REMOTE`).
const DRIVE_REMOTE: u32 = 4;

/// [`FileProbe`] that answers from local drives only.
pub struct WindowsFiles;

impl FileProbe for WindowsFiles {
    fn is_file(&self, path: &Path) -> Option<bool> {
        is_local(path).then(|| path.is_file())
    }

    /// Compares the final paths both resolve to, which follows symbolic
    /// links and junctions and settles letter case, short `8.3` names and
    /// the `\\?\` form. Hard links resolve to different paths.
    fn same_file(&self, a: &Path, b: &Path) -> bool {
        if !is_local(a) || !is_local(b) {
            return false;
        }
        match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
    }
}

/// Where a path is, as far as its prefix tells.
#[derive(Debug, PartialEq, Eq)]
enum Location {
    /// On the drive with this letter, which may be mapped to a share.
    Drive(u8),
    /// On a network share.
    Share,
    /// A device, such as `\\.\pipe\...`.
    Device,
    /// On a volume named some other way, or relative to the current one.
    Local,
}

/// Where `path` is, from its prefix alone.
fn location(path: &Path) -> Location {
    let Some(Component::Prefix(prefix)) = path.components().next() else {
        return Location::Local;
    };
    match prefix.kind() {
        Prefix::Disk(letter) | Prefix::VerbatimDisk(letter) => {
            Location::Drive(letter.to_ascii_uppercase())
        }
        Prefix::UNC(..) | Prefix::VerbatimUNC(..) => Location::Share,
        Prefix::DeviceNS(_) => Location::Device,
        Prefix::Verbatim(_) => Location::Local,
    }
}

/// Whether looking at `path` stays on this machine.
fn is_local(path: &Path) -> bool {
    match location(path) {
        Location::Drive(letter) => !is_network_drive(letter),
        Location::Share | Location::Device => false,
        Location::Local => true,
    }
}

/// Whether the drive with this letter is mapped to a network share. This
/// asks the system how the drive is mounted, not the share itself.
fn is_network_drive(letter: u8) -> bool {
    let root = [letter, b':', b'\\', 0].map(u16::from);
    // SAFETY: GetDriveTypeW is a standard Win32 API; `root` is a
    // null-terminated wide string that outlives the call.
    let kind = unsafe { GetDriveTypeW(PCWSTR(root.as_ptr())) };
    kind == DRIVE_REMOTE
}

#[cfg(test)]
#[path = "files_tests.rs"]
mod tests;
