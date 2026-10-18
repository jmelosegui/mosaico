//! Giving the windows of a column their minimum heights.

use crate::Rect;

use super::min_size::MinSize;

/// `slots`, the windows of a column one above the other, `gap` apart, with
/// each one at least as high as its minimum in `mins`: a window below its
/// minimum gets it, and the others that have room give up the height that
/// takes, in proportion to the room they have. The column keeps its top
/// and its bottom. Left as it is when no window is below its minimum, and
/// when the windows' room is not enough, which is when the column cannot
/// hold every minimum.
pub(super) fn fit_column(slots: &mut [Rect], mins: &[MinSize], gap: i32) {
    if mins.len() != slots.len() || slots.is_empty() {
        return;
    }
    let least = |i: usize| mins[i].along(false);
    let missing: i32 = (0..slots.len())
        .map(|i| (least(i) - slots[i].height).max(0))
        .sum();
    let room = |i: usize| (slots[i].height - least(i)).max(0);
    let spare: i32 = (0..slots.len()).map(room).sum();
    if missing == 0 || missing > spare {
        return;
    }
    let mut heights: Vec<i32> = (0..slots.len())
        .map(|i| slots[i].height.max(least(i)))
        .collect();
    let mut owed = missing;
    let mut takes: Vec<(usize, i32)> = (0..slots.len())
        .map(|i| {
            (
                i,
                (i64::from(missing) * i64::from(room(i)) / i64::from(spare)) as i32,
            )
        })
        .collect();
    owed -= takes.iter().map(|&(_, take)| take).sum::<i32>();
    // What rounding leaves comes off the windows with the most room.
    let mut by_room: Vec<usize> = (0..slots.len()).filter(|&i| room(i) > 0).collect();
    by_room.sort_by_key(|&i| std::cmp::Reverse(room(i)));
    for &i in by_room.iter().cycle().take(by_room.len() * 2) {
        if owed > 0 && takes[i].1 < room(i) {
            takes[i].1 += 1;
            owed -= 1;
        }
    }
    for (i, take) in takes {
        heights[i] -= take;
    }
    let mut y = slots[0].y;
    for (slot, height) in slots.iter_mut().zip(heights) {
        slot.y = y;
        slot.height = height;
        y += height + gap;
    }
}

#[cfg(test)]
#[path = "min_column_tests.rs"]
mod tests;
