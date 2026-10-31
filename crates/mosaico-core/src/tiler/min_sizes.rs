//! How small each managed window may be, read once and remembered.

use std::collections::HashMap;

use crate::{MinSize, WindowId};

use super::{Plan, Tiler};

/// The minimum size of each window the tiler laid out, as the desktop told
/// it when it first did. A window it has none for is remembered too, so
/// the application is not asked again.
#[derive(Debug, Default)]
pub(super) struct MinSizes(HashMap<WindowId, MinSize>);

impl MinSizes {
    /// Forgets every window's minimum, so the next layout asks again.
    pub(super) fn clear(&mut self) {
        self.0.clear();
    }
}

impl Tiler {
    /// Reads the minimum size of each window of the active workspace of
    /// monitor `idx` that has none remembered, and forgets the windows the
    /// tiler no longer manages.
    pub(super) fn learn_min_sizes(&mut self, plan: &Plan, idx: usize) {
        let Some(state) = self.monitors.get(idx) else {
            return;
        };
        for &window in state.active_ws().windows() {
            self.min_sizes
                .0
                .entry(window)
                .or_insert_with(|| plan.min_size(window).unwrap_or_default());
        }
        let monitors = &self.monitors;
        let managed = |window: &WindowId| {
            monitors
                .iter()
                .any(|m| m.workspaces.iter().any(|ws| ws.holds(*window)))
        };
        self.min_sizes.0.retain(|window, _| managed(window));
    }

    /// Reads the minimum size of `window`, which is not managed yet, for
    /// the layout that places it before it is shown.
    pub(super) fn learn_min_size(&mut self, plan: &Plan, window: WindowId) {
        let size = plan.min_size(window).unwrap_or_default();
        self.min_sizes.0.entry(window).or_insert(size);
    }

    /// The minimum size of each of `windows`, in order, or none at all
    /// when no window has one, so that a layout without any is laid out
    /// as it always was.
    pub(super) fn min_sizes_of(&self, windows: &[WindowId]) -> Vec<MinSize> {
        let sizes: Vec<MinSize> = windows
            .iter()
            .map(|window| self.min_sizes.0.get(window).copied().unwrap_or_default())
            .collect();
        if sizes.iter().all(|size| size.is_none()) {
            return Vec::new();
        }
        sizes
    }
}
