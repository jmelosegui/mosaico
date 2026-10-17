//! The pointer's shape at a point of the bar, as the bar sets it for that
//! point, to tell whether the pointer shows the hand there.

use std::thread;
use std::time::{Duration, Instant};

use super::super::strip::Bar;
use super::super::{bar_win32, win32};

/// How long the bar's window is given to answer, in milliseconds.
const ANSWER_MS: u32 = 1000;

/// How long the system is given to put the pointer where it was sent.
const ARRIVED: Duration = Duration::from_secs(1);

/// How far, in pixels, the pointer may land from where it was sent, as
/// `SendInput` rounds the point.
const SLACK: i32 = 2;

/// Whether the pointer shows the hand at `at`, as the bar sets its shape
/// for that point.
///
/// Once the system has the pointer at `at`, over the bar, the bar is sent
/// the `WM_SETCURSOR` the system sends it as the pointer moves over it.
/// The bar answers by setting the shape for where the pointer is, which it
/// reads itself, so the shape read once it has answered is the one it set
/// for `at`, whatever else is queued for its window, and however late it
/// handles the one the system sent. Fails the test when the pointer is not
/// at `at`, or leaves it meanwhile, when the bar is not the window under
/// it, and when the bar does not answer that it set the shape.
pub(super) fn shows_hand(bar: &Bar, at: (i32, i32)) -> bool {
    arrive(at);
    let under = window_under(at) == bar.hwnd;
    assert!(under, "the bar {bar:?} is not the window under {at:?}");
    let (message, over) = (bar_win32::WM_SETCURSOR, bar_win32::MOVED_OVER_CLIENT);
    let mut answer = 0usize;
    // SAFETY: WM_SETCURSOR carries no pointer, the bar only sets the
    // pointer's shape for it, and the answer is written on the stack,
    // within the timeout whatever the window does.
    let answered = unsafe {
        let (hwnd, wparam, flags) = (bar.hwnd, bar.hwnd as usize, win32::SMTO_ABORTIFHUNG);
        win32::SendMessageTimeoutW(hwnd, message, wparam, over, flags, ANSWER_MS, &mut answer)
    } != 0;
    assert!(
        answered && answer == 1,
        "the bar {bar:?} did not set the pointer's shape (answered: {answered}, with {answer})"
    );
    let (hand, seen) = cursor();
    assert!(near(seen, at), "the pointer went from {at:?} to {seen:?}");
    hand
}

/// Waits until the system has the pointer at `at`, where it was sent, and
/// fails the test when it does not in time, as when the mouse is moved
/// while the tests run.
fn arrive(at: (i32, i32)) {
    let deadline = Instant::now() + ARRIVED;
    loop {
        let (_, seen) = cursor();
        if near(seen, at) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "the pointer is at {seen:?}, not {at:?}"
        );
        thread::sleep(Duration::from_millis(5));
    }
}

/// The window under the pointer at (`x`, `y`), the one a click there goes
/// to.
pub(super) fn window_under((x, y): (i32, i32)) -> isize {
    // SAFETY: a read-only query about a point of the screen.
    unsafe { bar_win32::WindowFromPoint(win32::Point { x, y }) }
}

/// Whether `seen` is within `SLACK` of `at` on both axes.
fn near(seen: (i32, i32), at: (i32, i32)) -> bool {
    (seen.0 - at.0).abs() <= SLACK && (seen.1 - at.1).abs() <= SLACK
}

/// Whether the pointer shows the hand, and where it is, as the system has
/// them now.
fn cursor() -> (bool, (i32, i32)) {
    let size = size_of::<bar_win32::CursorInfo>() as u32;
    let mut info = bar_win32::CursorInfo {
        size,
        ..Default::default()
    };
    // SAFETY: LoadCursorW without a module gives one of the system's shared
    // cursors, the hand the bar shows, and GetCursorInfo writes into the
    // value on the stack, whose size it is told.
    let (hand, read) = unsafe {
        let hand = bar_win32::LoadCursorW(0, bar_win32::IDC_HAND);
        (hand, bar_win32::GetCursorInfo(&mut info) != 0)
    };
    assert!(read, "the pointer could not be read");
    (hand != 0 && info.cursor == hand, (info.x, info.y))
}
