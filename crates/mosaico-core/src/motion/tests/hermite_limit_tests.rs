use std::time::Instant;

use crate::Rect;
use crate::motion::Velocity;
use crate::motion::test_support::ms;

use super::super::Segment;

/// The edges of `rect`, `[left, top, right, bottom]`.
fn edges(rect: Rect) -> [i32; 4] {
    [rect.x, rect.y, rect.x + rect.width, rect.y + rect.height]
}

#[test]
fn an_edge_leaves_at_rest_when_moving_away_and_never_too_fast_to_stop() {
    // Arrange -- 100 px to the right in 200 ms, which an edge can start
    // at up to 1500 px a second without passing its target.
    let t0 = Instant::now();
    let (from, to) = (Rect::new(0, 0, 400, 400), Rect::new(100, 100, 400, 400));
    let velocity = Velocity {
        left: 50_000.0,
        top: -3000.0,
        right: 1000.0,
        bottom: 0.0,
    };

    // Act
    let leaving = Segment::hermite(from, velocity, to, t0, ms(200)).velocity_at(t0);

    // Assert
    let limited = Velocity {
        left: 1500.0,
        top: 0.0,
        right: 1000.0,
        bottom: 0.0,
    };
    assert_eq!(leaving, limited);
}

#[test]
fn a_hermite_segment_never_passes_its_target() {
    // Arrange -- far too fast towards the target, and away from it.
    let t0 = Instant::now();
    let (from, to) = (Rect::new(300, 200, 600, 400), Rect::new(340, 120, 500, 500));
    let rushing = Velocity {
        left: 90_000.0,
        top: -90_000.0,
        right: -90_000.0,
        bottom: 90_000.0,
    };
    let fleeing = Velocity::from_edges(rushing.edges().map(|v| -v));

    for velocity in [rushing, fleeing] {
        let segment = Segment::hermite(from, velocity, to, t0, ms(180));
        for at in (0..=180).map(|i| t0 + ms(i)) {
            // Act
            let rect = edges(segment.rect_at(at));

            // Assert -- every edge between where it was and where it goes.
            for (i, edge) in rect.iter().enumerate() {
                let (a, b) = (edges(from)[i], edges(to)[i]);
                assert!(
                    (a.min(b)..=a.max(b)).contains(edge),
                    "{velocity:?}: {rect:?}"
                );
            }
        }
    }
}

#[test]
fn an_edge_whose_speed_is_not_a_number_leaves_at_rest() {
    // Arrange -- only a caller passing these to the public API can get
    // here: every velocity a segment works out is finite.
    let t0 = Instant::now();
    let (from, to) = (Rect::new(0, 0, 400, 400), Rect::new(100, 500, 400, 400));
    let broken = Velocity {
        left: f64::NAN,
        top: f64::INFINITY,
        right: f64::NEG_INFINITY,
        bottom: f64::NAN,
    };
    let segment = Segment::hermite(from, broken, to, t0, ms(200));

    // Act
    let leaving = segment.velocity_at(t0);
    let halfway = segment.rect_at(t0 + ms(100));

    // Assert -- from rest, a Hermite curve is halfway at half the time.
    assert_eq!(leaving, Velocity::ZERO);
    assert_eq!(halfway, Rect::new(50, 250, 400, 400));
    assert_eq!(segment.rect_at(t0 + ms(200)), to);
}
