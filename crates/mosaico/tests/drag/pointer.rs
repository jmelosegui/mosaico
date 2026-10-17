//! Driving the real pointer, for the desktop tests: a drag with the left
//! button, a click, a turn of the wheel, and a walk over points one after
//! another, as the user makes them, through `SendInput`.
//!
//! Positions are in the physical pixels the daemon works in, which the
//! test process uses once it has set itself up as the daemon does. Every
//! helper puts the pointer back where it was as it ends, and one that fails
//! half way, as when the system refuses an input, lets go of the button all
//! the same. [`Resting`] puts it back where a test found it. A click or a
//! turn of the wheel only happens where the test says the pointer may make
//! it, once the pointer is there.

// Each suite uses some of these helpers.
#![allow(dead_code)]

use std::thread;

use super::win32;

#[path = "pointer_input.rs"]
mod input;

use input::{input, send};

/// How many steps the pointer takes from where it presses to where it
/// lets go.
const STEPS: i32 = 12;

/// One notch of the wheel, as `SendInput` counts its turns.
const NOTCH: i32 = 120;

/// The pointer as a helper found it. Dropped, as the helper ends or as
/// the test fails half way, it lets go of the left button if the helper
/// still holds it down, and puts the pointer back where it was, giving the
/// system a step to get it there before anything reads it again.
struct Held {
    /// Where the pointer was, if it could be read.
    back: Option<(i32, i32)>,
    /// Whether the helper holds the left button down.
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
            input(None, win32::MOUSEEVENTF_LEFTUP, 0);
        }
        if let Some(back) = self.back {
            input(Some(back), win32::MOUSEEVENTF_MOVE, 0);
            thread::sleep(input::STEP);
        }
    }
}

/// Where the pointer is as a test begins. Dropped, as the test ends or
/// fails, it puts the pointer back there, whatever the helpers did with
/// it meanwhile.
#[must_use = "dropping it puts the pointer back straight away"]
pub struct Resting(Held);

impl Resting {
    /// Where the pointer is now.
    pub fn here() -> Self {
        Self(Held::new())
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

/// Moves the pointer to `at`, and presses and releases the left button
/// there once `ready`, asked with the pointer there, says the pointer may
/// click there. Fails the test, without pressing, when it may not. Puts
/// the pointer back where it was.
pub fn click(at: (i32, i32), ready: impl FnOnce((i32, i32)) -> bool) {
    let mut held = Held::new();
    approach(at, ready);
    held.down = true;
    send(at, win32::MOUSEEVENTF_LEFTDOWN);
    send(at, win32::MOUSEEVENTF_LEFTUP);
    held.down = false;
}

/// Turns the wheel by `notches` at `at`, down, toward the user, for a
/// positive count, the way the bar counts them, once `ready` says the
/// pointer may, as [`click`] asks. Puts the pointer back where it was.
pub fn wheel(at: (i32, i32), notches: i32, ready: impl FnOnce((i32, i32)) -> bool) {
    let _held = Held::new();
    approach(at, ready);
    input::turn(-notches * NOTCH);
}

/// Moves the pointer to each of `points` in turn, giving the system time
/// to handle each move, and calls `at_each` there, until it says to stop.
/// Nothing is pressed. Puts the pointer back where it was.
pub fn visit(
    points: impl IntoIterator<Item = (i32, i32)>,
    mut at_each: impl FnMut((i32, i32)) -> bool,
) {
    let _held = Held::new();
    for at in points {
        send(at, win32::MOUSEEVENTF_MOVE);
        if !at_each(at) {
            break;
        }
    }
}

/// Moves the pointer to `at`, and fails the test before anything is
/// pressed or turned there unless `ready` says the pointer may act there.
fn approach(at: (i32, i32), ready: impl FnOnce((i32, i32)) -> bool) {
    send(at, win32::MOUSEEVENTF_MOVE);
    assert!(ready(at), "the pointer may not press or scroll at {at:?}");
}
