use std::time::Instant;

use crate::Rect;
use crate::id::wid;
use crate::motion::test_support::{ms, spec};
use crate::motion::{Easing, Tick};

use super::*;

#[test]
fn a_window_that_is_not_moving_eases_from_where_it_is() {
    // Arrange
    let t0 = Instant::now();
    let (from, to) = (Rect::new(0, 0, 800, 600), Rect::new(400, 0, 400, 600));
    let quick = spec(180, Easing::default());
    let mut motions = Motions::default();

    // Act
    motions.retarget(wid(1), from, to, t0, quick);
    let frame = motions.frame(t0 + ms(60));

    // Assert
    let path = Segment::eased(from, to, t0, quick);
    assert_eq!(frame.in_flight, vec![(wid(1), path.rect_at(t0 + ms(60)))]);
    assert!(frame.finished.is_empty());
    assert_eq!(motions.target(wid(1)), Some(to));
    assert!(motions.is_moving(wid(1)) && !motions.is_empty());
}

#[test]
fn a_motion_ends_with_its_exact_target_and_leaves_the_table() {
    // Arrange
    let t0 = Instant::now();
    let to = Rect::new(517, 33, 641, 401);
    let mut motions = Motions::default();
    let quick = spec(180, Easing::default());
    motions.retarget(wid(1), Rect::new(10, 20, 300, 200), to, t0, quick);
    motions.frame(t0 + ms(90));

    // Act
    let frame = motions.frame(t0 + ms(180));

    // Assert
    let ended = Tick {
        in_flight: Vec::new(),
        finished: vec![(wid(1), to)],
    };
    assert_eq!(frame, ended);
    assert!(motions.is_empty() && !motions.is_moving(wid(1)));
    assert_eq!(motions.target(wid(1)), None);
    assert_eq!(motions.frame(t0 + ms(200)), Tick::default());
}

#[test]
fn windows_moved_together_move_on_the_same_frames_and_end_together() {
    // Arrange -- a swap.
    let t0 = Instant::now();
    let (left, right) = (Rect::new(0, 0, 640, 720), Rect::new(648, 0, 632, 720));
    let quick = spec(180, Easing::Linear);
    let mut motions = Motions::default();
    motions.retarget(wid(1), left, right, t0, quick);
    motions.retarget(wid(2), right, left, t0, quick);

    // Act
    let frames = [45, 90, 135, 180].map(|i| motions.frame(t0 + ms(i)));

    // Assert
    for frame in &frames[..3] {
        let moved: Vec<_> = frame.in_flight.iter().map(|&(w, _)| w).collect();
        assert_eq!(moved, [wid(1), wid(2)]);
    }
    assert_eq!(frames[3].finished, vec![(wid(1), right), (wid(2), left)]);
}

#[test]
fn a_motion_keeps_the_duration_it_started_with() {
    // Arrange -- a short motion, then a long one, as after a config
    // reload that lengthens transitions while the first still moves.
    let t0 = Instant::now();
    let (short, long) = (Rect::new(500, 0, 500, 500), Rect::new(0, 500, 500, 500));
    let mut motions = Motions::default();
    motions.retarget(
        wid(1),
        Rect::new(0, 0, 500, 500),
        short,
        t0,
        spec(100, Easing::default()),
    );
    motions.retarget(
        wid(2),
        Rect::new(0, 0, 500, 500),
        long,
        t0 + ms(10),
        spec(400, Easing::default()),
    );

    // Act
    let frame = motions.frame(t0 + ms(100));

    // Assert
    assert_eq!(frame.finished, vec![(wid(1), short)]);
    assert!(motions.is_moving(wid(2)));
    assert_eq!(motions.frame(t0 + ms(410)).finished, vec![(wid(2), long)]);
}
