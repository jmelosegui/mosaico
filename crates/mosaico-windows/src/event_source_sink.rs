//! The hidden window that receives display and work-area broadcasts.

use std::sync::{Mutex, PoisonError};

use mosaico_core::{Error, Result};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassW, WNDCLASSW, WS_EX_TOOLWINDOW,
};
use windows::core::{PCWSTR, w};

use crate::error::OrPlatform;
use crate::event;

/// The name of the hidden window's class.
const CLASS_NAME: PCWSTR = w!("MosaicoEventSink");

/// Whether this process registered the class, so that a source started
/// again reuses it. A class of that name registered by anything else is
/// not ours, and its window procedure would not report the broadcasts.
static REGISTERED: Mutex<bool> = Mutex::new(false);

/// The hidden window of the event thread. Dropping it destroys the window.
pub(super) struct Sink(HWND);

impl Sink {
    /// Creates a hidden window that receives `WM_DISPLAYCHANGE` and
    /// `WM_SETTINGCHANGE`.
    ///
    /// Must NOT be a message-only window (`HWND_MESSAGE` parent) because
    /// those do not receive broadcast messages. Instead we create a regular
    /// hidden window with `WS_EX_TOOLWINDOW` to keep it out of the taskbar.
    pub(super) fn create() -> Result<Self> {
        register_class()?;
        // SAFETY: Creates a hidden popup window of the class registered
        // above. WS_EX_TOOLWINDOW: no taskbar entry. No WS_VISIBLE: the
        // window stays hidden. No HWND_MESSAGE parent: it receives
        // broadcast messages. An Ok handle is never null.
        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_TOOLWINDOW,
                CLASS_NAME,
                w!("MosaicoEventSink"),
                Default::default(),
                0,
                0,
                0,
                0,
                None,
                None,
                None,
                None,
            )
        }
        .or_platform("create the MosaicoEventSink window")?;
        Ok(Self(hwnd))
    }
}

impl Drop for Sink {
    fn drop(&mut self) {
        // SAFETY: DestroyWindow is called on the event thread with the
        // HWND that create made there; a Sink cannot leave that thread.
        unsafe {
            let _ = DestroyWindow(self.0);
        }
    }
}

/// Registers the hidden window's class, once per process.
fn register_class() -> Result<()> {
    let mut registered = REGISTERED.lock().unwrap_or_else(PoisonError::into_inner);
    if *registered {
        return Ok(());
    }
    let wc = WNDCLASSW {
        lpfnWndProc: Some(sink_proc),
        lpszClassName: CLASS_NAME,
        ..Default::default()
    };
    // SAFETY: RegisterClassW copies the class description, whose name is
    // a static string and whose window procedure is a static function.
    if unsafe { RegisterClassW(&wc) } == 0 {
        return Err(Error::platform(
            "register the MosaicoEventSink window class",
            windows::core::Error::from_win32(),
        ));
    }
    *registered = true;
    Ok(())
}

/// WNDPROC for the hidden window.
///
/// Catches `WM_DISPLAYCHANGE` and `WM_SETTINGCHANGE` (work-area changes)
/// and forwards them as events.
unsafe extern "system" fn sink_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if let Some(event) = event::translate_broadcast(msg, wparam.0) {
        super::send(event);
    }
    // SAFETY: DefWindowProcW is the default message handler for
    // messages not handled above. Required by the WNDPROC contract.
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}
