//! Deleting values under keys of the current user.

use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, WIN32_ERROR};
use windows::Win32::System::Registry::{HKEY_CURRENT_USER, RegDeleteKeyValueW};
use windows::core::PCWSTR;

use mosaico_core::Result;

use super::{Value, to_wide};
use crate::error::OrPlatform;

impl Value {
    /// Deletes the value. Succeeds when it, or its key, is not there.
    pub fn delete(&self, operation: &'static str) -> Result<()> {
        let wide_key = to_wide(self.key);
        let wide_name = to_wide(self.name);
        // SAFETY: RegDeleteKeyValueW is a standard Win32 registry API. Both
        // names are null-terminated.
        let status = unsafe {
            RegDeleteKeyValueW(
                HKEY_CURRENT_USER,
                PCWSTR(wide_key.as_ptr()),
                PCWSTR(wide_name.as_ptr()),
            )
        };
        deleted(status, operation)
    }
}

/// What the status of deleting a value means: a value, or key, that is
/// not there is as good as deleted.
fn deleted(status: WIN32_ERROR, operation: &'static str) -> Result<()> {
    if status == ERROR_FILE_NOT_FOUND {
        return Ok(());
    }
    status.ok().or_platform(operation)
}

#[cfg(test)]
#[path = "autostart_registry_tests.rs"]
mod tests;
