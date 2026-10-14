//! Taking windows out of view and bringing them back, through Win32.
//!
//! Cloaking goes through the ImmersiveShell COM interface, the mechanism
//! Windows uses for its own virtual desktops: the window becomes
//! invisible but keeps its taskbar icon. The other ways set the window's
//! show state with `ShowWindow`: `SW_HIDE` also removes the taskbar icon,
//! and `SW_MINIMIZE` minimizes it. The events each way fires are listed
//! on [`events`].

use mosaico_core::config::HidingBehaviour;
use mosaico_core::{HidingEvents, Result};
use windows::Win32::UI::WindowsAndMessaging::{
    SHOW_WINDOW_CMD, SW_HIDE, SW_MINIMIZE, SW_SHOW, SW_SHOWNOACTIVATE, ShowWindow,
};

use crate::com;
use crate::window::Window;

/// Takes a window out of view for a workspace switch, the way `how`
/// says.
pub(super) fn hide(win: Window, how: HidingBehaviour) -> Result<()> {
    match how {
        HidingBehaviour::Cloak => return com::cloak_window(win.hwnd()),
        HidingBehaviour::Hide => show_window(win, SW_HIDE),
        HidingBehaviour::Minimize => minimize(win),
    }
    Ok(())
}

/// Shows a window hidden `how` again without activating it, so a
/// workspace switch does not move focus. Uncloaking a window that is not
/// cloaked does nothing.
pub(super) fn show(win: Window, how: HidingBehaviour) -> Result<()> {
    match how {
        HidingBehaviour::Cloak => return com::uncloak_window(win.hwnd()),
        HidingBehaviour::Hide | HidingBehaviour::Minimize => show_window(win, SW_SHOWNOACTIVATE),
    }
    Ok(())
}

/// What hiding a window `how`, and showing it again, reports. Every way
/// is available.
///
/// Cloaking and uncloaking fire neither `EVENT_OBJECT_HIDE` nor the
/// minimize events. `SW_HIDE` fires `EVENT_OBJECT_HIDE`, and showing the
/// window again fires `EVENT_OBJECT_SHOW`, which arrives as a new window
/// rather than a restore. `SW_MINIMIZE` fires
/// `EVENT_SYSTEM_MINIMIZESTART`, and showing the window again
/// `EVENT_SYSTEM_MINIMIZEEND`.
pub(super) fn events(how: HidingBehaviour) -> HidingEvents {
    let (reports_hidden, reports_restored) = match how {
        HidingBehaviour::Cloak => (false, false),
        HidingBehaviour::Hide => (true, false),
        HidingBehaviour::Minimize => (true, true),
    };
    HidingEvents {
        reports_hidden,
        reports_restored,
    }
}

/// Uncloaks, then shows with `SW_SHOW`, which activates more strongly
/// than `SW_SHOWNOACTIVATE` and makes sure Windows fully restores the
/// window's visibility state. The window is shown even when uncloaking
/// it fails.
pub(super) fn reveal(win: Window) -> Result<()> {
    let uncloaked = com::uncloak_window(win.hwnd());
    show_window(win, SW_SHOW);
    uncloaked
}

/// Minimizes a window.
pub(super) fn minimize(win: Window) {
    show_window(win, SW_MINIMIZE);
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
