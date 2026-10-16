//! How each window the tiler hid was hidden.
//!
//! A window has to be shown again the way it was hidden: uncloaking a
//! window that was hidden, or showing one that was cloaked, leaves it
//! out of view. The configured strategy can change while windows are
//! hidden, so the tiler remembers the strategy of each window it hides
//! rather than reading the configuration again when it shows it (#92).

use std::collections::HashMap;

use crate::WindowId;
use crate::config::HidingBehaviour;

use super::{Effect, Plan};

/// The strategy each window the tiler hid was hidden with.
#[derive(Debug, Default)]
pub(super) struct HiddenHow {
    ways: HashMap<WindowId, HidingBehaviour>,
}

impl HiddenHow {
    /// Hides a window the way `how` says. A window the tiler already hid
    /// stays hidden the way it was.
    pub(super) fn hide(&mut self, plan: &mut Plan, window: WindowId, how: HidingBehaviour) {
        let how = *self.ways.entry(window).or_insert(how);
        plan.emit(Effect::Hide { window, how });
    }

    /// Shows a window again the way it was hidden, or the way `how` says
    /// when the tiler did not hide it.
    pub(super) fn show(&mut self, plan: &mut Plan, window: WindowId, how: HidingBehaviour) {
        let how = self.ways.remove(&window).unwrap_or(how);
        plan.emit(Effect::Show { window, how });
    }

    /// Drops a window that leaves the layout, so it, or a new window
    /// given the same id, is not hidden or shown the way this one was.
    pub(super) fn forget(&mut self, window: WindowId) {
        self.ways.remove(&window);
    }

    /// Whether the tiler hid the window and has not shown it since.
    pub(super) fn contains(&self, window: WindowId) -> bool {
        self.ways.contains_key(&window)
    }

    /// Shows a window the tiler is letting go of, the way it hid it, so
    /// it is not left out of view with nothing to bring it back. Does
    /// nothing for a window the tiler did not hide.
    pub(super) fn release(&mut self, plan: &mut Plan, window: WindowId) {
        if let Some(how) = self.ways.remove(&window) {
            plan.emit(Effect::Show { window, how });
        }
    }
}
