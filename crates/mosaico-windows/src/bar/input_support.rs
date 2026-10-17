//! Hidden bars of the tests' own, never shown, and the messages Windows
//! sends a bar.

use std::cell::RefCell;
use std::rc::Rc;

use mosaico_core::Rect;
use mosaico_core::bar::{BarId, PointerEvent, PointerSink};
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::GetCapture;
use windows::Win32::UI::WindowsAndMessaging::{GWL_EXSTYLE, GetWindowLongPtrW, SendMessageW};

use super::super::window::Bar;

/// The id the tests' bars are made with.
pub(super) const ID: BarId = BarId {
    monitor: 1,
    generation: 3,
};

/// What a bar reported, in order.
#[derive(Default)]
pub(super) struct Reported(RefCell<Vec<(BarId, PointerEvent)>>);

impl PointerSink for Reported {
    fn send(&self, bar: BarId, event: PointerEvent) {
        self.0.borrow_mut().push((bar, event));
    }
}

impl Reported {
    /// What was reported since the last call, all of it under [`ID`].
    pub(super) fn take(&self) -> Vec<PointerEvent> {
        let reported = std::mem::take(&mut *self.0.borrow_mut());
        assert!(reported.iter().all(|&(bar, _)| bar == ID), "{reported:?}");
        reported.into_iter().map(|(_, event)| event).collect()
    }
}

/// A hidden bar reporting to a sink of the test's own, or `None` on a
/// machine with no interactive desktop, which skips.
pub(super) fn hidden_bar() -> Option<(Bar, Rc<Reported>)> {
    let sink = Rc::new(Reported::default());
    let bar = Bar::new(ID, Rect::new(0, 0, 300, 40), sink.clone()).ok()?;
    Some((bar, sink))
}

/// Sends `msg` to the bar's window, the way Windows does, and returns the
/// answer of its window procedure.
pub(super) fn send(bar: &Bar, msg: u32, wparam: usize, lparam: isize) -> isize {
    // SAFETY: sending a message to a window this thread created.
    unsafe { SendMessageW(bar.hwnd, msg, Some(WPARAM(wparam)), Some(LPARAM(lparam))) }.0
}

/// The `lparam` of a mouse message at (`x`, `y`).
pub(super) fn at(x: i16, y: i16) -> isize {
    ((y as u16 as isize) << 16) | x as u16 as isize
}

/// The extended style of the bar's window.
pub(super) fn ex_style(bar: &Bar) -> u32 {
    // SAFETY: reads the style of a window this thread created.
    unsafe { GetWindowLongPtrW(bar.hwnd, GWL_EXSTYLE) as u32 }
}

/// Whether the bar's window holds the pointer.
pub(super) fn holds_pointer(bar: &Bar) -> bool {
    // SAFETY: reads which window of this thread holds the pointer.
    unsafe { GetCapture() == bar.hwnd }
}
