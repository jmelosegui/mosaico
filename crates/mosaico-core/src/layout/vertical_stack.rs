use crate::Rect;

use super::Layout;

/// Master/stack layout with one master pane on the left and remaining
/// windows stacked vertically on the right.
///
/// ```text
/// +-----------+-----------+
/// |           |     2     |
/// |     1     +-----------+
/// | (master)  |     3     |
/// +-----------+-----------+
/// ```
#[derive(Debug, Clone)]
pub struct VerticalStackLayout {
    /// Gap in pixels between windows.
    pub gap: i32,
    /// Ratio of width given to the master pane (0.0–1.0).
    pub ratio: f64,
}

impl Default for VerticalStackLayout {
    fn default() -> Self {
        Self { gap: 8, ratio: 0.5 }
    }
}

impl Layout for VerticalStackLayout {
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
        let master_w = (padded.width as f64 * self.ratio) as i32;

        let master = Rect::new(padded.x, padded.y, (master_w - half).max(1), padded.height);

        let stack_x = padded.x + master_w + half;
        let stack_w = (padded.width - master_w - half).max(1);
        let stack_count = handles.len() - 1;
        let slot_h = (padded.height - half * (stack_count as i32 - 1)) / stack_count as i32;

        let mut results = Vec::with_capacity(handles.len());
        results.push((handles[0], master));

        for (i, &hwnd) in handles[1..].iter().enumerate() {
            let y = padded.y + (i as i32) * (slot_h + half);
            let h = if i == stack_count - 1 {
                (padded.y + padded.height - y).max(1)
            } else {
                slot_h.max(1)
            };
            results.push((hwnd, Rect::new(stack_x, y, stack_w, h)));
        }

        results
    }
}
