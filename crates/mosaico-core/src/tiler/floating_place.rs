//! Where floating windows go: arithmetic on rectangles, with no state.

use crate::{Direction, MinSize, Rect};

/// How many steps a nudge or a resize takes to cross the work area.
const STEPS: i32 = 20;

/// A `width` by `height` rect centered in `area`, shrunk to fit it.
pub(in crate::tiler) fn centered(area: Rect, width: i32, height: i32) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    )
}

/// Where a window floating for the first time goes: centered in `area`,
/// at two-thirds of its width and height.
pub(in crate::tiler) fn first(area: Rect) -> Rect {
    centered(area, area.width * 2 / 3, area.height * 2 / 3)
}

/// `rect` shrunk to fit `area` and moved the least into it.
pub(in crate::tiler) fn clamp_into(rect: Rect, area: Rect) -> Rect {
    let width = rect.width.min(area.width);
    let height = rect.height.min(area.height);
    Rect::new(
        rect.x.clamp(area.x, area.x + area.width - width),
        rect.y.clamp(area.y, area.y + area.height - height),
        width,
        height,
    )
}

/// `rect`, inside `from`, moved to the same place inside `to`: a window
/// at the left third of one work area lands at the left third of the
/// other, keeping its size and shrunk to fit.
pub(in crate::tiler) fn relative_move(rect: Rect, from: Rect, to: Rect) -> Rect {
    let scale = |offset: i32, from: i32, to: i32| {
        if from <= 0 {
            return 0;
        }
        i32::try_from(i64::from(offset) * i64::from(to) / i64::from(from)).unwrap_or(0)
    };
    let moved = Rect::new(
        to.x + scale(rect.x - from.x, from.width, to.width),
        to.y + scale(rect.y - from.y, from.height, to.height),
        rect.width,
        rect.height,
    );
    clamp_into(moved, to)
}

/// Where a floating window at `rect`, which belongs in `own`, goes after
/// the displays changed: to the same place inside `own` when it is
/// centered in another of `areas`, as when its monitor came back, centered
/// in `own` when it is in none of them, as when its monitor went away,
/// and otherwise moved into `own` where it is partly outside.
pub(in crate::tiler) fn settle(rect: Rect, own: Rect, areas: &[Rect]) -> Rect {
    let holds = |a: &Rect| {
        (a.x..a.x + a.width).contains(&rect.center_x())
            && (a.y..a.y + a.height).contains(&rect.center_y())
    };
    match areas.iter().find(|a| holds(a)) {
        Some(&here) if here != own => relative_move(rect, here, own),
        Some(_) => clamp_into(rect, own),
        None => centered(own, rect.width, rect.height),
    }
}

/// `rect` moved one step towards `dir`, stopping at the edge of `area`.
pub(in crate::tiler) fn nudge(rect: Rect, area: Rect, dir: Direction) -> Rect {
    let (dx, dy) = match dir {
        Direction::Left => (-area.width / STEPS, 0),
        Direction::Right => (area.width / STEPS, 0),
        Direction::Up => (0, -area.height / STEPS),
        Direction::Down => (0, area.height / STEPS),
    };
    clamp_into(
        Rect::new(rect.x + dx, rect.y + dy, rect.width, rect.height),
        area,
    )
}

/// Which side of a window a resize changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::tiler) enum Axis {
    /// Its width.
    Width,
    /// Its height.
    Height,
}

/// `rect` one step wider or taller (`grow`), or narrower or shorter,
/// around its center: never larger than `area` nor smaller than `min`,
/// and kept inside `area`.
pub(in crate::tiler) fn resize(
    rect: Rect,
    area: Rect,
    axis: Axis,
    grow: bool,
    min: MinSize,
) -> Rect {
    let sign = if grow { 1 } else { -1 };
    let mut out = rect;
    match axis {
        Axis::Width => {
            let width = (rect.width + sign * area.width / STEPS).max(min.width.max(1));
            out.width = width.min(area.width);
            out.x = rect.center_x() - out.width / 2;
        }
        Axis::Height => {
            let height = (rect.height + sign * area.height / STEPS).max(min.height.max(1));
            out.height = height.min(area.height);
            out.y = rect.center_y() - out.height / 2;
        }
    }
    clamp_into(out, area)
}

#[cfg(test)]
#[path = "tests/floating_place_tests.rs"]
mod tests;
