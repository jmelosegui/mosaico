//! Reading and writing the `Mosaico` value under the per-user Run key.

use windows::Win32::Foundation::{
    ERROR_FILE_NOT_FOUND, ERROR_MORE_DATA, ERROR_SUCCESS, WIN32_ERROR,
};
use windows::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_SET_VALUE, REG_OPTION_NON_VOLATILE, REG_SZ, RRF_RT_REG_SZ,
    RegCloseKey, RegCreateKeyExW, RegDeleteKeyValueW, RegGetValueW, RegSetValueExW,
};
use windows::core::PCWSTR;

use mosaico_core::Result;

use crate::error::OrPlatform;
use crate::monitor_edid::wide_to_string;

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const VALUE_NAME: &str = "Mosaico";

/// Reads the Mosaico value as a string, or `None` when it, or the Run
/// key, does not exist.
///
/// RRF_RT_REG_SZ makes RegGetValueW expand the variables of a
/// REG_EXPAND_SZ value, as logon does, and fail for values that are not
/// strings. It always null-terminates what it returns.
pub fn read() -> Result<Option<String>> {
    let wide_key = to_wide(RUN_KEY);
    let wide_name = to_wide(VALUE_NAME);
    let mut buffer = vec![0u16; 512];
    loop {
        let mut size = (buffer.len() * 2) as u32;
        // SAFETY: RegGetValueW is a standard Win32 registry API. Both names
        // are null-terminated and the buffer holds `size` bytes.
        let status = unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                PCWSTR(wide_key.as_ptr()),
                PCWSTR(wide_name.as_ptr()),
                RRF_RT_REG_SZ,
                None,
                Some(buffer.as_mut_ptr().cast()),
                Some(&mut size),
            )
        };
        match status {
            ERROR_SUCCESS => return Ok(Some(wide_to_string(&buffer))),
            ERROR_FILE_NOT_FOUND => return Ok(None),
            // `size` is what the value needs now. Grow at least twofold,
            // so this ends even if the value changes between calls.
            ERROR_MORE_DATA => {
                let needed = (size as usize).div_ceil(2);
                buffer.resize(needed.max(buffer.len() * 2), 0);
            }
            _ => {
                return Err(windows::core::Error::from(status))
                    .or_platform("read the autostart entry");
            }
        }
    }
}

/// Sets the Mosaico value to `value`, replacing what was there. Creates
/// the Run key when the profile has none.
pub fn write(value: &str) -> Result<()> {
    let key = create_run_key()?;
    let result = set_string_value(key, value);
    close_key(key);
    result
}

/// Deletes the Mosaico value. Succeeds when it, or the Run key, is not
/// there.
pub fn delete() -> Result<()> {
    let wide_key = to_wide(RUN_KEY);
    let wide_name = to_wide(VALUE_NAME);
    // SAFETY: RegDeleteKeyValueW is a standard Win32 registry API. Both
    // names are null-terminated.
    let status = unsafe {
        RegDeleteKeyValueW(
            HKEY_CURRENT_USER,
            PCWSTR(wide_key.as_ptr()),
            PCWSTR(wide_name.as_ptr()),
        )
    };
    deleted(status, "remove the autostart entry")
}

/// What the status of deleting a value means: a value, or key, that is
/// not there is as good as deleted.
fn deleted(status: WIN32_ERROR, operation: &'static str) -> Result<()> {
    if status == ERROR_FILE_NOT_FOUND {
        return Ok(());
    }
    status.ok().or_platform(operation)
}

/// Opens the HKCU Run key for writing, creating it if it is missing.
fn create_run_key() -> Result<HKEY> {
    let wide_key = to_wide(RUN_KEY);
    let mut key = HKEY::default();
    // SAFETY: RegCreateKeyExW is a standard Win32 registry API. We pass a
    // null-terminated name and valid pointers, and close the key after
    // use.
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

/// Closes an open registry key handle.
fn close_key(key: HKEY) {
    // SAFETY: RegCloseKey is safe to call on any valid HKEY.
    let _ = unsafe { RegCloseKey(key) };
}

/// Converts a `&str` to a null-terminated wide (UTF-16) string.
fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(test)]
#[path = "autostart_registry_tests.rs"]
mod tests;
