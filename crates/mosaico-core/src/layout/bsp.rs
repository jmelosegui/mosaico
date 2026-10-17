use crate::{Rect, WindowId};

use super::Layout;

/// Binary Space Partitioning layout.
///
/// Recursively splits the available space in half, alternating between
/// horizontal and vertical splits. The first window gets the larger
/// partition.
///
/// For example, with 3 windows on a 1920x1080 screen:
/// ```text
/// +-----------+-----------+
/// |           |     2     |
/// |     1     +-----------+
/// |           |     3     |
/// +-----------+-----------+
/// ```
#[derive(Debug, Clone)]
pub struct BspLayout {
    /// Gap in pixels between windows.
    pub gap: i32,
    /// Ratio of space given to the first window in each split (0.0–1.0).
    pub ratio: f64,
}

impl Default for BspLayout {
    fn default() -> Self {
        Self { gap: 8, ratio: 0.5 }
    }
}

impl Layout for BspLayout {
    fn apply(&self, windows: &[WindowId], work_area: &Rect) -> Vec<(WindowId, Rect)> {
        self.apply_splits(windows, work_area, &[])
    }
}

impl BspLayout {
    /// Lays out `windows` as [`Layout::apply`] does, with split `k`, which
    /// divides windows `k..` into window `k` and the rest, at the ratio
    /// `splits[k]` where it has one, and at `ratio` elsewhere.
    pub fn apply_splits(
        &self,
        windows: &[WindowId],
        work_area: &Rect,
        splits: &[Option<f64>],
    ) -> Vec<(WindowId, Rect)> {
        if windows.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::with_capacity(windows.len());
        let padded = Rect::new(
            work_area.x + self.gap,
            work_area.y + self.gap,
            (work_area.width - self.gap * 2).max(1),
            (work_area.height - self.gap * 2).max(1),
        );
        self.split(windows, &padded, true, splits, &mut results);
        results
    }

    fn split(
        &self,
        windows: &[WindowId],
        area: &Rect,
        horizontal: bool,
        splits: &[Option<f64>],
        results: &mut Vec<(WindowId, Rect)>,
    ) {
        if windows.len() == 1 {
            results.push((windows[0], *area));
            return;
        }

        let half = self.gap / 2;
        let ratio = splits.first().copied().flatten().unwrap_or(self.ratio);
        let (first_area, rest_area) = if horizontal {
            let split = (area.width as f64 * ratio) as i32;
            let first = Rect::new(area.x, area.y, (split - half).max(1), area.height);
            let rest = Rect::new(
                area.x + split + half,
                area.y,
                (area.width - split - half).max(1),
                area.height,
            );
            (first, rest)
        } else {
            let split = (area.height as f64 * ratio) as i32;
            let first = Rect::new(area.x, area.y, area.width, (split - half).max(1));
            let rest = Rect::new(
                area.x,
                area.y + split + half,
                area.width,
                (area.height - split - half).max(1),
            );
            (first, rest)
        };

        results.push((windows[0], first_area));
        let deeper = splits.get(1..).unwrap_or_default();
        self.split(&windows[1..], &rest_area, !horizontal, deeper, results);
    }
}
