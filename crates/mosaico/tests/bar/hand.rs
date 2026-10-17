//! Where the pointer shows the hand on the bar, over the items a click
//! does something on, as the user sees it: read with the real pointer,
//! along the bar to find them, and where a click or the wheel is to go.

use std::time::{Duration, Instant};

use super::strip::Bar;
use super::{bar_win32, pointer};

#[path = "hand_shape.rs"]
mod shape;

use shape::shows_hand;

/// How far apart, in pixels, the pointer's shape is read along the bar:
/// well under the gap the tests' bar leaves between two items.
const STRIDE: usize = 4;

/// How long the bar is given to draw an item again.
const REDRAWN: Duration = Duration::from_secs(5);

/// Where the pointer shows the hand along the bar: `left` is the first x
/// read that shows it, and `right` the first after it that does not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Run {
    pub left: i32,
    pub right: i32,
}

impl Run {
    /// Half way between its ends.
    pub fn middle(self) -> i32 {
        (self.left + self.right) / 2
    }
}

/// The first `count` runs where the pointer shows the hand half way down
/// `bar`, left to right from `from`, or as many as there are.
pub fn runs(bar: &Bar, from: i32, count: usize) -> Vec<Run> {
    let mut runs = Vec::new();
    let mut open = None;
    let points = (from..bar.right).step_by(STRIDE).map(|x| bar.at(x));
    pointer::visit(points, |at| {
        match (open, shows_hand(bar, at)) {
            (None, true) => open = Some(at.0),
            (Some(left), false) => {
                runs.push(Run { left, right: at.0 });
                open = None;
            }
            _ => {}
        }
        runs.len() < count
    });
    runs
}

/// How wide the run that starts at `left` is: how far right of it, read
/// as [`runs`] reads, the pointer first does not show the hand. Fails the
/// test when the pointer does not show the hand at `left`.
pub fn width_at(bar: &Bar, left: i32) -> i32 {
    let mut right = left;
    let points = (left..bar.right).step_by(STRIDE).map(|x| bar.at(x));
    pointer::visit(points, |at| {
        right = at.0;
        shows_hand(bar, at)
    });
    assert!(right > left, "no hand at x = {left} on the bar {bar:?}");
    right - left
}

/// How wide the run that starts at `left` is once it is not `before`, as
/// when the bar has drawn its item again: once two walks in a row find the
/// same other width, so a walk made as the bar drew does not count. Fails
/// the test when no two walks in a row have found another width by the
/// time the bar had long enough to draw it again.
pub fn width_after(bar: &Bar, left: i32, before: i32) -> i32 {
    let deadline = Instant::now() + REDRAWN;
    let mut last = before;
    loop {
        let width = width_at(bar, left);
        if width != before && width == last {
            return width;
        }
        assert!(
            Instant::now() < deadline,
            "the run at x = {left} on the bar {bar:?} found no other width than \
             {before} in two walks in a row: {last}, then {width}"
        );
        last = width;
    }
}

/// Whether a click at `at`, where the pointer is, reaches an item of `bar`
/// and nothing else, and so does the wheel while it goes under the
/// pointer: the pointer shows the hand there, as the bar sets it, and the
/// bar is the window under it, with nothing in front of it. That is asked
/// again last, right before the press or the turn.
pub fn takes(bar: &Bar) -> impl FnOnce((i32, i32)) -> bool + '_ {
    move |at| shows_hand(bar, at) && shape::window_under(at) == bar.hwnd
}

/// Whether the system sends the wheel to the window under the pointer, as
/// "Scroll inactive windows when I hover over them" has it, rather than to
/// the window that has focus, which the bar never has.
pub fn wheel_goes_under_the_pointer() -> bool {
    let mut routing = 0u32;
    let get = bar_win32::SPI_GETMOUSEWHEELROUTING;
    // SAFETY: SystemParametersInfoW writes the routing into the value.
    let read = unsafe { bar_win32::SystemParametersInfoW(get, 0, &mut routing, 0) } != 0;
    read && routing == bar_win32::MOUSEWHEEL_ROUTING_MOUSE_POS
}
