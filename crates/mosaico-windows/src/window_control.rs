//! Changes to windows and the pointer, made through Win32.

mod hiding;
mod stacking;

use mosaico_core::config::HidingBehaviour;
use mosaico_core::{Error, HidingEvents, Rect, Result, WindowControl, WindowId};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    PostMessageW, SetCursorPos, SetForegroundWindow, WM_CLOSE,
};

use crate::error::OrPlatform;
use crate::window::Window;

/// [`WindowControl`] on the Win32 desktop.
///
/// Holds no state: every call goes straight to the window's handle.
/// `ShowWindow` does not say whether it worked, so the calls built on
/// it first check that the window still exists. Hiding, showing and
/// minimizing windows are in `window_control/hiding.rs`, and putting
/// one behind another in `window_control/stacking.rs`.
#[derive(Debug, Default, Clone, Copy)]
pub struct Win32WindowControl;

impl WindowControl for Win32WindowControl {
    fn place(&self, window: WindowId, rect: Rect) -> Result<()> {
        Window::from_id(window).set_rect(&rect)
    }

    /// Posts the move instead of sending it (see `Window::move_async`),
    /// so a window whose thread never pumps messages, or is still busy
    /// creating the window, cannot block the daemon.
    fn pre_place(&self, window: WindowId, rect: Rect) -> Result<()> {
        Window::from_id(window).move_async(&rect)
    }

    fn repaint(&self, window: WindowId) -> Result<()> {
        Window::from_id(window).invalidate()
    }

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

    fn put_behind(&self, window: WindowId, other: WindowId) -> Result<()> {
        stacking::put_behind(existing(window)?, existing(other)?)
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
        hiding::minimize(existing(window)?);
        Ok(())
    }

    fn hide(&self, window: WindowId, how: HidingBehaviour) -> Result<()> {
        hiding::hide(existing(window)?, how)
    }

    fn show(&self, window: WindowId, how: HidingBehaviour) -> Result<()> {
        hiding::show(existing(window)?, how)
    }

    fn hiding_events(&self, how: HidingBehaviour) -> Option<HidingEvents> {
        Some(hiding::events(how))
    }

    fn reveal(&self, window: WindowId) -> Result<()> {
        hiding::reveal(existing(window)?)
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
