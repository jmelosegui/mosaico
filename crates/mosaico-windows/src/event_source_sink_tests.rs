//! The hidden window's class, through a real start of the event source.

use std::sync::mpsc;

use mosaico_core::event::EventSource;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{DefWindowProcW, RegisterClassW, WNDCLASSW};

use crate::event_source::WindowsEventSource;

use super::CLASS_NAME;

/// Default handler, required because `WNDCLASSW` wants a
/// `extern "system"` function pointer rather than the raw import.
unsafe extern "system" fn test_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // SAFETY: DefWindowProcW is the default handler required by WNDPROC.
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

#[test]
fn a_missing_hidden_window_is_reported_to_the_caller() {
    // Arrange -- review of #97: this process already has a class of that
    // name, so the event source cannot register the hidden window's.
    let class = WNDCLASSW {
        lpfnWndProc: Some(test_wnd_proc),
        lpszClassName: CLASS_NAME,
        ..Default::default()
    };
    // SAFETY: registers a class with a valid name and window procedure.
    let atom = unsafe { RegisterClassW(&class) };
    assert_ne!(atom, 0, "the test could not register the class");
    let (events, _received) = mpsc::channel();

    // Act
    let Ok(watching) = WindowsEventSource.start(events) else {
        // No desktop to hook into, nothing to assert.
        return;
    };
    let error = watching.display_error().map(ToString::to_string);
    watching.stop();

    // Assert -- the source runs, and the caller can say what is missing.
    assert!(
        error
            .as_deref()
            .is_some_and(|e| e.starts_with("could not register the MosaicoEventSink window class")),
        "{error:?}"
    );
}
