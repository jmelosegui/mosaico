use crate::Rect;

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
    /// Fills a vertical stack of windows into the given area.
    fn fill_stack(
        &self,
        handles: &[usize],
        area: &Rect,
        half: i32,
        results: &mut Vec<(usize, Rect)>,
    ) {
        if handles.is_empty() {
            return;
        }
        let count = handles.len() as i32;
        let slot_h = (area.height - half * (count - 1)) / count;

        for (i, &hwnd) in handles.iter().enumerate() {
            let sy = area.y + (i as i32) * (slot_h + half);
            let sh = if i as i32 == count - 1 {
                (area.y + area.height - sy).max(1)
            } else {
                slot_h.max(1)
            };
            results.push((hwnd, Rect::new(area.x, sy, area.width, sh)));
        }
    }
}

impl Layout for ThreeColumnLayout {
    fn apply(&self, handles: &[usize], work_area: &Rect) -> Vec<(usize, Rect)> {
        if handles.is_empty() {
            return Vec::new();
        }

        let padded = Rect::new(
            work_area.x + self.gap,
            work_area.y + self.gap,
            (work_area.width - self.gap * 2).max(1),
            (work_area.height - self.gap * 2).max(1),
        );

        if handles.len() == 1 {
            return vec![(handles[0], padded)];
        }

        let half = self.gap / 2;

        // 2 windows: master left, second right.
        if handles.len() == 2 {
            let master_w = (padded.width as f64 * self.ratio) as i32;
            let master = Rect::new(padded.x, padded.y, (master_w - half).max(1), padded.height);
            let right = Rect::new(
                padded.x + master_w + half,
                padded.y,
                (padded.width - master_w - half).max(1),
                padded.height,
            );
            return vec![(handles[0], master), (handles[1], right)];
        }

        // 3+ windows: master in center, extras alternate left/right.
        let master_w = (padded.width as f64 * self.ratio) as i32;
        let side_w = (padded.width - master_w - half * 2) / 2;
        let left_x = padded.x;
        let master_x = padded.x + side_w + half;
        let right_x = master_x + master_w + half;
        let right_w = (padded.x + padded.width - right_x).max(1);

        let mut results = Vec::with_capacity(handles.len());

        results.push((
            handles[0],
            Rect::new(master_x, padded.y, master_w, padded.height),
        ));

        let mut left_handles = Vec::new();
        let mut right_handles = Vec::new();
        for (i, &hwnd) in handles[1..].iter().enumerate() {
            if i % 2 == 0 {
                left_handles.push(hwnd);
            } else {
                right_handles.push(hwnd);
            }
        }

        let left_area = Rect::new(left_x, padded.y, side_w.max(1), padded.height);
        self.fill_stack(&left_handles, &left_area, half, &mut results);

        let right_area = Rect::new(right_x, padded.y, right_w, padded.height);
        self.fill_stack(&right_handles, &right_area, half, &mut results);

        results
    }
}
