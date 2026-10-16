use std::time::Instant;

use crate::Rect;
use crate::id::wid;
use crate::motion::test_support::{ms, spec};
use crate::motion::{Easing, Segment};

use super::Motions;

#[path = "retarget_box_tests.rs"]
mod retarget_box_tests;

/// Where a window that is moving is said to be, which the table ignores.
const ANYWHERE: Rect = Rect {
    x: 0,
    y: 0,
    width: 1,
    height: 1,
};

#[test]
fn sending_a_window_where_it_is_already_going_changes_nothing() {
    // Arrange -- the same target again later and with another spec, as
    // when a config reload lays the windows out again.
    let t0 = Instant::now();
    let (from, to) = (Rect::new(0, 0, 800, 600), Rect::new(400, 0, 400, 600));
    let quick = spec(180, Easing::default());
    let (mut once, mut twice) = (Motions::default(), Motions::default());
    once.retarget(wid(1), from, to, t0, quick);
    twice.retarget(wid(1), from, to, t0, quick);

    // Act
    twice.retarget(wid(1), ANYWHERE, to, t0 + ms(50), spec(900, Easing::Linear));

    // Assert
    for at in [60, 120, 179].map(|i| t0 + ms(i)) {
        assert_eq!(twice.frame(at), once.frame(at));
    }
    assert_eq!(twice.frame(t0 + ms(180)).finished, vec![(wid(1), to)]);
}

#[test]
fn a_new_target_halfway_goes_on_from_where_the_window_is_as_fast_as_it_moves() {
    // Arrange -- 2000 px a second to the right, then halfway a target
    // further right and lower.
    let t0 = Instant::now();
    let linear = spec(1000, Easing::Linear);
    let turned = Rect::new(3000, 300, 500, 500);
    let mut motions = Motions::default();
    motions.retarget(
        wid(1),
        Rect::new(0, 0, 500, 500),
        Rect::new(2000, 0, 500, 500),
        t0,
        linear,
    );
    let at = t0 + ms(500);
    let before = [at - ms(10), at].map(|t| motions.rect_at(wid(1), t).unwrap());

    // Act
    motions.retarget(wid(1), ANYWHERE, turned, at, linear);

    // Assert -- no jump, and 20 px in the 10 ms either side of the turn.
    let after = [at, at + ms(10)].map(|t| motions.rect_at(wid(1), t).unwrap());
    assert_eq!(after[0], before[1]);
    assert_eq!(before[1].x - before[0].x, 20, "{before:?}");
    assert_eq!(after[1].x - after[0].x, 20, "{after:?}");
    assert!(
        motions.frame(at + ms(999)).finished.is_empty(),
        "the full duration"
    );
    assert_eq!(
        motions.frame(at + ms(1000)).finished,
        vec![(wid(1), turned)]
    );
}

#[test]
fn a_new_target_before_the_window_has_moved_starts_over_from_where_it_is() {
    // Arrange -- two placements of one window in the same step.
    let t0 = Instant::now();
    let (from, second) = (Rect::new(0, 0, 800, 600), Rect::new(0, 300, 800, 300));
    let quick = spec(180, Easing::default());
    let mut motions = Motions::default();
    motions.retarget(wid(1), from, Rect::new(400, 0, 400, 600), t0, quick);

    // Act
    motions.retarget(wid(1), ANYWHERE, second, t0, quick);

    // Assert -- as if the first had never been asked for.
    let alone = Segment::eased(from, second, t0, quick);
    for at in [30, 90, 150].map(|i| t0 + ms(i)) {
        assert_eq!(motions.rect_at(wid(1), at), Some(alone.rect_at(at)));
    }
}

#[test]
fn a_window_sent_back_the_way_it_came_starts_as_fast_as_a_new_motion() {
    // Arrange -- a third of the way from x = 0 to 800, it is sent back.
    // Every edge moves away from the new target, so it keeps none of its
    // speed.
    let t0 = Instant::now();
    let quick = spec(180, Easing::default());
    let (start, ahead) = (Rect::new(0, 0, 800, 600), Rect::new(800, 0, 800, 600));
    let mut motions = Motions::default();
    motions.retarget(wid(1), start, ahead, t0, quick);
    let at = t0 + ms(60);
    let turning = motions.rect_at(wid(1), at).unwrap();

    // Act
    motions.retarget(wid(1), ANYWHERE, start, at, quick);

    // Assert -- along the path of a motion that starts where it turns.
    let fresh = Segment::eased(turning, start, at, quick);
    for after in [1, 10, 30, 90, 179].map(|i| at + ms(i)) {
        assert_eq!(motions.rect_at(wid(1), after), Some(fresh.rect_at(after)));
    }
}
