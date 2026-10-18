//! One mouse input at a time, for the pointer helpers: sent through
//! `SendInput`, at a point of the virtual desktop or where the pointer is.

use std::thread;
use std::time::Duration;

use super::super::win32;

/// How long the system is given to handle each input, which `SendInput`
/// only queues.
pub(super) const STEP: Duration = Duration::from_millis(30);

/// `MOUSEEVENTF_WHEEL`: the wheel turned, by as much as the input's data
/// says.
const MOUSEEVENTF_WHEEL: u32 = 0x0800;

/// Sends one mouse input at `at`, with `flags`, fails the test when the
/// system refuses it, and waits a step.
pub(super) fn send(at: (i32, i32), flags: u32) {
    assert!(input(Some(at), flags, 0), "the system refused the input");
    thread::sleep(STEP);
}

/// Turns the wheel where the pointer is by `delta`, in 120ths of a notch,
/// positive away from the user, fails the test when the system refuses
/// it, and waits a step.
pub(super) fn turn(delta: i32) {
    let sent = input(None, MOUSEEVENTF_WHEEL, delta as u32);
    assert!(sent, "the system refused the input");
    thread::sleep(STEP);
}

/// Sends one mouse input with `flags` and `data`: at `at` on the virtual
/// desktop, or where the pointer is without one. Says whether the system
/// took it.
pub(super) fn input(at: Option<(i32, i32)>, flags: u32, data: u32) -> bool {
    let size = std::mem::size_of::<win32::Input>() as i32;
    let placed = win32::MOUSEEVENTF_ABSOLUTE | win32::MOUSEEVENTF_VIRTUALDESK;
    // SAFETY: GetSystemMetrics only reads system values, and SendInput
    // reads one INPUT from a value on the stack, of the size it is told.
    let sent = unsafe {
        let metric = |index| win32::GetSystemMetrics(index);
        let (x, w) = (
            metric(win32::SM_XVIRTUALSCREEN),
            metric(win32::SM_CXVIRTUALSCREEN),
        );
        let (y, h) = (
            metric(win32::SM_YVIRTUALSCREEN),
            metric(win32::SM_CYVIRTUALSCREEN),
        );
        let (dx, dy, flags) = match at {
            Some(at) => (
                normalized(at.0 - x, w),
                normalized(at.1 - y, h),
                flags | placed,
            ),
            None => (0, 0, flags),
        };
        let mi = win32::MouseInput {
            dx,
            dy,
            mouse_data: data,
            flags,
            time: 0,
            extra_info: 0,
        };
        let input = win32::Input {
            kind: win32::INPUT_MOUSE,
            mi,
        };
        win32::SendInput(1, &input, size)
    };
    sent == 1
}

/// `offset` pixels into an axis `extent` pixels long, as `SendInput` takes
/// it: 0 to 65535 across the virtual desktop.
fn normalized(offset: i32, extent: i32) -> i32 {
    let span = i64::from((extent - 1).max(1));
    (i64::from(offset) * 65535 / span) as i32
}
