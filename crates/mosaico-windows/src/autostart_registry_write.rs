//! Writing and deleting values under keys of the current user.

use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, WIN32_ERROR};
use windows::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_SET_VALUE, REG_OPTION_NON_VOLATILE, REG_SZ, RegCloseKey,
    RegCreateKeyExW, RegDeleteKeyValueW, RegSetValueExW,
};
use windows::core::PCWSTR;

use mosaico_core::Result;

use super::{Value, to_wide};
use crate::error::OrPlatform;

impl Value {
    /// Sets the value to the string `value`, replacing what was there and
    /// creating the key when it is missing.
    pub fn write_string(&self, value: &str, operation: &'static str) -> Result<()> {
        let key = self.create_key(operation)?;
        let result = set_string_value(key, self.name, value, operation);
        close_key(key);
        result
    }

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

    /// Opens the key for writing, creating it when it is missing.
    fn create_key(&self, operation: &'static str) -> Result<HKEY> {
        let wide_key = to_wide(self.key);
        let mut key = HKEY::default();
        // SAFETY: RegCreateKeyExW is a standard Win32 registry API. We pass
        // a null-terminated name and valid pointers, and close the key
        // after use.
        let status = unsafe {
            RegCreateKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(wide_key.as_ptr()),
                None,
                PCWSTR::null(),
                REG_OPTION_NON_VOLATILE,
                KEY_SET_VALUE,
                None,
                &mut key,
                None,
            )
        };
        status.ok().or_platform(operation)?;
        Ok(key)
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

/// Writes the REG_SZ value `name` under the opened key.
fn set_string_value(key: HKEY, name: &str, value: &str, operation: &'static str) -> Result<()> {
    let wide_name = to_wide(name);
    let wide_value = to_wide(value);
    // SAFETY: reinterpreting a &[u16] as &[u8] is safe; the layout is
    // contiguous and we compute the correct byte length.
    let bytes: &[u8] =
        unsafe { std::slice::from_raw_parts(wide_value.as_ptr().cast(), wide_value.len() * 2) };
    // SAFETY: RegSetValueExW is a standard Win32 registry API. We pass the
    // correct byte length for the wide-string value.
    let status =
        unsafe { RegSetValueExW(key, PCWSTR(wide_name.as_ptr()), None, REG_SZ, Some(bytes)) };
    status.ok().or_platform(operation)
}

/// Closes an open registry key handle.
fn close_key(key: HKEY) {
    // SAFETY: RegCloseKey is safe to call on any valid HKEY.
    let _ = unsafe { RegCloseKey(key) };
}

#[cfg(test)]
#[path = "autostart_registry_tests.rs"]
mod tests;
