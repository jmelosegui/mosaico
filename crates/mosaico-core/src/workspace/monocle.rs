//! Monocle mode on a workspace, and the windows that a display change
//! brings onto a monocle workspace from another monitor (#132).

use crate::WindowId;

use super::Workspace;

/// What a workspace in monocle keeps of the windows a display change
/// brought from another monitor. It only holds windows that are on the
/// workspace, and is emptied when monocle ends.
#[derive(Default)]
pub(super) struct Visits {
    /// Windows still to be put behind the monocle window, then in its
    /// rect, the next time the workspace is laid out in monocle.
    waiting: Vec<WindowId>,
    /// Windows put behind the monocle window that way, the last one
    /// directly behind it.
    behind: Vec<WindowId>,
}

impl Visits {
    /// Forgets a window that left the workspace.
    pub(super) fn forget(&mut self, window: WindowId) {
        self.waiting.retain(|&w| w != window);
        self.behind.retain(|&w| w != window);
    }
}

impl Workspace {
    /// Returns whether monocle mode is active on this workspace.
    pub fn monocle(&self) -> bool {
        self.monocle
    }

    /// Sets the monocle mode flag. Leaving monocle forgets the windows
    /// that waited behind the monocle window.
    pub fn set_monocle(&mut self, value: bool) {
        self.monocle = value;
        if !value {
            self.visits = Visits::default();
        }
    }

    /// Returns the monocle window handle, if any.
    pub fn monocle_window(&self) -> Option<WindowId> {
        self.monocle_window
    }

    /// Sets the monocle window handle.
    pub fn set_monocle_window(&mut self, value: Option<WindowId>) {
        self.monocle_window = value;
    }

    /// Ends monocle when `window`, leaving the tiles, is the monocle window.
    pub fn end_monocle_of(&mut self, window: WindowId) {
        if self.monocle && self.monocle_window == Some(window) {
            self.set_monocle(false);
            self.monocle_window = None;
        }
    }

    /// Has a window of the workspace, which a display change brought from
    /// another monitor, wait behind the monocle window: the next time the
    /// workspace is laid out in monocle, it is put behind it, then in its
    /// rect, once.
    pub(crate) fn wait_behind(&mut self, window: WindowId) {
        if self.contains(window) && !self.visits.waiting.contains(&window) {
            self.visits.behind.retain(|&w| w != window);
            self.visits.waiting.push(window);
        }
    }

    /// Whether `window` waits to be put behind the monocle window.
    pub(crate) fn waits_behind(&self, window: WindowId) -> bool {
        self.visits.waiting.contains(&window)
    }

    /// The windows waiting behind `front`, the window monocle shows, in
    /// workspace order. They wait no longer once returned.
    pub(crate) fn take_waiting(&mut self, front: WindowId) -> Vec<WindowId> {
        let waiting = std::mem::take(&mut self.visits.waiting);
        let behind: Vec<WindowId> = self
            .windows
            .iter()
            .copied()
            .filter(|&w| w != front && waiting.contains(&w))
            .collect();
        self.visits.behind.retain(|&w| w != front);
        self.visits.behind.extend(&behind);
        behind
    }

    /// Once the monocle window has left, makes the window put behind it
    /// last, which is in front of the others, the monocle window, or else
    /// the first that still waits. Returns whether there was one, so the
    /// workspace stays in monocle.
    pub(crate) fn pass_monocle_on(&mut self) -> bool {
        let next = self.visits.behind.last().copied().or_else(|| {
            self.windows
                .iter()
                .copied()
                .find(|w| self.visits.waiting.contains(w))
        });
        let Some(next) = next else {
            return false;
        };
        self.monocle_window = Some(next);
        self.visits.forget(next);
        true
    }

    /// Makes `window`, which waits behind the monocle window, the monocle
    /// window, when the workspace has none on it. The windows that were
    /// put behind before wait behind it in turn, as they may be in front
    /// of it.
    pub(crate) fn hand_monocle_to(&mut self, window: WindowId) {
        self.monocle_window = Some(window);
        self.visits.forget(window);
        for behind in std::mem::take(&mut self.visits.behind) {
            self.wait_behind(behind);
        }
    }
}

#[cfg(test)]
#[path = "monocle_tests.rs"]
mod tests;
