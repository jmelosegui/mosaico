//! Over random states that windows coming and going anywhere, retiles,
//! master drags and drags down the stacks reach, in the vertical stack and
//! the three columns, on areas of many heights and with many gaps: every
//! stack stays shared equally or at heights that sum to 1 and give each
//! window a tenth of an equal share, every edge dragged down a stack
//! trades height between its two windows alone and never moves against
//! the drag, and the windows of each column fill it, a whole gap apart.

use crate::id::wid;
use crate::{LayoutKind, Rect, WindowId};

use super::super::split_edges::Side;
use super::super::split_support::{beside, changed, dragged, edge_of, held, unmoved};
use super::super::{MIN_SPLIT_RATIO, SplitRatios, Tiles, resized_splits};
use super::column_of;

/// A small generator of numbers, the same on every run.
struct Numbers(u64);

impl Numbers {
    /// The next number, below `limit`.
    fn below(&mut self, limit: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % limit.max(1) as u64) as usize
    }
}

/// Where `ratios` lay out `count` windows of `kind` in `area`, `gap` apart.
fn slots(kind: LayoutKind, count: usize, gap: i32, area: &Rect, ratios: &SplitRatios) -> Vec<Rect> {
    let windows: Vec<WindowId> = (1..=count as u64).map(wid).collect();
    let laid_out = ratios.apply(kind, gap, 0.5, &windows, area);
    windows
        .iter()
        .filter_map(|w| laid_out.iter().find(|(o, _)| o == w).map(|o| o.1))
        .collect()
}

/// Whether both columns of `ratios` for `kind` are shared equally, or give
/// each of their windows a tenth of an equal share, the shares summing
/// to 1.
fn sound(ratios: &SplitRatios, kind: LayoutKind) -> bool {
    [1, 2].into_iter().all(|window| {
        let Some(shares) = ratios.heights(kind, window) else {
            return true;
        };
        let least = MIN_SPLIT_RATIO / shares.len() as f64 - 1e-12;
        let total: f64 = shares.iter().sum();
        shares.iter().all(|&share| share >= least) && (total - 1.0).abs() < 1e-9
    })
}

/// Whether the windows of each column of `slots` fill `area` inside its
/// outer gap, top to bottom, a whole `gap` apart.
fn filled(kind: LayoutKind, slots: &[Rect], gap: i32, area: &Rect) -> bool {
    let (top, bottom) = (area.y + gap, area.y + area.height - gap);
    [1, 2]
        .into_iter()
        .filter(|&w| w < slots.len())
        .all(|first| {
            let column: Vec<Rect> = column_of(kind, slots.len(), first)
                .into_iter()
                .map(|w| slots[w])
                .collect();
            let apart = column
                .windows(2)
                .all(|r| r[1].y == r[0].y + r[0].height + gap);
            let ends = column.last().map(|r| r.y + r.height) == Some(bottom);
            column[0].y == top && apart && ends && column.iter().all(|r| r.height >= 1)
        })
}

#[test]
fn random_states_keep_the_stacks_sound_and_every_edge_with_its_drag() {
    let mut numbers = Numbers(0x9e37_79b9_7f4a_7c15);
    let mut drags = 0;
    for round in 0..300 {
        let kind = [LayoutKind::VerticalStack, LayoutKind::ThreeColumn][round % 2];
        let gap = numbers.below(17) as i32;
        let area = Rect::new(0, 0, 1920, 600 + numbers.below(1561) as i32);
        let mut count = 3 + numbers.below(5);
        let mut ratios = unmoved(count);
        for _ in 0..40 {
            match numbers.below(12) {
                0 | 1 if count < 12 => {
                    ratios.insert(numbers.below(count + 1), count);
                    count += 1;
                }
                2 | 3 if count > 3 => {
                    ratios.remove(numbers.below(count), count);
                    count -= 1;
                }
                4 if numbers.below(4) == 0 => ratios.forget(),
                5 => ratios.set(kind, 0, 0.2 + numbers.below(7) as f64 / 10.0),
                _ => {
                    let before = slots(kind, count, gap, &area, &ratios);
                    let stacked =
                        |&(i, side): &(usize, Side)| beside(kind, count, i, side).is_some();
                    let edges: Vec<(usize, Side)> = (1..count)
                        .flat_map(|i| [(i, Side::Top), (i, Side::Bottom)])
                        .filter(stacked)
                        .collect();
                    let Some(&(index, side)) = edges.get(numbers.below(edges.len())) else {
                        continue;
                    };
                    let rect = dragged(before[index], side, numbers.below(801) as i32 - 400);
                    let tiles = Tiles::new(kind, gap, &before, &ratios);
                    if let Some(moved) =
                        resized_splits(tiles, index, held(side), before[index], rect)
                    {
                        ratios = moved;
                    }
                    let after = slots(kind, count, gap, &area, &ratios);
                    let two = beside(kind, count, index, side).unwrap_or_default();
                    let edge = [before[index], rect, after[index]].map(|r| edge_of(r, side));
                    let range = edge[0].min(edge[1])..=edge[0].max(edge[1]);
                    let case = format!("{kind:?} {index} {side:?} of {count}: {edge:?}");
                    assert!(
                        changed(&before, &after).iter().all(|w| two.contains(w)),
                        "{case}"
                    );
                    assert!(range.contains(&edge[2]), "{case}");
                    drags += 1;
                }
            }
            assert!(sound(&ratios, kind), "{kind:?} of {count}: {ratios:?}");
            let laid_out = slots(kind, count, gap, &area, &ratios);
            assert!(
                filled(kind, &laid_out, gap, &area),
                "{kind:?} of {count}: {ratios:?}"
            );
        }
    }
    assert!(drags > 5000, "{drags}");
}
