//! The effects of one step, and the desktop as it will look once they
//! are applied.
//!
//! The tiler decides a whole step before anything is applied, yet some
//! decisions depend on earlier ones in the same step: a workspace switch
//! shows windows and then lays them out, and the layout skips windows
//! that are invisible. Reading the real desktop there would see the
//! windows still hidden. [`Plan`] answers queries for windows touched
//! earlier in the step from the effects themselves, and passes the rest
//! through to the [`Desktop`].

use std::collections::HashMap;
use std::time::Instant;

use crate::config::HidingBehaviour;
use crate::{MonitorId, Rect, WindowId};

use super::{Desktop, Effect, WindowFacts};

#[path = "plan_borders.rs"]
mod borders;
#[path = "plan_maximized.rs"]
mod maximized;
#[path = "plan_transitions.rs"]
mod transitions;

pub(super) use transitions::StepKind;

/// What a window will look like after this step's effects.
#[derive(Debug, Default, Clone, Copy)]
struct Planned {
    rect: Option<Rect>,
    visible: Option<bool>,
    minimized: Option<bool>,
}

/// Effects of the current step plus the desktop they will produce.
pub(super) struct Plan<'a> {
    desktop: &'a dyn Desktop,
    now: Instant,
    effects: Vec<Effect>,
    planned: HashMap<WindowId, Planned>,
    /// What the step did that decides which placements may animate.
    placements: transitions::Placements,
}

impl<'a> Plan<'a> {
    pub(super) fn new(desktop: &'a dyn Desktop, now: Instant) -> Self {
        Self {
            desktop,
            now,
            effects: Vec::new(),
            planned: HashMap::new(),
            placements: transitions::Placements::default(),
        }
    }

    /// When the step is happening.
    pub(super) fn now(&self) -> Instant {
        self.now
    }

    /// Adds an effect to the step.
    pub(super) fn emit(&mut self, effect: Effect) {
        match effect {
            Effect::Place { window, rect, .. } => {
                // Positioning a window takes it out of maximized state
                // (see `is_maximized`), but a minimized one stays
                // minimized: the tiler restores a window before it places
                // it (see `WindowControl::place`).
                self.planned.entry(window).or_default().rect = Some(rect);
            }
            Effect::Show { window, .. } | Effect::Reveal { window } => {
                let p = self.planned.entry(window).or_default();
                p.visible = Some(true);
                p.minimized = Some(false);
            }
            Effect::Hide { window, how } => {
                let p = self.planned.entry(window).or_default();
                match how {
                    // A cloaked window still counts as shown.
                    HidingBehaviour::Cloak => {}
                    HidingBehaviour::Hide => p.visible = Some(false),
                    HidingBehaviour::Minimize => p.minimized = Some(true),
                }
            }
            Effect::Minimize { window } => {
                self.planned.entry(window).or_default().minimized = Some(true);
            }
            _ => {}
        }
        self.effects.push(effect);
    }

    /// Ends the step, returning its effects in order, with the placements
    /// that may animate marked so, see [`mark`](transitions::mark), and at
    /// most one border per window: the last one decided, see
    /// [`keep_last_borders`](borders::keep_last_borders).
    pub(super) fn finish(mut self, step: &StepKind) -> Vec<Effect> {
        transitions::mark(&mut self.effects, &self.placements, step, self.desktop);
        borders::keep_last_borders(self.effects)
    }

    fn planned(&self, window: WindowId) -> Planned {
        self.planned.get(&window).copied().unwrap_or_default()
    }

    pub(super) fn is_visible(&self, window: WindowId) -> bool {
        self.planned(window)
            .visible
            .unwrap_or_else(|| self.desktop.is_visible(window))
    }

    pub(super) fn is_minimized(&self, window: WindowId) -> bool {
        self.planned(window)
            .minimized
            .unwrap_or_else(|| self.desktop.is_minimized(window))
    }

    pub(super) fn rect(&self, window: WindowId) -> Option<Rect> {
        self.planned(window)
            .rect
            .or_else(|| self.desktop.rect(window))
    }

    pub(super) fn owner(&self, window: WindowId) -> Option<WindowId> {
        self.desktop.owner(window)
    }

    pub(super) fn monitor_of(&self, window: WindowId) -> MonitorId {
        self.desktop.monitor_of(window)
    }

    pub(super) fn cursor(&self) -> Option<(i32, i32)> {
        self.desktop.cursor()
    }

    pub(super) fn candidate(&self, window: WindowId) -> Option<WindowFacts> {
        self.desktop.candidate(window)
    }

    pub(super) fn facts(&self, window: WindowId) -> Option<WindowFacts> {
        self.desktop.facts(window)
    }
}
