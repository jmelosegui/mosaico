//! The tiles that follow the edges of a tiled window while the user
//! drags them, with `resize_splits` on: placed at once on each report, one
//! gap from the edge, in BSP, down a stack and at the floor, while the
//! window and the splits are left alone until the drop.

use crate::id::wid;
use crate::tiler::drag::live::support::border_at;
use crate::tiler::drag::resize::support::{RIGHT, grab_to, held, resizing, unmoved};
use crate::tiler::tests::Harness;
use crate::tiler::{Effect, Timing};
use crate::{Action, LayoutKind, Rect};

/// Windows `1..=count` tiled on one monitor in `kind`, with splits moved
/// by the edges of the windows the user resizes.
fn laid_out(kind: LayoutKind, count: u64) -> Harness {
    let windows: Vec<(u64, usize)> = (1..=count).map(|id| (id, 0)).collect();
    let mut h = Harness::with_settings(resizing(), 1, &windows);
    while h.tiler.monitors()[0].active().layout_kind() != kind {
        h.action(Action::CycleLayout);
    }
    h
}

/// `rect` with its bottom edge `by` pixels lower.
fn lower(rect: Rect, by: i32) -> Rect {
    Rect {
        height: rect.height + by,
        ..rect
    }
}

/// The bottom edge of a window, held alone.
const BOTTOM: crate::event::Edges = held(false, false, false, true);

#[test]
fn the_tile_beyond_a_held_edge_follows_it_before_the_drop() {
    // Arrange
    let mut h = laid_out(LayoutKind::Bsp, 2);
    let left = h.rect_of(1);

    // Act -- 200 pixels to the right, still held.
    let wider = Rect {
        width: left.width + 200,
        ..left
    };
    grab_to(&mut h, 1, RIGHT, wider);

    // Assert -- window 2 is placed at once, one gap from the edge, with
    // its border; window 1 is left where the user has it, and the split
    // waits for the drop.
    let beyond = Rect::new(wider.x + wider.width + 8, 8, 748, 1064);
    assert_eq!(h.placements(), vec![(2, beyond, Timing::Now)]);
    assert!(
        h.effects
            .contains(&Effect::snap(wid(2), beyond, Timing::Now))
    );
    assert_eq!(border_at(&h.effects, 2), Some(beyond));
    assert_eq!(h.rect_of(1), wider);
    assert!(unmoved(&h));
}

#[test]
fn the_tiles_on_both_sides_of_a_held_corner_follow_it() {
    // Arrange -- window 3 has the bottom of the right half.
    let mut h = laid_out(LayoutKind::Bsp, 3);
    let corner = held(true, true, false, false);

    // Act -- its top left corner 200 pixels left and 100 up.
    grab_to(&mut h, 3, corner, Rect::new(764, 444, 1148, 628));

    // Assert -- window 1 gives up the width, and window 2 the height.
    let placed = vec![
        (1, Rect::new(8, 8, 748, 1064), Timing::Now),
        (2, Rect::new(764, 8, 1148, 428), Timing::Now),
    ];
    assert_eq!(h.placements(), placed);
    assert!(unmoved(&h));
}

#[test]
fn down_the_stack_only_the_window_beyond_the_edge_follows_it() {
    // Arrange -- windows 2, 3 and 4 stacked right of the master.
    let mut h = laid_out(LayoutKind::VerticalStack, 4);
    let (third, last) = (h.rect_of(3), h.rect_of(4));

    // Act -- the bottom edge of window 3, 100 pixels down.
    let taller = lower(third, 100);
    grab_to(&mut h, 3, BOTTOM, taller);

    // Assert -- window 4 starts a gap below it and ends where it did, and
    // the master and window 2 stay.
    let top = taller.y + taller.height + 8;
    let below = Rect::new(last.x, top, last.width, last.y + last.height - top);
    assert_eq!(h.placements(), vec![(4, below, Timing::Now)]);
}

#[test]
fn down_a_side_column_only_the_window_beyond_the_edge_follows_it() {
    // Arrange -- windows 2 and 4 share the left column.
    let mut h = laid_out(LayoutKind::ThreeColumn, 5);
    let (top, bottom) = (h.rect_of(2), h.rect_of(4));

    // Act -- the bottom edge of window 2, 100 pixels down.
    let taller = lower(top, 100);
    grab_to(&mut h, 2, BOTTOM, taller);

    // Assert
    let y = taller.y + taller.height + 8;
    let below = Rect::new(bottom.x, y, bottom.width, bottom.y + bottom.height - y);
    assert_eq!(h.placements(), vec![(4, below, Timing::Now)]);
}

#[test]
fn a_tile_that_reaches_the_floor_stops_there_while_the_edge_goes_on() {
    // Arrange -- windows 2 to 5 stacked right of the master, 260 pixels
    // tall each.
    let mut h = laid_out(LayoutKind::VerticalStack, 5);
    let (second, third) = (h.rect_of(2), h.rect_of(3));

    // Act -- the bottom edge of window 2 dragged to the bottom of the
    // monitor, over the windows below it.
    let to_bottom = lower(second, 1072 - (second.y + second.height));
    grab_to(&mut h, 2, BOTTOM, to_bottom);

    // Assert -- window 3 keeps 26 pixels, a tenth of an equal share, and
    // the others below it stay; window 2 overlaps them until the drop.
    let floor = Rect::new(third.x, 510, third.width, 26);
    assert_eq!(h.placements(), vec![(3, floor, Timing::Now)]);
    assert_eq!(h.rect_of(2), to_bottom);
}
