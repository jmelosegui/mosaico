use std::time::Instant;

use crate::Rect;
use crate::motion::test_support::{ms, spec};
use crate::motion::{Easing, Velocity};

use super::super::Segment;

/// The fastest any edge moves at `v`.
fn fastest(v: Velocity) -> f64 {
    v.edges().iter().map(|e| e.abs()).fold(0.0, f64::max)
}

/// How far `a` is from `b`, as a fraction of how fast `b` is.
fn relative_gap(a: Velocity, b: Velocity) -> f64 {
    let (a, b) = (a.edges(), b.edges());
    let gap = (0..4).map(|i| (a[i] - b[i]).abs()).fold(0.0, f64::max);
    gap / fastest(Velocity::from_edges(b)).max(1.0)
}

#[test]
fn a_hermite_segment_leaves_with_its_velocity_and_comes_to_rest_at_its_target() {
    // Arrange
    let t0 = Instant::now();
    let (from, to) = (Rect::new(0, 0, 400, 300), Rect::new(600, 100, 300, 300));
    let velocity = Velocity {
        left: 2000.0,
        top: 300.0,
        right: 1500.0,
        bottom: 0.0,
    };
    let segment = Segment::hermite(from, velocity, to, t0, ms(200));

    // Act
    let leaving = segment.velocity_at(t0);
    let arriving = segment.velocity_at(t0 + ms(199));

    // Assert
    assert_eq!(segment.rect_at(t0), from);
    assert_eq!(leaving, velocity);
    assert!(fastest(arriving) < 0.05 * fastest(leaving), "{arriving:?}");
    assert_eq!(segment.velocity_at(t0 + ms(200)), Velocity::ZERO);
    assert_eq!(segment.rect_at(t0 + ms(200)), to);
}

#[test]
fn a_hermite_segment_continues_another_without_a_jump_in_place_or_speed() {
    // Arrange -- a third of the way to the right, the target moves
    // further right and down, and grows.
    let t0 = Instant::now();
    let first = Segment::eased(
        Rect::new(0, 0, 500, 500),
        Rect::new(1200, 0, 500, 500),
        t0,
        spec(180, Easing::default()),
    );
    let at = t0 + ms(60);
    let (rect, velocity) = (first.rect_at(at), first.velocity_at(at));

    // Act
    let second = Segment::hermite(rect, velocity, Rect::new(1600, 400, 700, 500), at, ms(180));

    // Assert -- the same place and speed when it takes over, and speeds
    // a tenth of a millisecond either side differ only as much as the
    // two curves bend in that time.
    assert_eq!(second.rect_at(at), rect);
    assert_eq!(second.velocity_at(at), velocity);
    let before = first.velocity_at(at - ms(1) / 10);
    let after = second.velocity_at(at + ms(1) / 10);
    assert!(
        relative_gap(after, before) < 0.01,
        "{before:?} then {after:?}"
    );
}

#[test]
fn the_velocity_is_how_fast_the_rect_moves() {
    // Arrange
    let t0 = Instant::now();
    let (from, to) = (Rect::new(-50, 20, 640, 480), Rect::new(900, 300, 320, 700));
    let velocity = Velocity {
        left: 3000.0,
        top: 800.0,
        right: 1500.0,
        bottom: 200.0,
    };
    let segments = [
        Segment::eased(from, to, t0, spec(250, Easing::EaseInOutCubic)),
        Segment::hermite(from, velocity, to, t0, ms(250)),
    ];

    for segment in segments {
        for at in [10, 60, 125, 190, 240].map(|i| t0 + ms(i)) {
            // Act -- where the edges are a microsecond either side.
            let step = ms(1) / 1000;
            let (behind, ahead) = (
                segment.position_at(at - step),
                segment.position_at(at + step),
            );
            let numeric = Velocity::from_edges(std::array::from_fn(|i| {
                (ahead[i] - behind[i]) / (2.0 * step.as_secs_f64())
            }));

            // Assert
            let analytic = segment.velocity_at(at);
            assert!(
                relative_gap(analytic, numeric) < 1e-4,
                "{analytic:?} {numeric:?}"
            );
        }
    }
}

impl Segment {
    /// Where the edges are at `now` in fractions of a pixel, while the
    /// segment runs.
    fn position_at(&self, now: Instant) -> [f64; 4] {
        self.position(self.time(now))
    }
}
