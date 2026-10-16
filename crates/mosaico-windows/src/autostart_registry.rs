//! Reading and writing values under keys of the current user.

#[path = "autostart_registry_write.rs"]
mod write;

use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_MORE_DATA, ERROR_SUCCESS};
use windows::Win32::System::Registry::{
    HKEY_CURRENT_USER, REG_ROUTINE_FLAGS, RRF_RT_REG_BINARY, RRF_RT_REG_SZ, RegGetValueW,
};
use windows::core::PCWSTR;

use mosaico_core::Result;

use crate::error::OrPlatform;
use crate::monitor_edid::wide_to_string;

/// A value under a key of `HKEY_CURRENT_USER`. Writing and deleting it
/// are in `autostart_registry_write.rs`.
pub struct Value {
    /// The path of the key.
    pub key: &'static str,
    /// The name of the value.
    pub name: &'static str,
}

impl Value {
    /// Reads the value as a string, or `None` when it, or its key, does
    /// not exist. A REG_EXPAND_SZ value has its variables expanded, as
    /// logon does, and a value that is not a string fails.
    pub fn read_string(&self, operation: &'static str) -> Result<Option<String>> {
        let bytes = self.read(RRF_RT_REG_SZ, operation)?;
        Ok(bytes.map(|bytes| {
            let (pairs, _) = bytes.as_chunks::<2>();
            let wide: Vec<u16> = pairs.iter().map(|pair| u16::from_le_bytes(*pair)).collect();
            wide_to_string(&wide)
        }))
    }

    /// Reads the value as bytes, or `None` when it, or its key, does not
    /// exist. A value that is not binary fails.
    pub fn read_binary(&self, operation: &'static str) -> Result<Option<Vec<u8>>> {
        self.read(RRF_RT_REG_BINARY, operation)
    }

    /// Reads the value with RegGetValueW, which only returns the types in
    /// `kind` and null-terminates the strings it returns.
    fn read(&self, kind: REG_ROUTINE_FLAGS, operation: &'static str) -> Result<Option<Vec<u8>>> {
        let wide_key = to_wide(self.key);
        let wide_name = to_wide(self.name);
        let mut buffer = vec![0u8; 1024];
        loop {
            let mut size = buffer.len() as u32;
            // SAFETY: RegGetValueW is a standard Win32 registry API. Both
            // names are null-terminated and the buffer holds `size` bytes.
            let status = unsafe {
                RegGetValueW(
                    HKEY_CURRENT_USER,
                    PCWSTR(wide_key.as_ptr()),
                    PCWSTR(wide_name.as_ptr()),
                    kind,
                    None,
                    Some(buffer.as_mut_ptr().cast()),
                    Some(&mut size),
                )
            };
            match status {
                ERROR_SUCCESS => {
                    buffer.truncate(size as usize);
                    return Ok(Some(buffer));
                }
                ERROR_FILE_NOT_FOUND => return Ok(None),
                // `size` is what the value needs now. Grow at least
                // twofold, so this ends even if the value keeps changing.
                ERROR_MORE_DATA => {
                    let needed = size as usize;
                    buffer.resize(needed.max(buffer.len() * 2), 0);
                }
                _ => return Err(windows::core::Error::from(status)).or_platform(operation),
            }
        }
    }
}

/// Converts a `&str` to a null-terminated wide (UTF-16) string.
fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
