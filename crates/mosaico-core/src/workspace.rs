use crate::Rect;
use crate::layout::Layout;

/// A workspace manages a set of tiled windows on a single monitor.
///
/// It maintains the window ordering (which determines layout positions)
/// and delegates positioning to a `Layout` implementation.
pub struct Workspace {
    /// Ordered list of managed window handles.
    handles: Vec<usize>,
}

impl Workspace {
    /// Creates an empty workspace.
    pub fn new() -> Self {
        Self {
            handles: Vec::new(),
        }
    }

    /// Adds a window to the workspace.
    ///
    /// Returns `false` if the window is already managed.
    pub fn add(&mut self, hwnd: usize) -> bool {
        if self.handles.contains(&hwnd) {
            return false;
        }
        self.handles.push(hwnd);
        true
    }

    /// Removes a window from the workspace.
    ///
    /// Returns `true` if the window was found and removed.
    pub fn remove(&mut self, hwnd: usize) -> bool {
        if let Some(pos) = self.handles.iter().position(|&h| h == hwnd) {
            self.handles.remove(pos);
            true
        } else {
            false
        }
    }

    /// Returns whether the workspace manages the given window.
    pub fn contains(&self, hwnd: usize) -> bool {
        self.handles.contains(&hwnd)
    }

    /// Returns the number of managed windows.
    pub fn len(&self) -> usize {
        self.handles.len()
    }

    /// Returns whether the workspace has no managed windows.
    pub fn is_empty(&self) -> bool {
        self.handles.is_empty()
    }

    /// Returns the ordered list of managed window handles.
    pub fn handles(&self) -> &[usize] {
        &self.handles
    }

    /// Computes the layout for all managed windows in the given work area.
    ///
    /// Returns a list of (handle, rect) pairs.
    pub fn compute_layout(&self, layout: &dyn Layout, work_area: &Rect) -> Vec<(usize, Rect)> {
        layout.apply(&self.handles, work_area)
    }

    /// Swaps two windows by their position indices.
    pub fn swap(&mut self, a: usize, b: usize) {
        if a < self.handles.len() && b < self.handles.len() {
            self.handles.swap(a, b);
        }
    }

    /// Returns the index of the given window handle, if managed.
    pub fn index_of(&self, hwnd: usize) -> Option<usize> {
        self.handles.iter().position(|&h| h == hwnd)
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::BspLayout;

    #[test]
    fn add_and_remove_windows() {
        // Arrange
        let mut ws = Workspace::new();

        // Act
        assert!(ws.add(1));
        assert!(ws.add(2));
        assert!(!ws.add(1)); // duplicate

        // Assert
        assert_eq!(ws.len(), 2);
        assert!(ws.contains(1));
        assert!(ws.remove(1));
        assert_eq!(ws.len(), 1);
        assert!(!ws.contains(1));
    }

    #[test]
    fn compute_layout_delegates_to_layout() {
        // Arrange
        let mut ws = Workspace::new();
        ws.add(100);
        ws.add(200);
        let layout = BspLayout { gap: 0, ratio: 0.5 };
        let area = Rect::new(0, 0, 1920, 1080);

        // Act
        let positions = ws.compute_layout(&layout, &area);

        // Assert
        assert_eq!(positions.len(), 2);
        assert_eq!(positions[0].0, 100);
        assert_eq!(positions[1].0, 200);
    }
}
