use std::sync::mpsc;

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{DefWindowProcW, RegisterClassW, WNDCLASSW};
use windows::core::w;

use super::start;

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
fn a_missing_event_sink_window_is_reported_to_the_caller() {
    // Arrange -- this process already has a class of that name, so the
    // event loop cannot register the event sink's.
    let class = WNDCLASSW {
        lpfnWndProc: Some(test_wnd_proc),
        lpszClassName: w!("MosaicoEventSink"),
        ..Default::default()
    };
    // SAFETY: registers a class with a valid name and window procedure.
    let atom = unsafe { RegisterClassW(&class) };
    assert_ne!(atom, 0, "the test could not register the class");
    let (event_tx, _events) = mpsc::channel();

    // Act
    let Ok(event_loop) = start(event_tx) else {
        // No desktop to hook into, nothing to assert.
        return;
    };
    let error = event_loop.sink_error().map(ToString::to_string);
    event_loop.stop();

    // Assert -- the loop runs, and the caller can say what is missing.
    assert!(
        error
            .as_deref()
            .is_some_and(|e| e.starts_with("could not register the MosaicoEventSink window class")),
        "{error:?}"
    );
}
