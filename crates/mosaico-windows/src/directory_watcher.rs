//! Change notifications on a directory.
//!
//! `FindFirstChangeNotificationW` returns a handle that is signaled when
//! a file directly in the directory is created, deleted, renamed or
//! written. It stays signaled until `FindNextChangeNotification` re-arms
//! it; a change made before the re-arm is kept and signals the handle
//! again at once, so none is lost. One save often signals it more than
//! once, and a write only counts once the file system has recorded it,
//! which can lag a little behind the write.
//!
//! When the directory is deleted, the handle is signaled once more, then
//! re-arming it fails with access denied. A directory that is renamed
//! stays watched where it went, and a new directory at the old path is
//! not watched.

use std::path::Path;
use std::time::Duration;

use windows::Win32::Foundation::{HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT};
use windows::Win32::Storage::FileSystem::{
    FILE_NOTIFY_CHANGE_FILE_NAME, FILE_NOTIFY_CHANGE_LAST_WRITE, FindCloseChangeNotification,
    FindFirstChangeNotificationW, FindNextChangeNotification,
};
use windows::Win32::System::Threading::{INFINITE, WaitForSingleObject};
use windows::core::HSTRING;

use mosaico_core::Result;
use mosaico_core::watch::{DirectoryWake, DirectoryWatcher};

use crate::error::OrPlatform;

/// Watches one directory through a change notification handle.
#[derive(Debug)]
pub struct WindowsDirectoryWatcher {
    handle: HANDLE,
    /// The last wait reported a change, so the handle is still signaled
    /// and has to be re-armed before the next one.
    signaled: bool,
}

impl WindowsDirectoryWatcher {
    /// Starts watching the files directly in `dir`.
    ///
    /// # Errors
    ///
    /// Fails when `dir` does not exist or cannot be watched.
    pub fn new(dir: &Path) -> Result<Self> {
        let path = HSTRING::from(dir.as_os_str());
        let flags = FILE_NOTIFY_CHANGE_LAST_WRITE | FILE_NOTIFY_CHANGE_FILE_NAME;

        // SAFETY: FindFirstChangeNotificationW only reads the path, which
        // the HSTRING keeps alive for the call.
        let handle = unsafe { FindFirstChangeNotificationW(&path, false, flags) }
            .or_platform("watch the directory")?;
        Ok(Self {
            handle,
            signaled: false,
        })
    }
}

impl DirectoryWatcher for WindowsDirectoryWatcher {
    /// Re-arms the handle after a reported change, then waits on it. The
    /// re-arm comes right before the wait, after the caller has looked at
    /// the files, so it has seen every change the handle reported.
    fn wait(&mut self, timeout: Duration) -> Result<DirectoryWake> {
        if std::mem::take(&mut self.signaled) {
            // SAFETY: the handle came from FindFirstChangeNotificationW and
            // stays open until drop.
            unsafe { FindNextChangeNotification(self.handle) }
                .or_platform("keep watching the directory")?;
        }

        // INFINITE would never time out, so stop one short of it.
        let millis =
            u32::try_from(timeout.as_millis()).map_or(INFINITE - 1, |ms| ms.min(INFINITE - 1));

        // SAFETY: the handle came from FindFirstChangeNotificationW and
        // stays open until drop.
        let result = unsafe { WaitForSingleObject(self.handle, millis) };
        if result == WAIT_OBJECT_0 {
            self.signaled = true;
            Ok(DirectoryWake::Changed)
        } else if result == WAIT_TIMEOUT {
            Ok(DirectoryWake::TimedOut)
        } else {
            Err(windows::core::Error::from_win32()).or_platform("wait for changes in the directory")
        }
    }
}

impl Drop for WindowsDirectoryWatcher {
    fn drop(&mut self) {
        // SAFETY: the handle came from FindFirstChangeNotificationW and is
        // closed only here.
        let _ = unsafe { FindCloseChangeNotification(self.handle) };
    }
}

#[cfg(test)]
#[path = "directory_watcher_tests.rs"]
mod tests;
