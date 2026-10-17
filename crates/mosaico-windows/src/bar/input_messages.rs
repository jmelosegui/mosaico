//! What the pointer messages a bar gets say, read from their parameters.

use mosaico_core::bar::{Button, ItemRegion, hit};
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    WHEEL_DELTA, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEHWHEEL,
    WM_RBUTTONDOWN, WM_RBUTTONUP,
};

/// `WM_MOUSELEAVE`, which the windows crate only has with the common
/// controls.
pub(super) const WM_MOUSELEAVE: u32 = 0x02A3;

/// `MK_LBUTTON`, `MK_RBUTTON` and `MK_MBUTTON`, the buttons a mouse
/// message's `wparam` says are down, which the windows crate only has
/// with the system services.
const BUTTONS_DOWN: usize = 0x0001 | 0x0002 | 0x0010;

/// The button a button message is about, and whether it went down.
pub(super) fn button(msg: u32) -> Option<(Button, bool)> {
    match msg {
        WM_LBUTTONDOWN => Some((Button::Left, true)),
        WM_LBUTTONUP => Some((Button::Left, false)),
        WM_RBUTTONDOWN => Some((Button::Right, true)),
        WM_RBUTTONUP => Some((Button::Right, false)),
        WM_MBUTTONDOWN => Some((Button::Middle, true)),
        WM_MBUTTONUP => Some((Button::Middle, false)),
        _ => None,
    }
}

/// Whether a mouse message's `wparam` says a button is still down.
pub(super) fn buttons_down(wparam: WPARAM) -> bool {
    wparam.0 & BUTTONS_DOWN != 0
}

/// The point in a mouse message's `lparam`: two signed 16-bit numbers, x
/// in the low word, so a point left of or above the window is negative.
/// In the window's coordinates for a button message, and the screen's
/// for the wheel.
pub(super) fn point(lparam: LPARAM) -> (i32, i32) {
    let x = (lparam.0 & 0xFFFF) as u16 as i16;
    let y = ((lparam.0 >> 16) & 0xFFFF) as u16 as i16;
    (i32::from(x), i32::from(y))
}

/// How far a wheel message says the wheel turned, in notches, positive
/// down or to the right. A wheel turned away from the user, which
/// scrolls up, reports a positive delta, and so does a tilt to the right.
pub(super) fn notches(msg: u32, wparam: WPARAM) -> f32 {
    let delta = ((wparam.0 >> 16) & 0xFFFF) as u16 as i16;
    let notches = f32::from(delta) / WHEEL_DELTA as f32;
    if msg == WM_MOUSEHWHEEL {
        notches
    } else {
        -notches
    }
}

/// The hit test code in the low word of a `WM_SETCURSOR` message's
/// `lparam`.
pub(super) fn hit_test(lparam: LPARAM) -> u32 {
    (lparam.0 & 0xFFFF) as u32
}

/// Whether the pointer at (`x`, `y`) on a bar that drew `regions` is over
/// an item a click on does something with, where it shows a hand.
pub(super) fn over_clickable(regions: &[ItemRegion], x: i32, y: i32) -> bool {
    hit(regions, x, y).is_some_and(|region| region.clickable)
}

#[cfg(test)]
#[path = "input_messages_tests.rs"]
mod tests;
