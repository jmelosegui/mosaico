use super::*;

use super::super::split_edges::border;
use super::super::split_support::{GAP, KINDS, after, dragged, edge_of, slots, unmoved};

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

#[test]
fn every_corner_moves_the_splits_its_edges_border_to_where_they_were_dropped() {
    for kind in KINDS {
        for count in 2..=5 {
            let before = slots(kind, count, &unmoved(count));
            for (index, sides) in (0..count).flat_map(|i| CORNERS.map(|c| (i, c))) {
                for (by_a, by_b) in [(-30, 23), (41, -17)] {
                    // Arrange -- both edges of the corner dragged.
                    let (a, b) = sides;
                    let rect = dragged(dragged(before[index], a, by_a), b, by_b);

                    // Act
                    let start = before[index];
                    let moved =
                        resized_splits(kind, GAP, &before, index, corner(sides), start, rect);

                    // Assert -- an edge that borders a split lands where it
                    // was dropped, and one that borders none goes back.
                    let laid_out = after(kind, count, &moved)[index];
                    let case = format!("{kind:?}, window {index} of {count}, {sides:?}");
                    let bordering = [a, b].map(|side| border(kind, count, index, side).is_some());
                    let expected = bordering.iter().filter(|&&yes| yes).count();
                    assert_eq!(moved.len(), expected, "{case}");
                    for (side, borders) in [a, b].into_iter().zip(bordering) {
                        let to = if borders { rect } else { before[index] };
                        let edge = (edge_of(laid_out, side), edge_of(to, side));
                        assert_eq!(edge.0, edge.1, "{case}, {side:?}");
                    }
                }
            }
        }
    }
}
