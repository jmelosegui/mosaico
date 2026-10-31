//! Opening folders and files with what the user chose for them.

use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use mosaico_core::system::Opener;
use mosaico_core::{Error, Result};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
use windows::core::{PCWSTR, w};

/// Opens folders in Explorer and files in their default application.
pub struct ShellOpener;

impl Opener for ShellOpener {
    fn open(&self, path: &Path) -> Result<()> {
        let wide: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
        // SAFETY: Both strings are nul-terminated and outlive the call.
        let result = unsafe {
            ShellExecuteW(
                None,
                w!("open"),
                PCWSTR(wide.as_ptr()),
                None,
                None,
                SW_SHOWNORMAL,
            )
        };
        // A value above 32 means it worked.
        if result.0 as isize > 32 {
            Ok(())
        } else {
            let reason = windows::core::Error::from_win32();
            Err(Error::platform("open a folder or file", reason))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_with_nothing_there_cannot_be_opened() {
        // Arrange
        let path = std::env::temp_dir().join("mosaico-opener-nothing-here.log");

        // Act
        let opened = ShellOpener.open(&path);

        // Assert
        assert!(
            matches!(
                opened,
                Err(Error::Platform {
                    operation: "open a folder or file",
                    ..
                })
            ),
            "{opened:?}"
        );
    }
}
