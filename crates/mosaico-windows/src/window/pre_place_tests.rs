use std::sync::mpsc;
use std::time::Duration;

use mosaico_core::{WindowControl, WindowId};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassW, WNDCLASSW, WS_EX_TOOLWINDOW,
    WS_POPUP, WS_VISIBLE,
};
use windows::core::PCWSTR;

use super::*;
use crate::window_control::Win32WindowControl;

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
fn pre_place_flags_post_instead_of_sending() {
    // Arrange / Act / Assert -- the asynchronous flag is the whole
    // point of the pre placement move. Without it the call is
    // delivered synchronously and a window whose thread never pumps
    // blocks the daemon for good.
    assert_ne!(PRE_PLACE_FLAGS.0 & SWP_ASYNCWINDOWPOS.0, 0);
}

/// Creates a window on a thread that never pumps messages, which is
/// what hidden helper windows from Qt, Electron and WinForms do.
///
/// Returns the handle plus a sender that tells the thread to destroy
/// the window, which has to happen on the thread that created it.
/// Returns `None` when a window cannot be created at all, so the test
/// skips rather than fails on a machine with no interactive desktop.
fn window_owned_by_a_thread_that_never_pumps()
-> Option<(usize, mpsc::Sender<()>, std::thread::JoinHandle<()>)> {
    let class: Vec<u16> = "MosaicoNeverPumpsTestWindow\0".encode_utf16().collect();
    let (ready_tx, ready_rx) = mpsc::channel::<usize>();
    let (stop_tx, stop_rx) = mpsc::channel::<()>();

    let handle = std::thread::spawn(move || {
        // SAFETY: registers a class and creates one window, both with
        // valid arguments. The window is destroyed on this same thread
        // below, which is the only thread allowed to destroy it.
        let hwnd = unsafe {
            let wc = WNDCLASSW {
                lpfnWndProc: Some(test_wnd_proc),
                lpszClassName: PCWSTR(class.as_ptr()),
                ..Default::default()
            };
            RegisterClassW(&wc);
            CreateWindowExW(
                WS_EX_TOOLWINDOW,
                PCWSTR(class.as_ptr()),
                PCWSTR::null(),
                WS_POPUP | WS_VISIBLE,
                0,
                0,
                1,
                1,
                None,
                None,
                None,
                None,
            )
        };
        let hwnd = hwnd.unwrap_or_default();
        let _ = ready_tx.send(hwnd.0 as usize);
        // Park without ever calling GetMessage or PeekMessage. This is
        // the condition the fix exists for.
        let _ = stop_rx.recv();
        if !hwnd.0.is_null() {
            // SAFETY: destroying a window this thread created.
            unsafe {
                let _ = DestroyWindow(hwnd);
            }
        }
    });

    match ready_rx.recv_timeout(Duration::from_secs(5)) {
        Ok(raw) if raw != 0 => Some((raw, stop_tx, handle)),
        _ => {
            let _ = stop_tx.send(());
            let _ = handle.join();
            None
        }
    }
}

#[test]
fn pre_place_returns_even_when_the_owner_never_pumps() {
    // Arrange -- a window whose thread will never process messages.
    let Some((raw, stop_tx, handle)) = window_owned_by_a_thread_that_never_pumps() else {
        // No interactive desktop available, nothing to assert.
        return;
    };
    // Act -- run the move on its own thread so that a regression
    // fails this test instead of hanging it forever. A synchronous
    // SetWindowPos against this window never returns at all.
    let (done_tx, done_rx) = mpsc::channel::<()>();
    let mover = std::thread::spawn(move || {
        let window = WindowId::from_raw(raw as u64);
        let _ = Win32WindowControl.pre_place(window, Rect::new(20, 20, 200, 200));
        let _ = done_tx.send(());
    });
    let returned = done_rx.recv_timeout(Duration::from_secs(2)).is_ok();

    // Cleanup before asserting so a failure cannot leak the window.
    let _ = stop_tx.send(());
    let _ = handle.join();
    if returned {
        let _ = mover.join();
    }

    // Assert
    assert!(
        returned,
        "pre_place did not return within 2s against a window whose thread never pumps, \
         which means the move was sent synchronously instead of posted"
    );
}
