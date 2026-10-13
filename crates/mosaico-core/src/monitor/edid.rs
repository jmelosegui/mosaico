//! Decoding of the display identity stored in an EDID block.
//!
//! Every platform can get at the raw EDID a display reports, so the
//! decoding lives here instead of being repeated in each platform crate.

use super::EdidIdentity;

/// Byte offset of the manufacturer ID (big endian, three 5-bit letters).
const MANUFACTURER: usize = 8;
/// Byte offset of the product code (little endian).
const PRODUCT: usize = 10;
/// Byte offset of the serial number (little endian, zero when unset).
const SERIAL: usize = 12;
/// The 8-byte header every EDID block starts with.
const HEADER: [u8; 8] = [0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00];

impl EdidIdentity {
    /// Reads manufacturer, product and serial from a raw EDID block.
    ///
    /// Returns `None` when the block is too short or does not start with
    /// the EDID header.
    pub fn from_edid(edid: &[u8]) -> Option<Self> {
        if edid.len() < SERIAL + 4 || edid[..HEADER.len()] != HEADER {
            return None;
        }
        let serial = u32::from_le_bytes([
            edid[SERIAL],
            edid[SERIAL + 1],
            edid[SERIAL + 2],
            edid[SERIAL + 3],
        ]);
        Some(Self {
            manufacturer: pnp_manufacturer(u16::from_be_bytes([
                edid[MANUFACTURER],
                edid[MANUFACTURER + 1],
            ])),
            product: u16::from_le_bytes([edid[PRODUCT], edid[PRODUCT + 1]]),
            serial: (serial != 0).then_some(serial),
        })
    }
}

/// Decodes a PNP manufacturer ID as stored in EDID (bytes 8 and 9, read
/// big endian) into its three letter form, e.g. `0x10AC` -> `"DEL"`.
///
/// Letters outside `A..=Z` are shown as `?`.
pub fn pnp_manufacturer(id: u16) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn edid_block(serial: u32) -> Vec<u8> {
        let mut edid = vec![0u8; 128];
        edid[..8].copy_from_slice(&HEADER);
        edid[8..10].copy_from_slice(&[0x10, 0xAC]); // DEL
        edid[10..12].copy_from_slice(&0xA0E9u16.to_le_bytes());
        edid[12..16].copy_from_slice(&serial.to_le_bytes());
        edid
    }

    #[test]
    fn decodes_manufacturer_product_and_serial() {
        // Act
        let id = EdidIdentity::from_edid(&edid_block(808_726_860));

        // Assert
        assert_eq!(
            id,
            Some(EdidIdentity {
                manufacturer: "DEL".into(),
                product: 0xA0E9,
                serial: Some(808_726_860),
            })
        );
    }

    #[test]
    fn zero_serial_means_none() {
        // Act
        let id = EdidIdentity::from_edid(&edid_block(0));

        // Assert
        assert_eq!(id.and_then(|i| i.serial), None);
    }

    #[test]
    fn rejects_short_or_headerless_blocks() {
        // Arrange
        let mut bad_header = edid_block(1);
        bad_header[0] = 0x42;

        // Act / Assert
        assert_eq!(EdidIdentity::from_edid(&[0u8; 10]), None);
        assert_eq!(EdidIdentity::from_edid(&bad_header), None);
    }

    #[test]
    fn pnp_manufacturer_decodes_known_ids() {
        assert_eq!(pnp_manufacturer(0x10AC), "DEL");
        assert_eq!(pnp_manufacturer(0x1E6D), "GSM");
        assert_eq!(pnp_manufacturer(0), "???");
    }
}
