use std::time::Instant;

use crate::Rect;
use crate::motion::test_support::{ms, spec};
use crate::motion::{Easing, Velocity};

use super::super::Segment;

#[test]
fn an_eased_segment_starts_at_rest_and_moves_as_fast_as_its_curve_says() {
    // Arrange -- 1000 px in a second.
    let t0 = Instant::now();
    let (from, to) = (Rect::new(0, 0, 100, 100), Rect::new(1000, 0, 100, 100));
    let linear = Segment::eased(from, to, t0, spec(1000, Easing::Linear));
    let quad = Segment::eased(from, to, t0, spec(1000, Easing::EaseOutQuad));

    // Act
    let resting = linear.velocity_at(t0);
    let steady = linear.velocity_at(t0 + ms(1));
    let slowing = quad.velocity_at(t0 + ms(500));
    let arrived = linear.velocity_at(t0 + ms(1000));

    // Assert
    assert_eq!(resting, Velocity::ZERO, "at rest before it moves");
    assert!((steady.left - 1000.0).abs() < 1e-9, "{steady:?}");
    assert!((slowing.left - 1000.0).abs() < 1e-9, "{slowing:?}");
    assert_eq!((slowing.right, slowing.top), (slowing.left, 0.0));
    assert_eq!(arrived, Velocity::ZERO, "at rest at the end");
}
