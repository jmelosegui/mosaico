//! The mouse pointer: where it is, and the window under it for
//! focus-follows-mouse.

use mosaico_core::event::MousePointer;
use mosaico_core::{Error, Result, WindowId};
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::WindowsAndMessaging::{
    GA_ROOT, GetAncestor, GetCursorPos, WindowFromPoint,
};

use crate::window::Window;

/// The pointer as Win32 reports it.
///
/// `WindowFromPoint` finds the deepest window under the pointer, often a
/// control or a browser's render widget, so `GetAncestor(GA_ROOT)`
/// climbs from it to the top-level window. Both give a null handle when
/// there is no window. `GetCursorPos` fails while another desktop takes
/// the input, such as the lock screen or an elevation prompt.
pub struct WindowsPointer;

impl MousePointer for WindowsPointer {
    fn window_under(&self) -> Result<Option<WindowId>> {
        let (x, y) = position()?;
        // SAFETY: WindowFromPoint and GetAncestor are read-only queries
        // that take any point and handle, and return null for none.
        let root = unsafe { GetAncestor(WindowFromPoint(POINT { x, y }), GA_ROOT) };
        Ok((!root.is_invalid()).then(|| Window::new(root).id()))
    }
}

/// Where the pointer is, in desktop coordinates, with `GetCursorPos`.
pub(crate) fn position() -> Result<(i32, i32)> {
    let mut point = POINT::default();
    // SAFETY: GetCursorPos writes the cursor position into a stack
    // allocated POINT and changes nothing.
    unsafe { GetCursorPos(&mut point) }
        .map_err(|e| Error::platform("read the pointer position", e))?;
    Ok((point.x, point.y))
}
