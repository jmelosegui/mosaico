//! The vertical stack and the three columns as they were laid out before
//! their stacks had splits, kept as they were to check that a stack nobody
//! resized is laid out as it always was.

use crate::{Rect, WindowId};

/// The area of `work_area` that the layouts fill, inside the outer gap.
fn padded(work_area: &Rect, gap: i32) -> Rect {
    Rect::new(
        work_area.x + gap,
        work_area.y + gap,
        (work_area.width - gap * 2).max(1),
        (work_area.height - gap * 2).max(1),
    )
}

/// The vertical stack with `gap` and the master at `ratio`, as it was.
pub(super) fn vertical_stack(
    gap: i32,
    ratio: f64,
    windows: &[WindowId],
    work_area: &Rect,
) -> Vec<(WindowId, Rect)> {
    if windows.is_empty() {
        return Vec::new();
    }
    let padded = padded(work_area, gap);
    if windows.len() == 1 {
        return vec![(windows[0], padded)];
    }
    let half = gap / 2;
    let master_w = (padded.width as f64 * ratio) as i32;
    let master = Rect::new(padded.x, padded.y, (master_w - half).max(1), padded.height);
    let stack_x = padded.x + master_w + half;
    let stack_w = (padded.width - master_w - half).max(1);
    let stack_count = windows.len() - 1;
    let slot_h = (padded.height - gap * (stack_count as i32 - 1)) / stack_count as i32;
    let mut results = Vec::with_capacity(windows.len());
    results.push((windows[0], master));
    for (i, &window) in windows[1..].iter().enumerate() {
        let y = padded.y + (i as i32) * (slot_h + gap);
        let h = if i == stack_count - 1 {
            (padded.y + padded.height - y).max(1)
        } else {
            slot_h.max(1)
        };
        results.push((window, Rect::new(stack_x, y, stack_w, h)));
    }
    results
}

/// `windows` stacked down `area`, a full `gap` apart, as a side column of
/// the three columns was.
fn fill_stack(windows: &[WindowId], area: &Rect, gap: i32, results: &mut Vec<(WindowId, Rect)>) {
    if windows.is_empty() {
        return;
    }
    let count = windows.len() as i32;
    let slot_h = (area.height - gap * (count - 1)) / count;
    for (i, &window) in windows.iter().enumerate() {
        let sy = area.y + (i as i32) * (slot_h + gap);
        let sh = if i as i32 == count - 1 {
            (area.y + area.height - sy).max(1)
        } else {
            slot_h.max(1)
        };
        results.push((window, Rect::new(area.x, sy, area.width, sh)));
    }
}

/// The three columns with `gap` and the master at `ratio`, as they were.
pub(super) fn three_column(
    gap: i32,
    ratio: f64,
    windows: &[WindowId],
    work_area: &Rect,
) -> Vec<(WindowId, Rect)> {
    if windows.is_empty() {
        return Vec::new();
    }
    let padded = padded(work_area, gap);
    if windows.len() == 1 {
        return vec![(windows[0], padded)];
    }
    let half = gap / 2;
    if windows.len() == 2 {
        let master_w = (padded.width as f64 * ratio) as i32;
        let master = Rect::new(padded.x, padded.y, (master_w - half).max(1), padded.height);
        let right = Rect::new(
            padded.x + master_w + half,
            padded.y,
            (padded.width - master_w - half).max(1),
            padded.height,
        );
        return vec![(windows[0], master), (windows[1], right)];
    }
    let master_w = (padded.width as f64 * ratio) as i32;
    let side_w = (padded.width - master_w - gap * 2) / 2;
    let left_x = padded.x;
    let master_x = padded.x + side_w + gap;
    let right_x = master_x + master_w + gap;
    let right_w = (padded.x + padded.width - right_x).max(1);
    let mut results = Vec::with_capacity(windows.len());
    let center = Rect::new(master_x, padded.y, master_w, padded.height);
    results.push((windows[0], center));
    let mut left_windows = Vec::new();
    let mut right_windows = Vec::new();
    for (i, &window) in windows[1..].iter().enumerate() {
        if i % 2 == 0 {
            left_windows.push(window);
        } else {
            right_windows.push(window);
        }
    }
    let left_area = Rect::new(left_x, padded.y, side_w.max(1), padded.height);
    fill_stack(&left_windows, &left_area, gap, &mut results);
    let right_area = Rect::new(right_x, padded.y, right_w, padded.height);
    fill_stack(&right_windows, &right_area, gap, &mut results);
    results
}
