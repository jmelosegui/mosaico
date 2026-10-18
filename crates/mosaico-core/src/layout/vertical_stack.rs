use crate::{LayoutKind, Rect, WindowId};

use super::Layout;
use super::column::column_slots;
use super::min_column::fit_column;
use super::min_limits::{fit, limits};
use super::min_size::MinSize;

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
    fn apply(&self, windows: &[WindowId], work_area: &Rect) -> Vec<(WindowId, Rect)> {
        self.apply_heights(windows, work_area, None)
    }
}

impl VerticalStackLayout {
    /// Lays out `windows` as [`Layout::apply`] does, with the windows of the
    /// stack at the heights `stack` gives them: where each but the last
    /// ends, as a share of the stack without its gaps, or shared equally
    /// without them.
    pub fn apply_heights(
        &self,
        windows: &[WindowId],
        work_area: &Rect,
        stack: Option<&[f64]>,
    ) -> Vec<(WindowId, Rect)> {
        self.apply_fitted(windows, work_area, stack, &[])
    }

    /// Lays out `windows` as [`apply_heights`](Self::apply_heights) does,
    /// with the master's edge kept where the master and the stack keep
    /// their minimum sizes, `mins`, one for each window or none at all, and
    /// the windows of the stack tall enough for theirs, the height coming
    /// from those with room. What cannot leave every window its minimum is
    /// left as it would be.
    pub fn apply_fitted(
        &self,
        windows: &[WindowId],
        work_area: &Rect,
        stack: Option<&[f64]>,
        mins: &[MinSize],
    ) -> Vec<(WindowId, Rect)> {
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
        let mins = if mins.len() == windows.len() {
            mins
        } else {
            &[]
        };
        let limits = limits(LayoutKind::VerticalStack, mins, self.gap, 0, padded.width);
        let master_w = fit((padded.width as f64 * self.ratio) as i32, limits);

        let master = Rect::new(padded.x, padded.y, (master_w - half).max(1), padded.height);

        let stack_x = padded.x + master_w + half;
        let stack_w = (padded.width - master_w - half).max(1);
        // Stack slots are separated by a full `gap` so inner spacing matches
        // the outer padding.
        let area = Rect::new(stack_x, padded.y, stack_w, padded.height);
        let mut slots = column_slots(&area, windows.len() - 1, self.gap, stack);
        fit_column(&mut slots, mins.get(1..).unwrap_or_default(), self.gap);

        let mut results = Vec::with_capacity(windows.len());
        results.push((windows[0], master));
        results.extend(windows[1..].iter().copied().zip(slots));
        results
    }
}
