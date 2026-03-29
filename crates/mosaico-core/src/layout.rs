use serde::{Deserialize, Serialize};

use crate::Rect;

/// Available tiling layout algorithms.
#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LayoutKind {
    /// Binary Space Partitioning — recursive subdivision.
    #[default]
    Bsp,
    /// Master pane on the left, vertical stack on the right.
    VerticalStack,
}

impl LayoutKind {
    /// Returns the next layout in the cycle.
    pub fn next(self) -> Self {
        match self {
            Self::Bsp => Self::VerticalStack,
            Self::VerticalStack => Self::Bsp,
        }
    }

    /// Short display name for the status bar.
    pub fn name(self) -> &'static str {
        match self {
            Self::Bsp => "BSP",
            Self::VerticalStack => "VStack",
        }
    }
}

/// A layout algorithm that computes window positions within a work area.
///
/// Given a list of window handles and the available space, a layout
/// produces a position and size for each window.
pub trait Layout {
    /// Computes positions for all windows in the given work area.
    ///
    /// Returns a list of (handle, rect) pairs in the same order as the
    /// input handles.
    fn apply(&self, handles: &[usize], work_area: &Rect) -> Vec<(usize, Rect)>;
}

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
    fn apply(&self, handles: &[usize], work_area: &Rect) -> Vec<(usize, Rect)> {
        if handles.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::with_capacity(handles.len());
        let padded = Rect::new(
            work_area.x + self.gap,
            work_area.y + self.gap,
            (work_area.width - self.gap * 2).max(1),
            (work_area.height - self.gap * 2).max(1),
        );
        self.split(handles, &padded, true, &mut results);
        results
    }
}

impl BspLayout {
    fn split(
        &self,
        handles: &[usize],
        area: &Rect,
        horizontal: bool,
        results: &mut Vec<(usize, Rect)>,
    ) {
        if handles.len() == 1 {
            results.push((handles[0], *area));
            return;
        }

        let half = self.gap / 2;
        let (first_area, rest_area) = if horizontal {
            let split = (area.width as f64 * self.ratio) as i32;
            let first = Rect::new(area.x, area.y, (split - half).max(1), area.height);
            let rest = Rect::new(
                area.x + split + half,
                area.y,
                (area.width - split - half).max(1),
                area.height,
            );
            (first, rest)
        } else {
            let split = (area.height as f64 * self.ratio) as i32;
            let first = Rect::new(area.x, area.y, area.width, (split - half).max(1));
            let rest = Rect::new(
                area.x,
                area.y + split + half,
                area.width,
                (area.height - split - half).max(1),
            );
            (first, rest)
        };

        results.push((handles[0], first_area));
        self.split(&handles[1..], &rest_area, !horizontal, results);
    }
}

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
                // Last window takes remaining space to avoid rounding gaps.
                (padded.y + padded.height - y).max(1)
            } else {
                slot_h.max(1)
            };
            results.push((hwnd, Rect::new(stack_x, y, stack_w, h)));
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_window_fills_work_area() {
        // Arrange
        let layout = BspLayout { gap: 0, ratio: 0.5 };
        let area = Rect::new(0, 0, 1920, 1080);

        // Act
        let result = layout.apply(&[1], &area);

        // Assert
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], (1, Rect::new(0, 0, 1920, 1080)));
    }

    #[test]
    fn two_windows_split_horizontally() {
        // Arrange
        let layout = BspLayout { gap: 0, ratio: 0.5 };
        let area = Rect::new(0, 0, 1920, 1080);

        // Act
        let result = layout.apply(&[1, 2], &area);

        // Assert
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], (1, Rect::new(0, 0, 960, 1080)));
        assert_eq!(result[1], (2, Rect::new(960, 0, 960, 1080)));
    }

    #[test]
    fn three_windows_bsp_split() {
        // Arrange
        let layout = BspLayout { gap: 0, ratio: 0.5 };
        let area = Rect::new(0, 0, 1920, 1080);

        // Act
        let result = layout.apply(&[1, 2, 3], &area);

        // Assert — first split horizontal, second vertical
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], (1, Rect::new(0, 0, 960, 1080)));
        assert_eq!(result[1], (2, Rect::new(960, 0, 960, 540)));
        assert_eq!(result[2], (3, Rect::new(960, 540, 960, 540)));
    }

    #[test]
    fn empty_handles_returns_empty() {
        // Arrange
        let layout = BspLayout::default();
        let area = Rect::new(0, 0, 1920, 1080);

        // Act
        let result = layout.apply(&[], &area);

        // Assert
        assert!(result.is_empty());
    }

    #[test]
    fn large_gap_never_produces_negative_dimensions() {
        // Arrange — gap is larger than the work area
        let layout = BspLayout {
            gap: 500,
            ratio: 0.5,
        };
        let area = Rect::new(0, 0, 200, 200);

        // Act
        let result = layout.apply(&[1, 2], &area);

        // Assert — all dimensions must be positive
        for (_hwnd, rect) in &result {
            assert!(rect.width > 0, "width was {}", rect.width);
            assert!(rect.height > 0, "height was {}", rect.height);
        }
    }

    // -- LayoutKind tests --

    #[test]
    fn layout_kind_cycles() {
        assert_eq!(LayoutKind::Bsp.next(), LayoutKind::VerticalStack);
        assert_eq!(LayoutKind::VerticalStack.next(), LayoutKind::Bsp);
    }

    #[test]
    fn layout_kind_names() {
        assert_eq!(LayoutKind::Bsp.name(), "BSP");
        assert_eq!(LayoutKind::VerticalStack.name(), "VStack");
    }

    // -- VerticalStack tests --

    #[test]
    fn vstack_single_window_fills_work_area() {
        let layout = VerticalStackLayout { gap: 0, ratio: 0.5 };
        let area = Rect::new(0, 0, 1920, 1080);
        let result = layout.apply(&[1], &area);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], (1, Rect::new(0, 0, 1920, 1080)));
    }

    #[test]
    fn vstack_two_windows_master_and_stack() {
        let layout = VerticalStackLayout { gap: 0, ratio: 0.5 };
        let area = Rect::new(0, 0, 1920, 1080);
        let result = layout.apply(&[1, 2], &area);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], (1, Rect::new(0, 0, 960, 1080)));
        assert_eq!(result[1], (2, Rect::new(960, 0, 960, 1080)));
    }

    #[test]
    fn vstack_three_windows_stack_splits_equally() {
        let layout = VerticalStackLayout { gap: 0, ratio: 0.5 };
        let area = Rect::new(0, 0, 1920, 1080);
        let result = layout.apply(&[1, 2, 3], &area);

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], (1, Rect::new(0, 0, 960, 1080)));
        assert_eq!(result[1], (2, Rect::new(960, 0, 960, 540)));
        assert_eq!(result[2], (3, Rect::new(960, 540, 960, 540)));
    }

    #[test]
    fn vstack_five_windows() {
        let layout = VerticalStackLayout { gap: 0, ratio: 0.5 };
        let area = Rect::new(0, 0, 1920, 1080);
        let result = layout.apply(&[1, 2, 3, 4, 5], &area);

        assert_eq!(result.len(), 5);
        // Master takes left half
        assert_eq!(result[0].1.width, 960);
        assert_eq!(result[0].1.height, 1080);
        // Stack windows all have same width
        for r in &result[1..] {
            assert_eq!(r.1.width, 960);
        }
        // Stack windows cover full height
        let stack_top = result[1].1.y;
        let stack_bottom = result[4].1.y + result[4].1.height;
        assert_eq!(stack_bottom - stack_top, 1080);
    }

    #[test]
    fn vstack_empty_returns_empty() {
        let layout = VerticalStackLayout::default();
        let area = Rect::new(0, 0, 1920, 1080);
        assert!(layout.apply(&[], &area).is_empty());
    }
}
