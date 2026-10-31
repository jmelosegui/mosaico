mod floating;
mod monocle;
mod splits;

use crate::WindowId;
use crate::layout::{LayoutKind, SplitRatios};

use monocle::Visits;

/// A workspace manages a set of tiled windows on a single monitor.
///
/// It maintains the window ordering (which determines layout positions)
/// and delegates positioning to a `Layout` implementation.
#[derive(Default)]
pub struct Workspace {
    /// Ordered list of managed window windows.
    windows: Vec<WindowId>,
    /// Managed windows the layout leaves alone, back to front (see
    /// `workspace/floating.rs`).
    floating: Vec<WindowId>,
    /// Whether monocle (single-window fullscreen) mode is active.
    monocle: bool,
    /// The window shown fullscreen in monocle mode.
    monocle_window: Option<WindowId>,
    /// What monocle keeps of the windows a display change brought from
    /// another monitor (see `workspace/monocle.rs`).
    visits: Visits,
    /// The last focused window on this workspace.
    ///
    /// Saved when switching away, restored when switching back so the
    /// user returns to the window they were working on.
    last_focused: Option<WindowId>,
    /// The active layout algorithm for this workspace.
    layout_kind: LayoutKind,
    /// Where the user moved the splits of each layout, kept in step with
    /// `windows` (see `workspace/splits.rs`).
    splits: SplitRatios,
}

impl Workspace {
    /// Creates an empty workspace with the default layout.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an empty workspace with the given layout.
    pub fn with_layout(layout_kind: LayoutKind) -> Self {
        Self {
            layout_kind,
            ..Self::new()
        }
    }

    /// Returns the last focused window on this workspace.
    pub fn last_focused(&self) -> Option<WindowId> {
        self.last_focused
    }

    /// Sets the last focused window on this workspace.
    pub fn set_last_focused(&mut self, value: Option<WindowId>) {
        self.last_focused = value;
    }

    /// Returns the active layout kind for this workspace.
    pub fn layout_kind(&self) -> LayoutKind {
        self.layout_kind
    }

    /// Sets the active layout kind for this workspace.
    pub fn set_layout_kind(&mut self, kind: LayoutKind) {
        self.layout_kind = kind;
    }

    /// Adds a window to the end of the workspace.
    ///
    /// Returns `false` if the window is already managed.
    pub fn add(&mut self, window: WindowId) -> bool {
        self.insert(self.windows.len(), window)
    }

    /// Inserts a window at a specific position in the workspace.
    ///
    /// The index is clamped to the current length.
    /// Returns `false` if the window is already managed, tiled or
    /// floating.
    pub fn insert(&mut self, index: usize, window: WindowId) -> bool {
        if self.holds(window) {
            return false;
        }
        let pos = index.min(self.windows.len());
        self.splits.insert(pos, self.windows.len());
        self.windows.insert(pos, window);
        true
    }

    /// Removes a window from the workspace.
    ///
    /// Returns `true` if the window was found and removed.
    pub fn remove(&mut self, window: WindowId) -> bool {
        if let Some(pos) = self.windows.iter().position(|&h| h == window) {
            self.splits.remove(pos, self.windows.len());
            self.windows.remove(pos);
            self.visits.forget(window);
            if self.last_focused == Some(window) {
                self.last_focused = None;
            }
            true
        } else {
            false
        }
    }

    /// Returns whether the workspace manages the given window.
    pub fn contains(&self, window: WindowId) -> bool {
        self.windows.contains(&window)
    }

    /// Returns the number of managed windows.
    pub fn len(&self) -> usize {
        self.windows.len()
    }

    /// Returns whether the workspace has no managed windows.
    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }

    /// Returns the ordered list of managed window windows.
    pub fn windows(&self) -> &[WindowId] {
        &self.windows
    }

    /// Swaps two windows by their position indices.
    pub fn swap(&mut self, a: usize, b: usize) {
        if a < self.windows.len() && b < self.windows.len() {
            self.windows.swap(a, b);
        }
    }

    /// Returns the index of the given window handle, if managed.
    pub fn index_of(&self, window: WindowId) -> Option<usize> {
        self.windows.iter().position(|&h| h == window)
    }
}

#[cfg(test)]
#[path = "workspace_tests.rs"]
mod tests;
