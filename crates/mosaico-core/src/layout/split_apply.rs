//! Laying out a workspace's windows with the minimum sizes they have.

use crate::{LayoutKind, Rect, WindowId};

use super::heights::Heights;
use super::{BspLayout, MinSize, SplitRatios, ThreeColumnLayout, VerticalStackLayout};

impl SplitRatios {
    /// Lays out `windows` as [`apply`](Self::apply) does, with each window
    /// at least as big as its minimum in `mins`, one for each window, in
    /// order, or none at all, where the area has room for the minimums of
    /// every window on each side of a split and in each column. Each split
    /// is moved just far enough for that, whatever ratio it has, and a
    /// column whose window is below its minimum takes the height from
    /// the others in it that have room. Where the area cannot hold them
    /// all, the split or column is laid out as `apply` does, and a window
    /// goes beyond its slot.
    ///
    /// With no minimum to keep, the slots are those of `apply` to the pixel.
    pub fn apply_fitted(
        &self,
        kind: LayoutKind,
        gap: i32,
        ratio: f64,
        windows: &[WindowId],
        area: &Rect,
        mins: &[MinSize],
    ) -> Vec<(WindowId, Rect)> {
        match kind {
            LayoutKind::Bsp => {
                BspLayout { gap, ratio }.apply_fitted(windows, area, &self.bsp, mins)
            }
            LayoutKind::VerticalStack => {
                let ratio = self.vertical_stack.unwrap_or(ratio);
                VerticalStackLayout { gap, ratio }.apply_fitted(
                    windows,
                    area,
                    self.stack.ends(),
                    mins,
                )
            }
            LayoutKind::ThreeColumn => {
                let ratio = self.three_column.unwrap_or(ratio);
                let columns = self.columns.each_ref().map(Heights::ends);
                ThreeColumnLayout { gap, ratio }.apply_fitted(windows, area, columns, mins)
            }
        }
    }
}
