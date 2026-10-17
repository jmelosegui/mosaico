//! Resizing the windows of a stack: the edge between two windows of the
//! vertical stack, or of a side column of the three columns, trades height
//! between them, and leaves every other edge where it is.

use crate::tiler::tests::Harness;
use crate::{LayoutKind, Rect};

use super::super::support::{held, resize_to};
use super::laid_out;

#[test]
fn the_bottom_edge_of_a_window_of_the_stack_moves_the_split_below_it() {
    // Arrange -- windows 2 and 3 stacked right of the master.
    let mut h = laid_out(LayoutKind::VerticalStack, 3);
    let (top, bottom) = (h.rect_of(2), h.rect_of(3));
    let gap = bottom.y - (top.y + top.height);

    // Act -- the bottom edge of window 2, 100 pixels down.
    let taller = Rect {
        height: top.height + 100,
        ..top
    };
    resize_to(&mut h, 2, held(false, false, false, true), taller);

    // Assert -- window 3 starts a gap below it, and still ends where it did.
    assert_eq!(h.rect_of(2), taller);
    let below = h.rect_of(3);
    let edges = (below.y, below.y + below.height);
    assert_eq!(
        edges,
        (taller.y + taller.height + gap, bottom.y + bottom.height)
    );
}

#[test]
fn the_bottom_edge_of_a_window_of_a_side_column_moves_the_split_below_it() {
    // Arrange -- windows 2 and 4 share the left column.
    let mut h = laid_out(LayoutKind::ThreeColumn, 5);
    let (top, bottom) = (h.rect_of(2), h.rect_of(4));
    let gap = bottom.y - (top.y + top.height);

    // Act -- the bottom edge of window 2, 100 pixels down.
    let taller = Rect {
        height: top.height + 100,
        ..top
    };
    resize_to(&mut h, 2, held(false, false, false, true), taller);

    // Assert
    assert_eq!(h.rect_of(2), taller);
    let below = h.rect_of(4);
    let edges = (below.y, below.y + below.height);
    assert_eq!(
        edges,
        (taller.y + taller.height + gap, bottom.y + bottom.height)
    );
}

#[test]
fn an_edge_down_the_stack_leaves_the_edges_below_it_where_they_are() {
    // Arrange -- windows 2, 3 and 4 stacked right of the master, the edge
    // between windows 3 and 4 put at 800.
    let mut h = laid_out(LayoutKind::VerticalStack, 4);
    let third = h.rect_of(3);
    let to_800 = Rect {
        height: 800 - third.y,
        ..third
    };
    resize_to(&mut h, 3, held(false, false, false, true), to_800);
    let last = h.rect_of(4);

    // Act -- the edge between windows 2 and 3, 100 pixels down.
    let second = h.rect_of(2);
    let taller = Rect {
        height: second.height + 100,
        ..second
    };
    resize_to(&mut h, 2, held(false, false, false, true), taller);

    // Assert -- window 3 gives up the height, and the edge below it stays
    // at 800, where a split of window 3 kept as a share of what is below
    // its top would take it to 838.
    assert_eq!(h.rect_of(2), taller);
    let third = h.rect_of(3);
    let edges = (third.y, third.y + third.height);
    assert_eq!(edges, (taller.y + taller.height + 8, 800));
    assert_eq!(h.rect_of(4), last);
}

#[test]
fn an_edge_down_the_stack_dragged_to_the_bottom_stops_and_leaves_the_rest_alone() {
    // Arrange -- windows 2 to 5 stacked right of the master, 260 pixels
    // tall each.
    let mut h = laid_out(LayoutKind::VerticalStack, 5);
    let bottom = held(false, false, false, true);
    let to_bottom = |rect: Rect| Rect {
        height: 1080 - rect.y,
        ..rect
    };
    let rows = |h: &Harness| [2, 3, 4, 5].map(|id| (h.rect_of(id).y, h.rect_of(id).height));

    // Act -- the bottom edge of window 2 dragged to the bottom of the
    // monitor, and then that of window 3.
    let second = to_bottom(h.rect_of(2));
    resize_to(&mut h, 2, bottom, second);
    let first = rows(&h);
    let third = to_bottom(h.rect_of(3));
    resize_to(&mut h, 3, bottom, third);

    // Assert -- each edge stops where the lower window keeps 26 pixels, a
    // tenth of the 260 an equal share of the stack has, and every other
    // window keeps its slot, whole gaps apart.
    assert_eq!(first, [(8, 494), (510, 26), (544, 260), (812, 260)]);
    assert_eq!(rows(&h), [(8, 494), (510, 260), (778, 26), (812, 260)]);
}
