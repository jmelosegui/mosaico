//! Dragging with the real pointer, for the drag and drop tests: the left
//! button pressed, the pointer moved in steps, and the button released, as
//! the user does, through `SendInput`.
//!
//! Positions are in the physical pixels the daemon works in, which the
//! test process uses once it has set itself up as the daemon does. A drag
//! that fails half way, as when the system refuses an input, lets go of the
//! button all the same, and every drag puts the pointer back where it was.

use std::thread;
use std::time::Duration;

use super::win32;

/// How long the system is given to handle each step of a drag.
const STEP: Duration = Duration::from_millis(30);

/// How many steps the pointer takes from where it presses to where it
/// lets go.
const STEPS: i32 = 12;

/// The pointer as a drag found it. Dropped, as the drag ends or as the
/// test fails half way, it lets go of the left button if the drag still
/// holds it down, and puts the pointer back where it was.
struct Held {
    /// Where the pointer was, if it could be read.
    back: Option<(i32, i32)>,
    /// Whether the drag holds the left button down.
    down: bool,
}

impl Held {
    fn new() -> Self {
        let mut point = win32::Point { x: 0, y: 0 };
        // SAFETY: GetCursorPos writes the pointer position on the stack.
        let read = unsafe { win32::GetCursorPos(&mut point) } != 0;
        let back = read.then_some((point.x, point.y));
        Self { back, down: false }
    }
}

impl Drop for Held {
    fn drop(&mut self) {
        if self.down {
            input(None, win32::MOUSEEVENTF_LEFTUP);
        }
        if let Some(back) = self.back {
            input(Some(back), win32::MOUSEEVENTF_MOVE);
        }
    }
}

/// Presses the left button at `from`, moves the pointer to `to` in steps
/// and releases it there, giving the system time to handle each step, then
/// puts the pointer back where it was.
pub fn drag(from: (i32, i32), to: (i32, i32)) {
    let mut held = Held::new();
    send(from, win32::MOUSEEVENTF_MOVE);
    held.down = true;
    send(from, win32::MOUSEEVENTF_LEFTDOWN);
    for step in 1..=STEPS {
        let at = |a: i32, b: i32| a + (b - a) * step / STEPS;
        send(
            (at(from.0, to.0), at(from.1, to.1)),
            win32::MOUSEEVENTF_MOVE,
        );
    }
    send(to, win32::MOUSEEVENTF_LEFTUP);
    held.down = false;
}

/// Sends one mouse input at `at`, with `flags`, fails the test when the
/// system refuses it, and waits a step.
fn send(at: (i32, i32), flags: u32) {
    assert!(input(Some(at), flags), "the system refused the input");
    thread::sleep(STEP);
}

/// Sends one mouse input with `flags`: at `at` on the virtual desktop, or
/// where the pointer is without one. Says whether the system took it.
fn input(at: Option<(i32, i32)>, flags: u32) -> bool {
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
            mouse_data: 0,
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
