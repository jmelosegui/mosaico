use std::time::Instant;

use crate::Rect;
use crate::id::wid;
use crate::motion::test_support::{flights, ms, spec};
use crate::motion::{Easing, Segment};

use super::super::Motions;
use super::Tick;

/// A table with one window moving 10 px right in a second, so most
/// frames find it where the last one left it.
fn creeping(t0: Instant) -> Motions {
    let mut motions = Motions::default();
    let (from, to) = (Rect::new(0, 0, 400, 400), Rect::new(10, 0, 400, 400));
    motions.retarget(wid(1), from, to, t0, spec(1000, Easing::Linear));
    motions
}

#[test]
fn frames_follow_the_clock_however_many_are_missed() {
    // Arrange
    let t0 = Instant::now();
    let (from, to) = (Rect::new(0, 0, 800, 600), Rect::new(1200, 100, 400, 500));
    let linear = spec(250, Easing::Linear);
    let path = Segment::eased(from, to, t0, linear);
    let (mut steady, mut missing) = (Motions::default(), Motions::default());
    steady.retarget(wid(1), from, to, t0, linear);
    missing.retarget(wid(1), from, to, t0, linear);

    // Act -- 60 frames a second, against three late ones.
    let steady_frames: Vec<_> = (1..=15)
        .map(|i| (i * 16, steady.frame(t0 + ms(i * 16))))
        .collect();
    let missing_frames: Vec<_> = [37, 101, 249]
        .map(|i| (i, missing.frame(t0 + ms(i))))
        .into();

    // Assert -- each frame has the window where it is at its own time.
    for (i, frame) in steady_frames.iter().chain(&missing_frames) {
        assert_eq!(
            flights(frame),
            vec![(wid(1), path.rect_at(t0 + ms(*i)))],
            "at {i} ms"
        );
    }
    for motions in [&mut steady, &mut missing] {
        assert_eq!(motions.frame(t0 + ms(250)).finished, vec![(wid(1), to)]);
    }
}

#[test]
fn a_frame_only_moves_the_windows_whose_rect_changed() {
    // Arrange
    let t0 = Instant::now();
    let mut motions = creeping(t0);

    // Act
    let at_start = motions.frame(t0);
    let moved = motions.frame(t0 + ms(100));
    let same = motions.frame(t0 + ms(116));
    let next = motions.frame(t0 + ms(200));

    // Assert -- not even to where it starts, which is where it is.
    assert_eq!(at_start, Tick::default());
    assert_eq!(flights(&moved), vec![(wid(1), Rect::new(1, 0, 400, 400))]);
    assert_eq!(same, Tick::default());
    assert_eq!(flights(&next), vec![(wid(1), Rect::new(2, 0, 400, 400))]);
}

#[test]
fn ending_a_motion_stops_its_frames() {
    // Arrange
    let t0 = Instant::now();
    let (left, right) = (Rect::new(0, 0, 400, 400), Rect::new(800, 0, 400, 400));
    let linear = spec(200, Easing::Linear);
    let mut motions = Motions::default();
    motions.retarget(wid(1), left, right, t0, linear);
    motions.retarget(wid(2), right, left, t0, linear);

    // Act
    let ended = motions.end(wid(1));

    // Assert
    assert_eq!(ended, Some(right));
    assert_eq!(motions.end(wid(1)), None, "it has ended already");
    assert_eq!(motions.end(wid(3)), None, "it never moved");
    let frame = motions.frame(t0 + ms(100));
    assert_eq!(flights(&frame), vec![(wid(2), Rect::new(400, 0, 400, 400))]);
    assert_eq!(motions.frame(t0 + ms(200)).finished, vec![(wid(2), left)]);
}

#[test]
fn clearing_the_table_stops_every_motion_and_gives_back_the_targets() {
    // Arrange
    let t0 = Instant::now();
    let mut motions = creeping(t0);
    let (from, to) = (Rect::new(0, 0, 400, 400), Rect::new(0, 400, 400, 400));
    motions.retarget(wid(2), from, to, t0, spec(200, Easing::default()));

    // Act
    let targets = motions.clear();

    // Assert -- for the caller to place each window at its target, as
    // shutdown does, or to leave it where it is, as a display change does.
    let crept = Rect::new(10, 0, 400, 400);
    assert_eq!(targets, vec![(wid(1), crept), (wid(2), to)]);
    assert!(motions.is_empty());
    assert!(!motions.is_moving(wid(1)) && !motions.is_moving(wid(2)));
    assert_eq!(motions.frame(t0 + ms(100)), Tick::default());
    assert_eq!(motions.frame(t0 + ms(1000)), Tick::default());
}
