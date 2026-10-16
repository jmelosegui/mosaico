use std::time::Instant;

use crate::Rect;
use crate::motion::Easing;
use crate::motion::test_support::{ms, spec};

use super::super::Segment;

#[test]
fn rects_moving_together_keep_the_gap_between_them() {
    // Arrange -- two tiles 8 px apart whose shared edge moves right.
    let t0 = Instant::now();
    let quick = spec(180, Easing::default());
    let left = Segment::eased(
        Rect::new(0, 0, 500, 400),
        Rect::new(0, 0, 733, 400),
        t0,
        quick,
    );
    let right = Segment::eased(
        Rect::new(508, 0, 500, 400),
        Rect::new(741, 0, 267, 400),
        t0,
        quick,
    );

    for at in (0..=180).map(|i| t0 + ms(i)) {
        // Act
        let (l, r) = (left.rect_at(at), right.rect_at(at));

        // Assert -- edge by edge, whatever the rounding.
        assert_eq!(l.x + l.width + 8, r.x, "{l:?} {r:?}");
        assert_eq!(r.x + r.width, 1008, "{r:?}");
    }
}

#[test]
fn rects_moving_together_keep_the_gap_between_them_across_zero() {
    // Arrange -- two tiles stacked 8 px apart moving up through y = 0,
    // a pixel a millisecond, so every half millisecond puts their edges
    // on half pixels on either side of 0.
    let t0 = Instant::now();
    let linear = spec(200, Easing::Linear);
    let upper = Segment::eased(
        Rect::new(0, 0, 400, 100),
        Rect::new(0, -200, 400, 100),
        t0,
        linear,
    );
    let lower = Segment::eased(
        Rect::new(0, 108, 400, 300),
        Rect::new(0, -92, 400, 300),
        t0,
        linear,
    );

    for at in (0..=400).map(|i| t0 + ms(i) / 2) {
        // Act
        let (u, l) = (upper.rect_at(at), lower.rect_at(at));

        // Assert
        assert_eq!(u.y + u.height + 8, l.y, "{u:?} {l:?}");
    }
}
