//! The windows on screen, which the bars look through for a maximized or
//! fullscreen window under them.

use mosaico_core::WindowId;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GWL_EXSTYLE, GetWindowLongPtrW, WS_EX_LAYERED, WS_EX_TRANSPARENT,
};

use crate::enumerate::top_level_windows;
use crate::window::Window;

/// The classes of the shell's own surfaces: the desktop, which is
/// `Progman` and the `WorkerW` windows that hold the wallpaper, all of
/// them over every monitor, and the taskbars, on the primary monitor and
/// on the others.
const SHELL_CLASSES: [&str; 4] = [
    "Progman",
    "WorkerW",
    "Shell_TrayWnd",
    "Shell_SecondaryTrayWnd",
];

/// The top-level windows that are on screen: shown (`IsWindowVisible`) and
/// not minimized (`IsIconic`), not layered with `WS_EX_TRANSPARENT`, which
/// lets the pointer through, not of this process, which draws the bars and
/// the borders, not cloaked, which is how both virtual desktops and
/// `hiding = "cloak"` hide a window, and not of the shell. None when the
/// windows cannot be listed.
pub(super) fn windows_on_screen() -> Vec<WindowId> {
    top_level_windows(is_on_screen).unwrap_or_else(|e| {
        mosaico_core::log_debug!("Windows on screen not listed: {e}");
        Vec::new()
    })
}

/// Whether the window `hwnd` is on screen, as [`windows_on_screen`] has
/// it.
fn is_on_screen(hwnd: HWND) -> bool {
    on_screen(&Window::new(hwnd), std::process::id())
}

/// What the walk reads of a window to tell whether it is on screen, one
/// read at a time, so that the cheap ones can rule it out first.
trait Reads {
    fn visible(&self) -> bool;
    fn minimized(&self) -> bool;
    fn ex_style(&self) -> u32;
    fn process(&self) -> Option<u32>;
    fn cloaked(&self) -> bool;
    fn class_name(&self) -> String;
}

/// `IsWindowVisible`, `IsIconic`, the `GWL_EXSTYLE` of `GetWindowLongPtrW`,
/// `GetWindowThreadProcessId`, `DWMWA_CLOAKED` and `RealGetWindowClassW`.
impl Reads for Window {
    fn visible(&self) -> bool {
        self.is_visible()
    }

    fn minimized(&self) -> bool {
        self.is_minimized()
    }

    fn ex_style(&self) -> u32 {
        // SAFETY: GetWindowLongPtrW only reads the window's extended style.
        unsafe { GetWindowLongPtrW(self.hwnd(), GWL_EXSTYLE) as u32 }
    }

    fn process(&self) -> Option<u32> {
        self.process_id()
    }

    fn cloaked(&self) -> bool {
        self.is_cloaked()
    }

    fn class_name(&self) -> String {
        self.class().unwrap_or_default()
    }
}

/// Whether a window is on screen, from what `window` reads, when the
/// daemon, which draws the bars and the borders, runs as process `own`.
/// The cheapest reads come first, since most windows are not shown.
fn on_screen(window: &impl Reads, own: u32) -> bool {
    window.visible()
        && !window.minimized()
        && !lets_pointer_through(window.ex_style())
        && window.process() != Some(own)
        && !window.cloaked()
        && !is_shell(&window.class_name())
}

/// Whether a window with the extended style `ex_style` lets the pointer
/// through: a layered window that is also `WS_EX_TRANSPARENT`.
fn lets_pointer_through(ex_style: u32) -> bool {
    let through = WS_EX_LAYERED.0 | WS_EX_TRANSPARENT.0;
    ex_style & through == through
}

/// Whether `class` is the class of one of the shell's own surfaces.
fn is_shell(class: &str) -> bool {
    SHELL_CLASSES.contains(&class)
}

#[cfg(test)]
#[path = "on_screen_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "on_screen_decision_tests.rs"]
mod decision_tests;
