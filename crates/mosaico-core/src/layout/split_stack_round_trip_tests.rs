//! A stack the user resized comes back to the pixel from what leaves it as
//! it was: a whole monitor that disconnects and comes back in order, and a
//! window that opens after the last and closes again, whatever half pixels
//! its edges end on.

use crate::id::wid;
use crate::{LayoutKind, Rect, WindowId};

use super::super::split_edges::Side;
use super::super::split_support::{beside, dragged, held, unmoved};
use super::super::{SplitRatios, Tiles, resized_splits};

/// Where `ratios` lay out `count` windows of `kind` in `area`, `gap` apart.
fn slots(kind: LayoutKind, count: usize, gap: i32, area: &Rect, ratios: &SplitRatios) -> Vec<Rect> {
    let windows: Vec<WindowId> = (1..=count as u64).map(wid).collect();
    let laid_out = ratios.apply(kind, gap, 0.5, &windows, area);
    let slot = |w: &WindowId| laid_out.iter().find(|(o, _)| o == w).map(|o| o.1);
    windows.iter().filter_map(slot).collect()
}

/// `ratios` once the `side` edge of window `index` of `count` windows of
/// `kind`, laid out in `area`, `gap` apart, is dragged `by` pixels.
fn drag(
    (kind, gap, area): (LayoutKind, i32, &Rect),
    ratios: &SplitRatios,
    count: usize,
    (index, side): (usize, Side),
    by: i32,
) -> SplitRatios {
    let before = slots(kind, count, gap, area, ratios);
    let rect = dragged(before[index], side, by);
    let tiles = Tiles::new(kind, gap, &before, ratios);
    let moved = resized_splits(tiles, index, held(side), before[index], rect);
    moved.unwrap_or_else(|| ratios.clone())
}

/// `ratios` once every one of `count` windows has left with what it owns,
/// as from a monitor that disconnects, and come back to its place, as
/// when the monitor is back.
fn back_in_order(ratios: &SplitRatios, count: usize) -> SplitRatios {
    let owned: Vec<_> = (0..count).map(|index| ratios.owned_by(index)).collect();
    let mut back = ratios.clone();
    (1..=count).rev().for_each(|left| back.remove(0, left));
    (0..count).for_each(|slot| back.insert(slot, slot));
    let given: Vec<_> = (0..count)
        .map(|index| (index, index, owned[index]))
        .collect();
    back.give(&given, count);
    back
}

/// `ratios` once a window opens after the last of `count` and closes again.
fn opened_and_closed(ratios: &SplitRatios, count: usize) -> SplitRatios {
    let mut again = ratios.clone();
    again.insert(count, count);
    again.remove(count, count + 1);
    again
}

#[test]
fn a_window_that_opens_and_closes_after_a_drag_to_the_floor_leaves_the_stack_as_it_was() {
    // Arrange -- seven windows on 1920x1050 with no gaps, 175 pixels each
    // down the stack, the bottom edge of window 5 dragged up to the floor,
    // which ends window 5 on half a pixel.
    let (kind, area) = (LayoutKind::VerticalStack, Rect::new(0, 0, 1920, 1050));
    let resized = drag((kind, 0, &area), &unmoved(7), 7, (5, Side::Bottom), -3000);

    // Act
    let again = opened_and_closed(&resized, 7);

    // Assert
    let laid_out = |ratios| slots(kind, 7, 0, &area, ratios);
    assert_eq!(laid_out(&again), laid_out(&resized));
}

#[test]
fn random_resized_stacks_come_back_to_the_pixel() {
    let mut seed = 0x0bad_5eed_1234_5678_u64;
    let mut next = move |below: usize| {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        (seed % below.max(1) as u64) as usize
    };
    let heights = [1080, 1050, 1032, 1200, 1410, 1440, 1600, 2130, 2160];
    let mut checked = 0;
    for round in 0..600 {
        let kind = [LayoutKind::VerticalStack, LayoutKind::ThreeColumn][round % 2];
        let gap = [0, 4, 5, 8, 10, 12, 16][next(7)];
        let area = Rect::new(0, 0, 1920, heights[next(heights.len())]);
        let mut count = 3 + next(6);
        let mut ratios = unmoved(count);
        for _ in 0..30 {
            match next(6) {
                0 if count < 12 => {
                    ratios.insert(next(count + 1), count);
                    count += 1;
                }
                1 if count > 3 => {
                    ratios.remove(next(count), count);
                    count -= 1;
                }
                _ => {
                    let stacked =
                        |&(i, side): &(usize, Side)| beside(kind, count, i, side).is_some();
                    let edges: Vec<(usize, Side)> = (1..count)
                        .flat_map(|i| [(i, Side::Top), (i, Side::Bottom)])
                        .filter(stacked)
                        .collect();
                    let Some(&edge) = edges.get(next(edges.len())) else {
                        continue;
                    };
                    let by = [-3000, 3000, next(401) as i32 - 200][next(3)];
                    ratios = drag((kind, gap, &area), &ratios, count, edge, by);
                }
            }
            // Assert -- after every step.
            let laid_out = |ratios: &SplitRatios| slots(kind, count, gap, &area, ratios);
            let (back, again) = (
                back_in_order(&ratios, count),
                opened_and_closed(&ratios, count),
            );
            let case = format!("round {round}, {kind:?} of {count}, gap {gap}, {area:?}");
            assert_eq!(laid_out(&back), laid_out(&ratios), "{case}");
            assert_eq!(laid_out(&again), laid_out(&ratios), "{case}");
            checked += 1;
        }
    }
    assert!(checked > 15_000, "{checked}");
}
