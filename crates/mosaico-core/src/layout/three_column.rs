use crate::{Rect, WindowId};

use super::Layout;

/// Three-column layout with a master pane in the center and stacks on
/// both sides. Extra windows alternate between the left and right stacks.
///
/// ```text
/// +------+-----------+------+
/// |  2   |           |  3   |
/// +------+     1     +------+
/// |  4   | (master)  |  5   |
/// +------+-----------+------+
/// ```
#[derive(Debug, Clone)]
pub struct ThreeColumnLayout {
    /// Gap in pixels between windows.
    pub gap: i32,
    /// Ratio of width given to the center master pane (0.0–1.0).
    pub ratio: f64,
}

impl Default for ThreeColumnLayout {
    fn default() -> Self {
        Self { gap: 8, ratio: 0.5 }
    }
}

impl ThreeColumnLayout {
    /// Fills a vertical stack of windows into the given area, separating
    /// adjacent slots by a full `gap` so the inner spacing matches the
    /// outer padding.
    fn fill_stack(
        &self,
        windows: &[WindowId],
        area: &Rect,
        gap: i32,
        results: &mut Vec<(WindowId, Rect)>,
    ) {
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
}

impl Layout for ThreeColumnLayout {
    fn apply(&self, windows: &[WindowId], work_area: &Rect) -> Vec<(WindowId, Rect)> {
        if windows.is_empty() {
            return Vec::new();
        }

        let padded = Rect::new(
            work_area.x + self.gap,
            work_area.y + self.gap,
            (work_area.width - self.gap * 2).max(1),
            (work_area.height - self.gap * 2).max(1),
        );

        if windows.len() == 1 {
            return vec![(windows[0], padded)];
        }

        let half = self.gap / 2;

        // 2 windows: master left, second right.
        if windows.len() == 2 {
            let master_w = (padded.width as f64 * self.ratio) as i32;
            let master = Rect::new(padded.x, padded.y, (master_w - half).max(1), padded.height);
            let right = Rect::new(
                padded.x + master_w + half,
                padded.y,
                (padded.width - master_w - half).max(1),
                padded.height,
            );
            return vec![(windows[0], master), (windows[1], right)];
        }

        // 3+ windows: master in center, extras alternate left/right.
        // Reserve a full `gap` on each side of the master (2 × gap total).
        let master_w = (padded.width as f64 * self.ratio) as i32;
        let side_w = (padded.width - master_w - self.gap * 2) / 2;
        let left_x = padded.x;
        let master_x = padded.x + side_w + self.gap;
        let right_x = master_x + master_w + self.gap;
        let right_w = (padded.x + padded.width - right_x).max(1);

        let mut results = Vec::with_capacity(windows.len());

        results.push((
            windows[0],
            Rect::new(master_x, padded.y, master_w, padded.height),
        ));

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
        self.fill_stack(&left_windows, &left_area, self.gap, &mut results);

        let right_area = Rect::new(right_x, padded.y, right_w, padded.height);
        self.fill_stack(&right_windows, &right_area, self.gap, &mut results);

        results
    }
}
