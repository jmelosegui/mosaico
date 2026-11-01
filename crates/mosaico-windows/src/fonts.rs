//! The font families installed on the system, through GDI.

use mosaico_core::Result;
use mosaico_core::system::FontCatalog;
use windows::Win32::Foundation::LPARAM;
use windows::Win32::Graphics::Gdi::{
    DEFAULT_CHARSET, EnumFontFamiliesExW, GetDC, LOGFONTW, ReleaseDC, TEXTMETRICW,
};

/// Lists the font families GDI offers to every program.
pub struct InstalledFonts;

impl FontCatalog for InstalledFonts {
    fn families(&self) -> Result<Vec<String>> {
        let mut names: Vec<String> = Vec::new();
        let filter = LOGFONTW {
            lfCharSet: DEFAULT_CHARSET,
            ..Default::default()
        };
        // SAFETY: The screen device context is released before returning.
        // `filter` and `names` outlive the enumeration, which runs the
        // callback synchronously on this thread, and the callback only
        // treats its `lparam` as the `Vec` passed here.
        unsafe {
            let dc = GetDC(None);
            EnumFontFamiliesExW(
                dc,
                &filter,
                Some(collect),
                LPARAM(&mut names as *mut Vec<String> as isize),
                0,
            );
            ReleaseDC(None, dc);
        }
        Ok(names)
    }
}

/// Adds the family of one face to the list `lparam` points at.
///
/// # Safety
///
/// `font` must point to a valid `LOGFONTW` and `lparam` to a live
/// `Vec<String>`, as `EnumFontFamiliesExW` and [`InstalledFonts::families`]
/// guarantee.
unsafe extern "system" fn collect(
    font: *const LOGFONTW,
    _metrics: *const TEXTMETRICW,
    _kind: u32,
    lparam: LPARAM,
) -> i32 {
    // SAFETY: See the function's contract.
    let (font, names) = unsafe { (&*font, &mut *(lparam.0 as *mut Vec<String>)) };
    let len = font
        .lfFaceName
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(font.lfFaceName.len());
    names.push(String::from_utf16_lossy(&font.lfFaceName[..len]));
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_system_lists_some_font_families() {
        // Arrange
        let catalog = InstalledFonts;

        // Act
        let families = catalog.families().unwrap();

        // Assert
        assert!(!families.is_empty());
        assert!(families.iter().all(|name| !name.is_empty()));
    }
}
