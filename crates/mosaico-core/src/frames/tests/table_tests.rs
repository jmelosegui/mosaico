//! The frame table against a probe of the test's own: windows that are
//! where the test puts them, and take the moves posted to them at once,
//! or keep them queued.

use crate::Rect;
use crate::id::wid;

use super::*;
use probe::{Probe, START, Window};

#[path = "table_limit_tests.rs"]
mod limit_tests;
#[path = "table_motion_tests.rs"]
mod motion_tests;
#[path = "table_probe.rs"]
mod probe;
#[path = "table_queue_late_tests.rs"]
mod queue_late_tests;
#[path = "table_queue_tests.rs"]
mod queue_tests;
#[path = "table_settle_tests.rs"]
mod settle_tests;
#[path = "table_view_order_tests.rs"]
mod view_order_tests;
#[path = "table_view_tests.rs"]
mod view_tests;

/// Window 1 at `x`, in the first frame of its motion when `first`.
fn frame(x: i32, first: bool) -> Frame {
    Frame {
        window: wid(1),
        rect: Rect { x, ..START },
        first,
    }
}

/// Window 1 at `x`.
fn at(x: i32) -> Rect {
    Rect { x, ..START }
}

/// Places one frame at the probe's time and returns what was found of
/// its window.
fn place(table: &mut FrameTable<i32>, probe: &mut Probe, frame: Frame) -> FrameOutcome {
    let mut outcomes = vec![FrameOutcome::Gone; 3];
    let now = probe.now;
    table.place(&[frame], probe, now, &mut outcomes);
    assert_eq!(outcomes.len(), 1, "{outcomes:?}");
    outcomes[0]
}

/// Milliseconds in the limit.
fn limit_ms() -> u64 {
    PENDING_LIMIT.as_millis() as u64
}

#[test]
fn a_window_that_takes_its_frames_gets_each_one_and_is_where_the_last_put_it() {
    // Arrange
    let mut table = FrameTable::default();
    let mut probe = Probe::new(true);

    // Act
    let first = place(&mut table, &mut probe, frame(120, true));
    let second = place(&mut table, &mut probe, frame(140, false));

    // Assert
    assert_eq!(first, FrameOutcome::Handled { reached: START });
    assert_eq!(second, FrameOutcome::Handled { reached: at(120) });
    assert_eq!(probe.posted, vec![(wid(1), at(120)), (wid(1), at(140))]);
}

#[test]
fn a_frame_the_window_cannot_take_is_not_posted_and_is_handled() {
    // Arrange -- the window moves in steps of 4: at 100, a frame at 102
    // would leave it where it is.
    let mut table = FrameTable::default();
    let mut probe = Probe::new(true);
    probe.pixel = 4;

    // Act
    let outcome = place(&mut table, &mut probe, frame(102, true));
    let again = place(&mut table, &mut probe, frame(102, false));

    // Assert
    assert_eq!(
        [outcome, again],
        [FrameOutcome::Handled { reached: START }; 2]
    );
    assert!(probe.posted.is_empty(), "{:?}", probe.posted);
}

#[test]
fn a_refused_post_holds_nothing_back() {
    // Arrange
    let mut table = FrameTable::default();
    let mut probe = Probe::new(false);
    probe.refuses = true;

    // Act
    let first = place(&mut table, &mut probe, frame(120, true));
    let second = place(&mut table, &mut probe, frame(140, false));

    // Assert -- tried again at once.
    assert_eq!(
        [first, second],
        [FrameOutcome::Handled { reached: START }; 2]
    );
    assert_eq!(probe.posted, vec![(wid(1), at(120)), (wid(1), at(140))]);
}
