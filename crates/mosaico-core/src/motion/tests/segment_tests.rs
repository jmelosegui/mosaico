use std::time::{Duration, Instant};

use crate::Rect;
use crate::motion::test_support::{ms, spec};
use crate::motion::{Easing, Velocity};

use super::*;

#[path = "hermite_tests.rs"]
mod hermite_tests;

#[path = "hermite_limit_tests.rs"]
mod hermite_limit_tests;

#[path = "segment_speed_tests.rs"]
mod segment_speed_tests;

#[path = "segment_gap_tests.rs"]
mod segment_gap_tests;

#[test]
fn an_eased_segment_starts_where_it_was_and_ends_exactly_at_its_target() {
    // Arrange -- odd sizes, so that nothing comes out even by chance.
    let t0 = Instant::now();
    let (from, to) = (Rect::new(10, 20, 300, 200), Rect::new(517, 33, 641, 401));
    let segment = Segment::eased(from, to, t0 + ms(50), spec(180, Easing::default()));

    // Act
    let before = segment.rect_at(t0);
    let start = segment.rect_at(t0 + ms(50));
    let midway = segment.rect_at(t0 + ms(140));
    let end = segment.rect_at(t0 + ms(230));
    let later = segment.rect_at(t0 + ms(10_000));

    // Assert
    assert_eq!((before, start), (from, from));
    assert!(midway != from && midway != to, "{midway:?}");
    assert_eq!((end, later), (to, to));
    assert!(!segment.is_done(t0 + ms(229)));
    assert!(segment.is_done(t0 + ms(230)));
    assert_eq!((segment.target(), segment.end()), (to, t0 + ms(230)));
}

#[test]
fn an_eased_segment_moves_along_its_easing() {
    // Arrange
    let t0 = Instant::now();
    let (from, to) = (Rect::new(0, 0, 100, 100), Rect::new(1000, 0, 100, 100));
    let linear = Segment::eased(from, to, t0, spec(1000, Easing::Linear));
    let quad = Segment::eased(from, to, t0, spec(1000, Easing::EaseOutQuad));

    // Act / Assert
    assert_eq!(linear.rect_at(t0 + ms(250)), Rect::new(250, 0, 100, 100));
    assert_eq!(linear.rect_at(t0 + ms(500)), Rect::new(500, 0, 100, 100));
    assert_eq!(quad.rect_at(t0 + ms(500)), Rect::new(750, 0, 100, 100));
}

#[test]
fn a_rect_never_gets_narrower_or_shorter_than_a_pixel() {
    // Arrange -- shrinking to nothing, as a target worked out for a
    // monitor too small for its gaps can be.
    let t0 = Instant::now();
    let (from, to) = (Rect::new(0, 0, 100, 50), Rect::new(50, 25, 0, 0));
    let eased = Segment::eased(from, to, t0, spec(200, Easing::default()));
    let inwards = Velocity {
        left: 400.0,
        top: 200.0,
        right: -400.0,
        bottom: -200.0,
    };
    let hermite = Segment::hermite(from, inwards, to, t0, ms(200));

    for segment in [eased, hermite] {
        // Act
        let rects: Vec<Rect> = (0..200).map(|i| segment.rect_at(t0 + ms(i))).collect();

        // Assert -- down to a pixel on the way, and exactly the target at
        // the end.
        assert!(
            rects.iter().all(|r| r.width >= 1 && r.height >= 1),
            "{rects:?}"
        );
        assert!(rects.iter().any(|r| r.width == 1), "{rects:?}");
        assert_eq!(segment.rect_at(t0 + ms(200)), to);
    }
}

#[test]
fn a_segment_that_lasts_no_time_is_done_at_once() {
    // Arrange
    let t0 = Instant::now();
    let (from, to) = (Rect::new(0, 0, 100, 100), Rect::new(300, 0, 100, 100));
    let moving = Velocity {
        left: 900.0,
        right: 900.0,
        ..Velocity::ZERO
    };
    let segments = [
        Segment::eased(from, to, t0, spec(0, Easing::default())),
        Segment::hermite(from, moving, to, t0, Duration::ZERO),
    ];

    for segment in segments {
        // Act / Assert
        assert!(segment.is_done(t0));
        assert_eq!(segment.rect_at(t0), to);
        assert_eq!(segment.velocity_at(t0), Velocity::ZERO);
    }
}
