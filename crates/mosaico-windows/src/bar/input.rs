//! The pointer on a bar: what its window procedure reports to the bar's
//! sink, and the pointer's shape over the bar.
//!
//! The window takes no activation: `WS_EX_NOACTIVATE`, and
//! `MA_NOACTIVATE` for `WM_MOUSEACTIVATE`. A press holds the pointer with
//! `SetCapture` until the last button is released. `WM_SETCURSOR` shows
//! the hand over a clickable item, found with core's `hit` on the regions
//! the bar drew last, and `TrackMouseEvent` reports the pointer leaving.
//! Moves are not reported: hovering is not input.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use mosaico_core::bar::{BarId, ItemRegion, PointerEvent, PointerSink};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    GWLP_USERDATA, GetWindowLongPtrW, HTCLIENT, MA_NOACTIVATE, WM_CAPTURECHANGED, WM_MOUSEACTIVATE,
    WM_MOUSEHWHEEL, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_SETCURSOR,
};

use super::input_messages::{
    WM_MOUSELEAVE, button, buttons_down, hit_test, notches, over_clickable, point,
};
use super::input_os::{hear_leaving, hold, let_go, on_window, pointer_on, show_pointer};

/// What a bar's window procedure needs to report the pointer, kept where
/// the window finds it, in its user data.
pub(super) struct BarInput {
    id: BarId,
    sink: Rc<dyn PointerSink>,
    /// Where the bar drew its items last, for the pointer's shape.
    regions: RefCell<Vec<ItemRegion>>,
    /// Whether the window asked to hear when the pointer leaves it.
    tracking: Cell<bool>,
    /// Whether the window holds the pointer for a press.
    holding: Cell<bool>,
    /// Whether the bar lets the pointer through, as it does when made.
    click_through: Cell<bool>,
}

impl BarInput {
    /// The input of the bar `id`, which reports to `sink`.
    pub(super) fn new(id: BarId, sink: Rc<dyn PointerSink>) -> Self {
        Self {
            id,
            sink,
            regions: RefCell::new(Vec::new()),
            tracking: Cell::new(false),
            holding: Cell::new(false),
            click_through: Cell::new(true),
        }
    }

    /// The input of the bar whose window is `hwnd`, or `None` for a window
    /// that has none, as one of the bar class made by a test.
    ///
    /// # Safety
    ///
    /// Only for the bar class's window procedure, for the message it is
    /// handling: a bar keeps its input until its window no longer points
    /// at it, and the window only runs on the thread that made it.
    pub(super) unsafe fn of<'a>(hwnd: HWND) -> Option<&'a BarInput> {
        // SAFETY: reads the user data of the window being handled.
        let address = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) };
        // SAFETY: the user data of a bar's window is 0 or the address of
        // that bar's input, alive as the caller's contract says.
        unsafe { (address as *const BarInput).as_ref() }
    }

    /// Whether the bar lets the pointer through.
    pub(super) fn click_through(&self) -> bool {
        self.click_through.get()
    }

    /// Notes that the bar now lets the pointer through, or takes it.
    pub(super) fn set_click_through(&self, click_through: bool) {
        self.click_through.set(click_through);
    }

    /// Keeps where the bar drew its items, for the pointer's shape.
    pub(super) fn set_regions(&self, regions: Vec<ItemRegion>) {
        *self.regions.borrow_mut() = regions;
    }

    /// Handles a message of the bar's window that is about the pointer,
    /// and returns what the window procedure answers, or `None` to leave
    /// the message to the default handling.
    pub(super) fn handle(
        &self,
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> Option<LRESULT> {
        match msg {
            WM_MOUSEACTIVATE => return Some(LRESULT(MA_NOACTIVATE as isize)),
            WM_SETCURSOR if hit_test(lparam) == HTCLIENT => {
                let regions = self.regions.borrow();
                let hand = pointer_on(hwnd).is_some_and(|(x, y)| over_clickable(&regions, x, y));
                show_pointer(hand);
                return Some(LRESULT(1));
            }
            WM_MOUSEMOVE if !self.tracking.get() => self.tracking.set(hear_leaving(hwnd)),
            WM_MOUSEMOVE => {}
            WM_MOUSELEAVE => {
                self.tracking.set(false);
                // While a press holds the pointer, its release is still to come.
                if !self.holding.get() {
                    self.send(PointerEvent::Leave);
                }
            }
            WM_CAPTURECHANGED => {
                // Something else took the pointer before the release.
                if self.holding.replace(false) {
                    self.send(PointerEvent::Leave);
                }
            }
            WM_MOUSEWHEEL | WM_MOUSEHWHEEL => {
                let (x, y) = on_window(hwnd, point(lparam));
                let notches = notches(msg, wparam);
                self.send(PointerEvent::Wheel { notches, x, y });
            }
            _ => self.button(hwnd, msg, wparam, lparam)?,
        }
        Some(LRESULT(0))
    }

    /// Handles a press or a release, or returns `None` for any other
    /// message.
    fn button(&self, hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> Option<()> {
        let (button, down) = button(msg)?;
        let (x, y) = point(lparam);
        if down {
            hold(hwnd);
            self.holding.set(true);
            self.send(PointerEvent::Press { button, x, y });
        } else {
            self.send(PointerEvent::Release { button, x, y });
            if !buttons_down(wparam) && self.holding.replace(false) {
                let_go();
            }
        }
        Some(())
    }

    fn send(&self, event: PointerEvent) {
        self.sink.send(self.id, event);
    }
}
