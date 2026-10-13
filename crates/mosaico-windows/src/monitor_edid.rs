//! Decoding helpers for monitor identity: EDID fields and the strings
//! the display configuration APIs return.

use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::System::Registry::{HKEY_LOCAL_MACHINE, RRF_RT_REG_BINARY, RegGetValueW};
use windows::core::HSTRING;

/// Decodes the three letter PNP ID packed into an EDID manufacturer field.
///
/// The display stack returns the two EDID bytes in their on-wire (big
/// endian) order, so they have to be swapped before unpacking the three
/// 5-bit letters.
pub(crate) fn pnp_id(raw: u16) -> String {
    let id = raw.swap_bytes();
    [10, 5, 0]
        .iter()
        .map(|shift| {
            let letter = ((id >> shift) & 0x1F) as u8;
            if (1..=26).contains(&letter) {
                (b'A' + letter - 1) as char
            } else {
                '?'
            }
        })
        .collect()
}

/// Reads the serial number from the monitor's EDID in the registry.
///
/// `device_path` looks like
/// `\\?\DISPLAY#DEL41B6#5&2f3c5ad&0&UID4353#{e6f07b5f-...}`. The middle
/// two parts name the device's key under `Enum\DISPLAY`, where Windows
/// keeps a copy of the EDID it read from the monitor. Bytes 12 to 15 are
/// the serial; many monitors leave it zero, which means "none".
pub(crate) fn edid_serial(device_path: &str) -> Option<u32> {
    let mut parts = device_path.split('#');
    let _prefix = parts.next()?;
    let model = parts.next()?;
    let instance = parts.next()?;
    let subkey = HSTRING::from(format!(
        r"SYSTEM\CurrentControlSet\Enum\DISPLAY\{model}\{instance}\Device Parameters"
    ));

    let mut edid = [0u8; 256];
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
    if status != ERROR_SUCCESS || size < 16 {
        return None;
    }
    let serial = u32::from_le_bytes([edid[12], edid[13], edid[14], edid[15]]);
    (serial != 0).then_some(serial)
}

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

pub(crate) fn wide_to_string(buffer: &[u16]) -> String {
    let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..len])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pnp_id_decodes_byte_swapped_manufacturer() {
        // "DEL" is 0x10AC in EDID byte order (bytes 0x10, 0xAC).
        assert_eq!(pnp_id(0xAC10), "DEL");
        // "GSM" (LG) is 0x1E6D.
        assert_eq!(pnp_id(0x6D1E), "GSM");
    }

    #[test]
    fn pnp_id_marks_invalid_letters() {
        assert_eq!(pnp_id(0), "???");
    }

    #[test]
    fn wide_to_string_stops_at_nul() {
        let buffer: Vec<u16> = "abc\0zzz".encode_utf16().collect();
        assert_eq!(wide_to_string(&buffer), "abc");
    }
}
