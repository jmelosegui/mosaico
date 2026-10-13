//! Windows sources for monitor identity: the EDID copy in the registry
//! and the strings the display configuration APIs return. Decoding the
//! EDID itself is done by `mosaico_core::EdidIdentity::from_edid`.

use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::System::Registry::{HKEY_LOCAL_MACHINE, RRF_RT_REG_BINARY, RegGetValueW};
use windows::core::HSTRING;

/// Reads the EDID Windows stored for a monitor.
///
/// `device_path` looks like
/// `\\?\DISPLAY#DEL41B6#5&2f3c5ad&0&UID4353#{e6f07b5f-...}`. The middle
/// two parts name the device's key under `Enum\DISPLAY`, where Windows
/// keeps a copy of the EDID it read from the monitor.
pub(crate) fn registry_edid(device_path: &str) -> Option<Vec<u8>> {
    let mut parts = device_path.split('#');
    let _prefix = parts.next()?;
    let model = parts.next()?;
    let instance = parts.next()?;
    let subkey = HSTRING::from(format!(
        r"SYSTEM\CurrentControlSet\Enum\DISPLAY\{model}\{instance}\Device Parameters"
    ));

    let mut edid = vec![0u8; 512];
    let mut size = edid.len() as u32;
    // SAFETY: Reads a binary value into a caller-owned buffer of `size`
    // bytes; the call writes back how many it filled.
    let status = unsafe {
        RegGetValueW(
            HKEY_LOCAL_MACHINE,
            &subkey,
            &HSTRING::from("EDID"),
            RRF_RT_REG_BINARY,
            None,
            Some(edid.as_mut_ptr().cast()),
            Some(&mut size),
        )
    };
    if status != ERROR_SUCCESS {
        return None;
    }
    edid.truncate(size as usize);
    Some(edid)
}

/// Short name for a `DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY` value, used as
/// the connector part of a monitor key.
pub(crate) fn output_technology_name(value: i32) -> String {
    match value {
        0 => "vga".into(),
        4 => "dvi".into(),
        5 => "hdmi".into(),
        6 => "lvds".into(),
        10 => "dp".into(),
        11 => "edp".into(),
        15 => "miracast".into(),
        16 | 17 => "indirect".into(),
        18 => "usb-dp".into(),
        i32::MIN => "internal".into(),
        other => format!("tech{other}"),
    }
}

/// Converts a NUL-terminated UTF-16 buffer into a `String`.
pub(crate) fn wide_to_string(buffer: &[u16]) -> String {
    let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..len])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_to_string_stops_at_nul() {
        let buffer: Vec<u16> = "abc\0zzz".encode_utf16().collect();
        assert_eq!(wide_to_string(&buffer), "abc");
    }

    #[test]
    fn output_technology_names_common_ports() {
        assert_eq!(output_technology_name(10), "dp");
        assert_eq!(output_technology_name(i32::MIN), "internal");
        assert_eq!(output_technology_name(99), "tech99");
    }
}
