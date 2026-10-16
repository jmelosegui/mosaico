//! Changes to windows and the pointer, made through Win32.

use mosaico_core::config::HidingBehaviour;
use mosaico_core::{Error, HidingEvents, Rect, Result, WindowControl, WindowId};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    PostMessageW, SHOW_WINDOW_CMD, SW_HIDE, SW_MINIMIZE, SW_SHOW, SW_SHOWNOACTIVATE, SetCursorPos,
    SetForegroundWindow, ShowWindow, WM_CLOSE,
};

use crate::com;
use crate::error::OrPlatform;
use crate::window::Window;

/// [`WindowControl`] on the Win32 desktop.
///
/// Holds no state: every call goes straight to the window's handle.
/// `ShowWindow` does not say whether it worked, so the calls built on
/// it first check that the window still exists.
#[derive(Debug, Default, Clone, Copy)]
pub struct Win32WindowControl;

impl WindowControl for Win32WindowControl {
    fn place(&self, window: WindowId, rect: Rect) -> Result<()> {
        Window::from_id(window).set_rect(&rect)
    }

    /// Posts the move instead of sending it (see `Window::move_async`),
    /// so a window whose thread never pumps messages cannot block the
    /// daemon.
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

    /// Posts `WM_CLOSE`, the message the window's close button leads
    /// to, without waiting for the application to handle it.
    fn close(&self, window: WindowId) -> Result<()> {
        // SAFETY: PostMessageW only queues a message; an invalid handle
        // makes it fail without side effects.
        unsafe { PostMessageW(Some(hwnd(window)), WM_CLOSE, WPARAM(0), LPARAM(0)) }
            .or_platform("ask the window to close")
    }

    fn minimize(&self, window: WindowId) -> Result<()> {
        show_window(existing(window)?, SW_MINIMIZE);
        Ok(())
    }

    /// Cloaking goes through the ImmersiveShell COM interface, the
    /// mechanism Windows uses for its own virtual desktops: the window
    /// becomes invisible but keeps its taskbar icon. `SW_HIDE` also
    /// removes the taskbar icon. The events each way fires are listed on
    /// `hiding_events`.
    fn hide(&self, window: WindowId, how: HidingBehaviour) -> Result<()> {
        let win = existing(window)?;
        match how {
            HidingBehaviour::Cloak => return com::cloak_window(win.hwnd()),
            HidingBehaviour::Hide => show_window(win, SW_HIDE),
            HidingBehaviour::Minimize => show_window(win, SW_MINIMIZE),
        }
        Ok(())
    }

    /// Shows without activating, so a workspace switch does not move
    /// focus. Uncloaking a window that is not cloaked does nothing.
    fn show(&self, window: WindowId, how: HidingBehaviour) -> Result<()> {
        let win = existing(window)?;
        match how {
            HidingBehaviour::Cloak => return com::uncloak_window(win.hwnd()),
            HidingBehaviour::Hide | HidingBehaviour::Minimize => {
                show_window(win, SW_SHOWNOACTIVATE)
            }
        }
        Ok(())
    }

    /// Every way is available. Cloaking and uncloaking fire neither
    /// `EVENT_OBJECT_HIDE` nor the minimize events. `SW_HIDE` fires
    /// `EVENT_OBJECT_HIDE`, and showing the window again fires
    /// `EVENT_OBJECT_SHOW`, which arrives as a new window rather than a
    /// restore. `SW_MINIMIZE` fires `EVENT_SYSTEM_MINIMIZESTART`, and
    /// showing the window again `EVENT_SYSTEM_MINIMIZEEND`.
    fn hiding_events(&self, how: HidingBehaviour) -> Option<HidingEvents> {
        let (reports_hidden, reports_restored) = match how {
            HidingBehaviour::Cloak => (false, false),
            HidingBehaviour::Hide => (true, false),
            HidingBehaviour::Minimize => (true, true),
        };
        Some(HidingEvents {
            reports_hidden,
            reports_restored,
        })
    }

    /// Uncloaks, then shows with `SW_SHOW`, which activates more strongly
    /// than `SW_SHOWNOACTIVATE` and makes sure Windows fully restores the
    /// window's visibility state. The window is shown even when
    /// uncloaking it fails.
    fn reveal(&self, window: WindowId) -> Result<()> {
        let win = existing(window)?;
        let uncloaked = com::uncloak_window(win.hwnd());
        show_window(win, SW_SHOW);
        uncloaked
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

/// Sets a window's show state.
///
/// `ShowWindow` reports whether the window was visible before, not
/// whether the call worked, so there is no failure to pass on.
fn show_window(window: Window, command: SHOW_WINDOW_CMD) {
    // SAFETY: ShowWindow is safe to call with a valid HWND; an invalid
    // one makes it fail without side effects.
    unsafe {
        let _ = ShowWindow(window.hwnd(), command);
    }
}

#[cfg(test)]
#[path = "window_control_tests.rs"]
mod tests;
