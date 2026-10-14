//! Colors as the configuration writes them, `"#RRGGBB"`.
//!
//! Border and bar colors are kept in the configuration as text, which
//! may also be a theme color name; the theme resolves names to hex
//! before anything is drawn. [`Color`] is what the hex reads as.

/// An opaque color in 8-bit RGB.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    /// Red, from 0 to 255.
    pub r: u8,
    /// Green, from 0 to 255.
    pub g: u8,
    /// Blue, from 0 to 255.
    pub b: u8,
}

impl Color {
    /// Parses a hex color string like `"#00b4d8"` or `"00b4d8"`.
    ///
    /// Returns `None` for anything else, such as a color name the theme
    /// did not resolve.
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }
        Some(Self {
            r: u8::from_str_radix(&hex[0..2], 16).ok()?,
            g: u8::from_str_radix(&hex[2..4], 16).ok()?,
            b: u8::from_str_radix(&hex[4..6], 16).ok()?,
        })
    }
}

#[cfg(test)]
#[path = "color_tests.rs"]
mod tests;
