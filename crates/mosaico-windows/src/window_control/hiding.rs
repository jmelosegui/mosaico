//! Taking windows out of view and bringing them back, through Win32.
//!
//! Cloaking goes through the ImmersiveShell COM interface, the mechanism
//! Windows uses for its own virtual desktops: the window becomes
//! invisible but keeps its taskbar icon. The other ways set the window's
//! show state with `ShowWindow`: `SW_HIDE` also removes the taskbar icon,
//! and `SW_MINIMIZE` minimizes it. The events each way fires are listed
//! on [`events`].
//!
//! A change of the show state made with `ShowWindow` takes effect at
//! once, and a frame of a transition still queued for the window would
//! overtake it, as it would any change applied at once: after a minimize,
//! the frame would move the minimized window's caption on screen. So
//! while something may still be queued for the window, the change is
//! posted with `ShowWindowAsync` instead, and lands after it, in order
//! with every frame and change posted around it. Cloaking does not go
//! through the window's queue, and is never posted.

use mosaico_core::config::HidingBehaviour;
use mosaico_core::{HidingEvents, Result};
use windows::Win32::UI::WindowsAndMessaging::{
    SHOW_WINDOW_CMD, SW_HIDE, SW_MINIMIZE, SW_SHOW, SW_SHOWNOACTIVATE, ShowWindow, ShowWindowAsync,
};

use crate::com;
use crate::error::OrPlatform;
use crate::window::Window;

/// Takes a window out of view for a workspace switch, the way `how`
/// says, posted behind what may be queued for it when `posted`. Cloaking
/// is not a show state, and never posted.
pub(super) fn hide(win: Window, how: HidingBehaviour, posted: bool) -> Result<()> {
    match how {
        HidingBehaviour::Cloak => com::cloak_window(win.hwnd()),
        HidingBehaviour::Hide => set_show_state(win, SW_HIDE, posted),
        HidingBehaviour::Minimize => set_show_state(win, SW_MINIMIZE, posted),
    }
}

/// Shows a window hidden `how` again without activating it, so a
/// workspace switch does not move focus, posted behind what may be queued
/// for it when `posted`. Uncloaking a window that is not cloaked does
/// nothing.
pub(super) fn show(win: Window, how: HidingBehaviour, posted: bool) -> Result<()> {
    match how {
        HidingBehaviour::Cloak => com::uncloak_window(win.hwnd()),
        HidingBehaviour::Hide | HidingBehaviour::Minimize => {
            set_show_state(win, SW_SHOWNOACTIVATE, posted)
        }
    }
}

/// Whether hiding or showing a window `how` changes its show state, and
/// so keeps its order with what is posted to the window: all but
/// cloaking, which goes through the shell.
pub(super) fn sets_show_state(how: HidingBehaviour) -> bool {
    how != HidingBehaviour::Cloak
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

/// Uncloaks a window `reveal` shows, which goes through the shell and is
/// never posted.
pub(super) fn uncloak(win: Window) -> Result<()> {
    com::uncloak_window(win.hwnd())
}

/// Shows a window `reveal` uncloaked with `SW_SHOW`, which activates more
/// strongly than `SW_SHOWNOACTIVATE` and makes sure Windows fully restores
/// the window's visibility state, posted behind what may be queued for it
/// when `posted`. It is shown even when uncloaking it failed.
pub(super) fn show_revealed(win: Window, posted: bool) -> Result<()> {
    set_show_state(win, SW_SHOW, posted)
}

/// Whether a window `reveal` shows is in view once that lands: `SW_SHOW`
/// leaves one that is `minimized` minimized.
pub(super) fn revealed_in_view(minimized: bool) -> bool {
    !minimized
}

/// Minimizes a window, posted behind what may be queued for it when
/// `posted`.
pub(super) fn minimize(win: Window, posted: bool) -> Result<()> {
    set_show_state(win, SW_MINIMIZE, posted)
}

/// Sets a window's show state at once, or posts it to the window's
/// thread, behind what was posted to it before, when `posted`.
///
/// `ShowWindow` reports whether the window was visible before, not
/// whether the call worked, so there is no failure to pass on.
/// `ShowWindowAsync` says whether it queued the change.
fn set_show_state(window: Window, command: SHOW_WINDOW_CMD, posted: bool) -> Result<()> {
    if posted {
        // SAFETY: ShowWindowAsync only posts the show state to the
        // window's thread; an invalid handle makes it fail without side
        // effects.
        return unsafe { ShowWindowAsync(window.hwnd(), command) }
            .ok()
            .or_platform("queue the window's show state");
    }
    // SAFETY: ShowWindow is safe to call with a valid HWND; an invalid
    // one makes it fail without side effects.
    unsafe {
        let _ = ShowWindow(window.hwnd(), command);
    }
    Ok(())
}
