use crate::{LayoutKind, Rect, WindowId};

use super::Layout;
use super::column::column_slots;
use super::min_column::fit_column;
use super::min_limits::{fit, limits};
use super::min_size::MinSize;

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

impl Layout for ThreeColumnLayout {
    fn apply(&self, windows: &[WindowId], work_area: &Rect) -> Vec<(WindowId, Rect)> {
        self.apply_heights(windows, work_area, [None, None])
    }
}

impl ThreeColumnLayout {
    /// Lays out `windows` as [`Layout::apply`] does, with the windows of the
    /// left column and of the right one at the heights `columns` gives
    /// them: where each but the last ends, as a share of the column without
    /// its gaps, or shared equally without them. The windows after the
    /// master alternate between the left column and the right one.
    pub fn apply_heights(
        &self,
        windows: &[WindowId],
        work_area: &Rect,
        columns: [Option<&[f64]>; 2],
    ) -> Vec<(WindowId, Rect)> {
        self.apply_fitted(windows, work_area, columns, &[])
    }

    /// Lays out `windows` as [`apply_heights`](Self::apply_heights) does,
    /// with the master's width kept where the master and the columns beside
    /// it keep their minimum sizes, `mins`, one for each window or none at
    /// all, and the windows of each column tall enough for theirs, the
    /// height coming from those with room. What cannot leave every window
    /// its minimum is left as it would be.
    pub fn apply_fitted(
        &self,
        windows: &[WindowId],
        work_area: &Rect,
        columns: [Option<&[f64]>; 2],
        mins: &[MinSize],
    ) -> Vec<(WindowId, Rect)> {
        let [left_heights, right_heights] = columns;
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
        let limits = limits(LayoutKind::ThreeColumn, mins, self.gap, 0, padded.width);
        let master_w = fit((padded.width as f64 * self.ratio) as i32, limits);

        // 2 windows: master left, second right.
        if windows.len() == 2 {
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

        let sides = &windows[1..];
        let left: Vec<WindowId> = sides.iter().copied().step_by(2).collect();
        let right: Vec<WindowId> = sides.iter().copied().skip(1).step_by(2).collect();

        // Each column's slots are separated by a full `gap`, so the inner
        // spacing matches the outer padding.
        let left_area = Rect::new(left_x, padded.y, side_w.max(1), padded.height);
        let mut left_slots = column_slots(&left_area, left.len(), self.gap, left_heights);
        fit_column(&mut left_slots, &column_mins(mins, 0), self.gap);
        results.extend(left.into_iter().zip(left_slots));

        let right_area = Rect::new(right_x, padded.y, right_w, padded.height);
        let mut right_slots = column_slots(&right_area, right.len(), self.gap, right_heights);
        fit_column(&mut right_slots, &column_mins(mins, 1), self.gap);
        results.extend(right.into_iter().zip(right_slots));

        results
    }
}

/// The minimums of the windows of the left column, or of the right one,
/// top down: the windows after the master alternate between them. Empty
/// when the layout has no minimums.
fn column_mins(mins: &[MinSize], column: usize) -> Vec<MinSize> {
    let sides = mins.get(1..).unwrap_or_default();
    sides.iter().copied().skip(column).step_by(2).collect()
}
