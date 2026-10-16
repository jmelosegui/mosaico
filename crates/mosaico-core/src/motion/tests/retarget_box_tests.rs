use std::time::Instant;

use crate::Rect;
use crate::id::wid;
use crate::motion::Easing;
use crate::motion::test_support::{ms, spec};

use super::super::Motions;
use super::ANYWHERE;

/// The span of each edge of `rects`, as `(least, most)` for the left,
/// top, right and bottom edges.
fn spans(rects: &[Rect]) -> [(i32, i32); 4] {
    std::array::from_fn(|i| {
        let edges = rects.iter().map(|r| edges(*r)[i]);
        (edges.clone().min().unwrap(), edges.max().unwrap())
    })
}

fn edges(rect: Rect) -> [i32; 4] {
    [rect.x, rect.y, rect.x + rect.width, rect.y + rect.height]
}

#[test]
fn a_new_target_never_takes_the_window_beyond_where_it_was_and_where_it_goes() {
    // Arrange -- sliding right fast, a third of the way the target turns
    // back left and down, or comes up just ahead of the window, which
    // its speed would carry far past.
    let t0 = Instant::now();
    let (start, first) = (Rect::new(0, 0, 800, 600), Rect::new(1200, 0, 800, 600));
    let quick = spec(180, Easing::default());
    let at = t0 + ms(60);
    for target in [Rect::new(200, 300, 600, 400), Rect::new(900, 40, 800, 600)] {
        let mut motions = Motions::default();
        motions.retarget(wid(1), start, first, t0, quick);
        let mut sent = Vec::new();
        for i in 0..60 {
            sent.extend(motions.frame(t0 + ms(i)).in_flight);
        }
        let turning = motions.rect_at(wid(1), at).unwrap();

        // Act
        motions.retarget(wid(1), ANYWHERE, target, at, quick);
        for i in 0..=180 {
            let frame = motions.frame(at + ms(i));
            sent.extend(frame.in_flight.into_iter().chain(frame.finished));
        }

        // Assert -- frame by frame, edge by edge.
        let bounds = spans(&[start, turning, target]);
        for (_, rect) in &sent {
            let inside = (0..4).all(|i| (bounds[i].0..=bounds[i].1).contains(&edges(*rect)[i]));
            assert!(
                inside,
                "{rect:?} leaves {bounds:?} on the way to {target:?}"
            );
        }
        assert_eq!(sent.last(), Some(&(wid(1), target)));
    }
}
