//! Changes to windows and the pointer, made through Win32.

mod frames;
mod hiding;
mod ordered;
mod posted_placement;
mod stacking;

use std::sync::{Mutex, MutexGuard, PoisonError};

use mosaico_core::config::HidingBehaviour;
use mosaico_core::frames::FrameTable;
use mosaico_core::{Error, HidingEvents, Rect, Result, WindowControl, WindowId};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    PostMessageW, SetCursorPos, SetForegroundWindow, WM_CLOSE,
};

use crate::error::OrPlatform;
use crate::window::Window;

use frames::Kept;

/// [`WindowControl`] on the Win32 desktop.
///
/// Every call goes straight to the window's handle, and only the frames
/// of transitions leave something behind: the [`FrameTable`] that holds
/// a frame back while the last one may be on its way, read through
/// `window_control/frames.rs`, and says when a change is to be posted
/// behind it (`window_control/ordered.rs`). `ShowWindow` does not say
/// whether it worked, so the calls built on it first check that the
/// window still exists. Hiding, showing and minimizing windows are in
/// `window_control/hiding.rs`, and putting one behind another in
/// `window_control/stacking.rs`.
#[derive(Debug, Default)]
pub struct Win32WindowControl {
    frames: Mutex<FrameTable<Kept>>,
}

impl Win32WindowControl {
    /// What is kept about the frames posted. A panic while it was held
    /// leaves nothing half done that matters, so the lock is taken back.
    fn frames(&self) -> MutexGuard<'_, FrameTable<Kept>> {
        self.frames.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl WindowControl for Win32WindowControl {
    /// Made at once, unless something may be queued for the window: then
    /// posted behind it, as the frame table says, to land after it.
    fn place(&self, window: WindowId, rect: Rect) -> Result<()> {
        self.ordered(window, |posted| self.move_into_place(window, &rect, posted))
    }

    /// Posts the move instead of sending it (see `Window::move_async`),
    /// so a window whose thread never pumps messages, or is still busy
    /// creating the window, cannot block the daemon.
    fn pre_place(&self, window: WindowId, rect: Rect) -> Result<()> {
        Window::from_id(window).move_async(&rect)
    }

    /// Without waiting for the application when something may be queued
    /// for the window, so the paint comes after what was posted.
    fn repaint(&self, window: WindowId) -> Result<()> {
        Window::from_id(window).redraw(self.repaint_flags(window))
    }

    /// Never posted, so it may reach a window before a show posted to it.
    fn focus(&self, window: WindowId) -> Result<()> {
        // SAFETY: SetForegroundWindow is safe to call with a valid HWND;
        // an invalid one makes it fail without side effects.
        let focused = unsafe { SetForegroundWindow(hwnd(window)) };
        if !focused.as_bool() {
            return Err(Error::platform(
                "focus the window",
                "it was not brought to the foreground",
            ));
        }
        Ok(())
    }

    /// Posted behind what may be queued for `window`, as a placement is.
    fn put_behind(&self, window: WindowId, other: WindowId) -> Result<()> {
        let (win, other) = (existing(window)?, existing(other)?);
        self.ordered(window, |posted| stacking::put_behind(win, other, posted))
    }

    /// Posts `WM_CLOSE`, the message the window's close button leads
    /// to, without waiting for the application to handle it.
    fn close(&self, window: WindowId) -> Result<()> {
        // SAFETY: PostMessageW only queues a message; an invalid handle
        // makes it fail without side effects.
        unsafe { PostMessageW(Some(hwnd(window)), WM_CLOSE, WPARAM(0), LPARAM(0)) }
            .or_platform("ask the window to close")
    }

    fn minimize(&self, window: WindowId) -> Result<()> {
        let win = existing(window)?;
        self.ordered_show_state(window, false, |posted| hiding::minimize(win, posted))
    }

    fn hide(&self, window: WindowId, how: HidingBehaviour) -> Result<()> {
        let win = existing(window)?;
        if !hiding::sets_show_state(how) {
            return hiding::hide(win, how, false);
        }
        self.ordered_show_state(window, false, |posted| hiding::hide(win, how, posted))
    }

    fn show(&self, window: WindowId, how: HidingBehaviour) -> Result<()> {
        let win = existing(window)?;
        if !hiding::sets_show_state(how) {
            return hiding::show(win, how, false);
        }
        self.ordered_show_state(window, true, |posted| hiding::show(win, how, posted))
    }

    fn hiding_events(&self, how: HidingBehaviour) -> Option<HidingEvents> {
        Some(hiding::events(how))
    }

    fn reveal(&self, window: WindowId) -> Result<()> {
        let win = existing(window)?;
        let uncloaked = hiding::uncloak(win);
        self.show_revealed(window, win).and(uncloaked)
    }

    fn move_cursor(&self, x: i32, y: i32) -> Result<()> {
        // SAFETY: SetCursorPos only takes screen coordinates.
        unsafe { SetCursorPos(x, y) }.or_platform("move the pointer")
    }
}

/// The handle a window id stands for.
fn hwnd(window: WindowId) -> HWND {
    Window::from_id(window).hwnd()
}

/// The window an id stands for, or an error once it no longer exists.
fn existing(window: WindowId) -> Result<Window> {
    let win = Window::from_id(window);
    if !win.is_valid() {
        return Err(Error::platform("find the window", "it no longer exists"));
    }
    Ok(win)
}

#[cfg(test)]
#[path = "window_control_tests.rs"]
mod tests;
