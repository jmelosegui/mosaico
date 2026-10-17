use super::*;

use super::super::split_edges::border;
use super::super::split_support::{
    GAP, KINDS, after, beside, changed, dragged, edge_of, moved_splits, slots, unmoved,
};

/// The four corners of a window, each as the two edges that meet there.
const CORNERS: [(Side, Side); 4] = [
    (Side::Left, Side::Top),
    (Side::Top, Side::Right),
    (Side::Right, Side::Bottom),
    (Side::Bottom, Side::Left),
];

/// The edges of a window held at the corner where `a` and `b` meet.
fn corner((a, b): (Side, Side)) -> Edges {
    let held = |side: Side| a == side || b == side;
    Edges {
        left: held(Side::Left),
        top: held(Side::Top),
        right: held(Side::Right),
        bottom: held(Side::Bottom),
    }
}

/// Where each of `rects` is down the area, and how tall.
fn rows(rects: &[Rect]) -> Vec<Rect> {
    rects
        .iter()
        .map(|r| Rect::new(0, r.y, 0, r.height))
        .collect()
}

#[test]
fn every_corner_moves_the_splits_its_edges_border_to_where_they_were_dropped() {
    for kind in KINDS {
        for count in 2..=8 {
            let ratios = unmoved(count);
            let before = slots(kind, count, &ratios);
            for (index, sides) in (0..count).flat_map(|i| CORNERS.map(|c| (i, c))) {
                for (by_a, by_b) in [(-30, 23), (41, -17)] {
                    // Arrange -- both edges of the corner dragged.
                    let (a, b) = sides;
                    let rect = dragged(dragged(before[index], a, by_a), b, by_b);

                    // Act
                    let tiles = Tiles::new(kind, GAP, &before, &ratios);
                    let moved = resized_splits(tiles, index, corner(sides), before[index], rect);

                    // Assert -- an edge that borders a split lands where it
                    // was dropped, and one that borders none goes back. One
                    // down a stack leaves every window but the two beside it
                    // where it was down the area, and as tall.
                    let laid_out = after(kind, count, &moved);
                    let case = format!("{kind:?}, window {index} of {count}, {sides:?}");
                    let bordering = [a, b].map(|side| border(kind, count, index, side).is_some());
                    let stacked = [a, b].map(|side| beside(kind, count, index, side));
                    if stacked.iter().all(Option::is_none) {
                        let expected = bordering.iter().filter(|&&yes| yes).count();
                        let splits = moved_splits(kind, count, &moved);
                        assert_eq!(splits.len(), expected, "{case}");
                    }
                    for (side, borders) in [a, b].into_iter().zip(bordering) {
                        let to = if borders { rect } else { before[index] };
                        let edge = (edge_of(laid_out[index], side), edge_of(to, side));
                        assert_eq!(edge.0, edge.1, "{case}, {side:?}");
                    }
                    for two in stacked.into_iter().flatten() {
                        let others = changed(&rows(&before), &rows(&laid_out));
                        assert!(others.iter().all(|w| two.contains(w)), "{case}");
                    }
                }
            }
        }
    }
}
