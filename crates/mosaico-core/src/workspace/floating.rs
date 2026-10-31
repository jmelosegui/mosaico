//! The floating windows of a workspace: managed, but not tiled.

use crate::WindowId;

use super::Workspace;

impl Workspace {
    /// The floating windows, back to front.
    pub fn floating(&self) -> &[WindowId] {
        &self.floating
    }

    /// Whether the window floats on this workspace.
    pub fn is_floating(&self, window: WindowId) -> bool {
        self.floating.contains(&window)
    }

    /// Whether the workspace holds the window, tiled or floating.
    pub fn holds(&self, window: WindowId) -> bool {
        self.contains(window) || self.is_floating(window)
    }

    /// Adds a floating window in front of the others. Returns `false`,
    /// and changes nothing, when the workspace already holds it.
    pub fn add_floating(&mut self, window: WindowId) -> bool {
        if self.holds(window) {
            return false;
        }
        self.floating.push(window);
        true
    }

    /// Removes a floating window. Returns whether it floated here.
    pub fn remove_floating(&mut self, window: WindowId) -> bool {
        if self.last_focused == Some(window) {
            self.last_focused = None;
        }
        self.take_floating(window)
    }

    /// Brings a floating window in front of the others.
    pub fn raise_floating(&mut self, window: WindowId) {
        if self.take_floating(window) {
            self.floating.push(window);
        }
    }

    /// The windows on screen while the workspace is: its tiles, then its
    /// floating windows back to front, which monocle hides.
    pub fn shown(&self) -> Vec<WindowId> {
        let mut shown = self.windows.clone();
        if !self.monocle {
            shown.extend_from_slice(&self.floating);
        }
        shown
    }

    fn take_floating(&mut self, window: WindowId) -> bool {
        let before = self.floating.len();
        self.floating.retain(|&w| w != window);
        self.floating.len() != before
    }
}

#[cfg(test)]
#[path = "floating_tests.rs"]
mod tests;
