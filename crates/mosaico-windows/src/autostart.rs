//! Windows autostart registration via the HKCU Run registry key.
//!
//! Writes a `Mosaico` value under
//! `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`
//! holding the command line Windows runs on user logon. No elevation is
//! required since HKCU is per-user.

#[path = "autostart_command_line.rs"]
mod command_line;

use windows::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_SAM_FLAGS, REG_SZ, RegCloseKey,
    RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
};
use windows::core::PCWSTR;

use mosaico_core::Result;
use mosaico_core::system::{Autostart, LaunchCommand};

use crate::error::OrPlatform;

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const VALUE_NAME: &str = "Mosaico";

/// Opens the HKCU Run key with the given access rights.
fn open_run_key(access: REG_SAM_FLAGS) -> Result<HKEY> {
    let wide_key = to_wide(RUN_KEY);
    let mut key = HKEY::default();
    // SAFETY: RegOpenKeyExW is a standard Win32 registry API. We pass valid
    // pointers and close the key after use.
    let status = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(wide_key.as_ptr()),
            None,
            access,
            &mut key,
        )
    };
    status.ok().or_platform("open the Run registry key")?;
    Ok(key)
}

/// Writes a REG_SZ value under the opened key.
fn set_string_value(key: HKEY, value: &str) -> Result<()> {
    let wide_name = to_wide(VALUE_NAME);
    let wide_value = to_wide(value);
    // SAFETY: reinterpreting a &[u16] as &[u8] is safe; the layout is
    // contiguous and we compute the correct byte length.
    let bytes: &[u8] =
        unsafe { std::slice::from_raw_parts(wide_value.as_ptr().cast(), wide_value.len() * 2) };
    // SAFETY: RegSetValueExW is a standard Win32 registry API. We pass the
    // correct byte length for the wide-string value.
    let status =
        unsafe { RegSetValueExW(key, PCWSTR(wide_name.as_ptr()), None, REG_SZ, Some(bytes)) };
    status.ok().or_platform("write the autostart entry")
}

/// Deletes the Mosaico value from the opened key.
fn delete_value(key: HKEY) -> Result<()> {
    let wide_name = to_wide(VALUE_NAME);
    // SAFETY: RegDeleteValueW is a standard Win32 registry API.
    let status = unsafe { RegDeleteValueW(key, PCWSTR(wide_name.as_ptr())) };
    if status.is_err() {
        // ERROR_FILE_NOT_FOUND (2) means the value doesn't exist — that's fine.
        if status.0 as u32 == 2 {
            return Ok(());
        }
    }
    status.ok().or_platform("remove the autostart entry")
}

/// Checks if the Mosaico value exists under the opened key.
fn query_value_exists(key: HKEY) -> bool {
    let wide_name = to_wide(VALUE_NAME);
    // SAFETY: RegQueryValueExW with null data buffer just checks existence.
    let status =
        unsafe { RegQueryValueExW(key, PCWSTR(wide_name.as_ptr()), None, None, None, None) };
    status.is_ok()
}

/// Closes an open registry key handle.
fn close_key(key: HKEY) {
    // SAFETY: RegCloseKey is safe to call on any valid HKEY.
    let _ = unsafe { RegCloseKey(key) };
}

/// Converts a `&str` to a null-terminated wide (UTF-16) string.
fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// [`Autostart`] through the per-user Run registry key.
pub struct RegistryAutostart;

impl Autostart for RegistryAutostart {
    /// Checks whether the Mosaico value exists in the Run key.
    fn is_enabled(&self) -> bool {
        let Ok(key) = open_run_key(KEY_QUERY_VALUE) else {
            return false;
        };
        let exists = query_value_exists(key);
        close_key(key);
        exists
    }

    /// Writes the command line that runs `command` to the Mosaico value,
    /// replacing what was there.
    fn enable(&self, command: &LaunchCommand) -> Result<()> {
        let key = open_run_key(KEY_SET_VALUE)?;
        let result = set_string_value(key, &command_line::join(command));
        close_key(key);
        result
    }

    /// Deletes the Mosaico value. Succeeds if it did not exist.
    fn disable(&self) -> Result<()> {
        let key = open_run_key(KEY_SET_VALUE)?;
        let result = delete_value(key);
        close_key(key);
        result
    }
}
