//! Reading what the shell sends about the icon, and writing the text it
//! shows.

use windows::Win32::UI::Shell::NIN_SELECT;
use windows::Win32::UI::WindowsAndMessaging::WM_CONTEXTMENU;

/// What the shell sends when the icon is chosen with the keyboard. The
/// `windows` crate leaves it out.
const NIN_KEYSELECT: u32 = NIN_SELECT | 1;

/// What a message about the icon asks for.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Click {
    /// The icon was clicked or chosen with the keyboard.
    Default,
    /// The menu was asked for, at this point on the screen.
    Menu(i32, i32),
}

/// Reads a message about the icon, in the form of version 4: the event in
/// the low word of `lparam`, and where to anchor a menu in `wparam`.
pub(super) fn click(wparam: usize, lparam: isize) -> Option<Click> {
    #[allow(clippy::cast_sign_loss)]
    let event = (lparam as usize & 0xFFFF) as u32;
    match event {
        NIN_SELECT | NIN_KEYSELECT => Some(Click::Default),
        WM_CONTEXTMENU => {
            let x = i32::from((wparam & 0xFFFF) as u16 as i16);
            let y = i32::from(((wparam >> 16) & 0xFFFF) as u16 as i16);
            Some(Click::Menu(x, y))
        }
        _ => None,
    }
}

/// Copies `text` into the fixed buffer `to`, cut short at a whole
/// character if it does not fit, and ends it with a nul.
pub(super) fn copy_wide(to: &mut [u16], text: &str) {
    let Some(room) = to.len().checked_sub(1) else {
        return;
    };
    let mut written = 0;
    for c in text.chars() {
        let mut units = [0; 2];
        let units = c.encode_utf16(&mut units);
        if written + units.len() > room {
            break;
        }
        to[written..written + units.len()].copy_from_slice(units);
        written += units.len();
    }
    to[written] = 0;
}

#[cfg(test)]
#[path = "decode_tests.rs"]
mod tests;
