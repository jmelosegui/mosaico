//! Windows autostart registration via the HKCU Run registry key.
//!
//! Writes a `Mosaico` value under
//! `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`
//! so that `mosaico start` is executed on user logon. No elevation is
//! required since HKCU is per-user.

use windows::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_SAM_FLAGS, REG_SZ, RegCloseKey,
    RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
};
use windows::core::PCWSTR;

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const VALUE_NAME: &str = "Mosaico";

/// Registers Mosaico to start on Windows logon.
///
/// Writes `"<exe_path>" start` to the registry Run key.
pub fn enable() -> Result<(), String> {
    let value = exe_command()?;
    let key = open_run_key(KEY_SET_VALUE)?;
    let result = set_string_value(key, &value);
    close_key(key);
    result
}

/// Removes the Mosaico autostart entry from the registry.
///
/// Returns `Ok(())` if the value was removed or didn't exist.
pub fn disable() -> Result<(), String> {
    let key = open_run_key(KEY_SET_VALUE)?;
    let result = delete_value(key);
    close_key(key);
    result
}

/// Checks whether the Mosaico autostart entry exists in the registry.
pub fn is_enabled() -> bool {
    let Ok(key) = open_run_key(KEY_QUERY_VALUE) else {
        return false;
    };
    let exists = query_value_exists(key);
    close_key(key);
    exists
}

/// Returns `"<exe_path>" start` for the registry value.
fn exe_command() -> Result<String, String> {
    let exe = std::env::current_exe().map_err(|e| format!("could not resolve exe path: {e}"))?;
    Ok(format!("\"{}\" start", exe.display()))
}

/// Opens the HKCU Run key with the given access rights.
fn open_run_key(access: REG_SAM_FLAGS) -> Result<HKEY, String> {
    let wide_key: Vec<u16> = RUN_KEY.encode_utf16().chain(std::iter::once(0)).collect();
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
    if status.is_err() {
        return Err(format!("could not open registry key: {status:?}"));
    }
    Ok(key)
}

/// Writes a REG_SZ value under the opened key.
fn set_string_value(key: HKEY, value: &str) -> Result<(), String> {
    let wide_name: Vec<u16> = VALUE_NAME
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let wide_value: Vec<u16> = value.encode_utf16().chain(std::iter::once(0)).collect();
    // SAFETY: reinterpreting a &[u16] as &[u8] is safe; the layout is
    // contiguous and we compute the correct byte length.
    let bytes: &[u8] =
        unsafe { std::slice::from_raw_parts(wide_value.as_ptr().cast(), wide_value.len() * 2) };
    // SAFETY: RegSetValueExW is a standard Win32 registry API. We pass the
    // correct byte length for the wide-string value.
    let status =
        unsafe { RegSetValueExW(key, PCWSTR(wide_name.as_ptr()), None, REG_SZ, Some(bytes)) };
    if status.is_err() {
        return Err(format!("could not write registry value: {status:?}"));
    }
    Ok(())
}

/// Deletes the Mosaico value from the opened key.
fn delete_value(key: HKEY) -> Result<(), String> {
    let wide_name: Vec<u16> = VALUE_NAME
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    // SAFETY: RegDeleteValueW is a standard Win32 registry API.
    let status = unsafe { RegDeleteValueW(key, PCWSTR(wide_name.as_ptr())) };
    if status.is_err() {
        // ERROR_FILE_NOT_FOUND (2) means the value doesn't exist — that's fine.
        if status.0 as u32 == 2 {
            return Ok(());
        }
        return Err(format!("could not delete registry value: {status:?}"));
    }
    Ok(())
}

/// Checks if the Mosaico value exists under the opened key.
fn query_value_exists(key: HKEY) -> bool {
    let wide_name: Vec<u16> = VALUE_NAME
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
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
