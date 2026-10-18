//! A column of windows, one above the other: the stack of the vertical
//! stack, and each side column of the three columns.

use crate::Rect;

/// How far past half a pixel an end may be and still round down: an end on
/// half a pixel rounds down whatever the last bits of its share, which
/// rescaling the shares, as a window that joins and leaves or a monitor
/// that comes back does, can change, while an end dropped on a pixel stays
/// on it.
const HALF_PIXEL_SLACK: f64 = 1e-6;

/// The slots of `count` windows stacked down `area`, a full `gap` apart.
///
/// With no `ends`, as for a column nobody resized, the windows share the
/// column equally, and the last takes what rounding leaves, as columns
/// always were. With `ends`, where each window but the last ends as a
/// share of the column without its gaps, each one ends there, to the
/// nearest pixel, or the one above it on half a pixel, and the last at the
/// bottom of the area, so the gaps stay whole and no window leaves the
/// area.
pub(super) fn column_slots(area: &Rect, count: usize, gap: i32, ends: Option<&[f64]>) -> Vec<Rect> {
    let bottom = area.y + area.height;
    let available = area.height - gap * (count as i32 - 1).max(0);
    let ends = ends.filter(|ends| ends.len() + 1 == count);
    // Where window `i` ends: past the windows above it, its own height and
    // the gaps between them.
    let end = |i: usize| {
        if i + 1 == count {
            return bottom;
        }
        let heights = match ends {
            Some(ends) => (f64::from(available.max(0)) * ends[i] - HALF_PIXEL_SLACK).round() as i32,
            None => available / count as i32 * (i as i32 + 1),
        };
        area.y + heights + gap * i as i32
    };
    let mut slots = Vec::with_capacity(count);
    let mut y = area.y;
    for i in 0..count {
        let to = end(i);
        slots.push(Rect::new(area.x, y, area.width, (to - y).max(1)));
        y = to + gap;
    }
    slots
}

#[cfg(test)]
#[path = "column_tests.rs"]
mod tests;
