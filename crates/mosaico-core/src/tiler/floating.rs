//! Floating windows: managed on a workspace and shown and hidden with it,
//! but placed by the user rather than by the layout, and kept above its
//! tiles.

use std::collections::{HashMap, HashSet};

use crate::{Rect, WindowId};

use super::{Plan, Tiler};

#[path = "floating_adopt.rs"]
mod adopt;
#[path = "floating_display.rs"]
mod display;
#[path = "floating_keys.rs"]
mod keys;
#[path = "floating_place.rs"]
pub(super) mod place;
#[path = "floating_rules.rs"]
mod rules;
#[path = "floating_stack.rs"]
mod stack;
#[path = "floating_toggle.rs"]
mod toggle;

pub(super) use place::Axis;

/// Where each window last floated in this session, so floating it again
/// puts it back there, and which floating windows left their workspace
/// by hiding or minimizing, so they float again when they come back.
#[derive(Debug, Default)]
pub(super) struct Floats {
    rects: HashMap<WindowId, Rect>,
    away: HashSet<WindowId>,
}

impl Floats {
    /// Remembers where a window floated.
    pub(super) fn remember(&mut self, window: WindowId, rect: Rect) {
        self.rects.insert(window, rect);
    }

    /// Where a window last floated.
    pub(super) fn remembered(&self, window: WindowId) -> Option<Rect> {
        self.rects.get(&window).copied()
    }

    /// Where a window lands as it starts floating in `area`: where it last
    /// floated, moved into `area`, or else centered at its `natural` size,
    /// shrunk to fit, or else centered at two-thirds of `area`.
    pub(super) fn landing(&self, window: WindowId, area: Rect, natural: Option<Rect>) -> Rect {
        match (self.remembered(window), natural) {
            (Some(rect), _) => place::clamp_into(rect, area),
            (None, Some(rect)) => place::centered(area, rect.width, rect.height),
            (None, None) => place::first(area),
        }
    }

    /// Notes that a floating window left its workspace while it still
    /// exists, at `rect`, so it floats there again when it comes back.
    pub(super) fn went_away(&mut self, window: WindowId, rect: Option<Rect>) {
        if let Some(rect) = rect {
            self.remember(window, rect);
        }
        self.away.insert(window);
    }

    /// Whether a window floated when it left its workspace.
    fn is_away(&self, window: WindowId) -> bool {
        self.away.contains(&window)
    }

    /// Notes that a window that floated when it left is back.
    fn came_back(&mut self, window: WindowId) {
        self.away.remove(&window);
    }

    /// Forgets a window that is gone, so a new window given its id does
    /// not float where it did.
    pub(super) fn forget(&mut self, window: WindowId) {
        self.rects.remove(&window);
        self.away.remove(&window);
    }
}

impl Tiler {
    /// Whether a window being taken in floats: one the rules float, or one
    /// that floated when it left its workspace.
    pub(super) fn floats_on_return(&self, plan: &Plan, window: WindowId) -> bool {
        self.floats.is_away(window) || self.rules_float(plan, window)
    }

    /// Hides the floating windows of the workspace on screen of monitor
    /// `mi` while it is in monocle, and shows them again once it is not.
    /// Every layout pass calls it, so whatever ends monocle shows them.
    pub(super) fn floats_follow_monocle(&mut self, plan: &mut Plan, mi: usize) {
        let Some(mon) = self.monitors.get(mi) else {
            return;
        };
        let ws = mon.active_ws();
        let how = self.layout.hiding;
        for &window in ws.floating() {
            match (ws.monocle(), self.hidden_how.contains(window)) {
                (true, false) => self.hidden_how.hide(plan, window, how),
                (false, true) => self.hidden_how.show(plan, window, how),
                _ => {}
            }
        }
    }
}

#[cfg(test)]
#[path = "tests/floating_toggle_tests.rs"]
mod toggle_tests;

#[cfg(test)]
#[path = "tests/floating_keys_tests.rs"]
mod keys_tests;

#[cfg(test)]
#[path = "tests/floating_mode_tests.rs"]
mod mode_tests;

#[cfg(test)]
#[path = "tests/floating_life_tests.rs"]
mod life_tests;

#[cfg(test)]
#[path = "tests/floating_retile_tests.rs"]
mod retile_tests;

#[cfg(test)]
#[path = "tests/floating_rules_tests.rs"]
mod rules_tests;

#[cfg(test)]
#[path = "tests/floating_stack_tests.rs"]
mod stack_tests;
