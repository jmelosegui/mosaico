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

use crate::Rect;
use crate::config::HidingBehaviour;

use super::{Desktop, Effect, WindowFacts};

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
    planned: HashMap<usize, Planned>,
}

impl<'a> Plan<'a> {
    pub(super) fn new(desktop: &'a dyn Desktop, now: Instant) -> Self {
        Self {
            desktop,
            now,
            effects: Vec::new(),
            planned: HashMap::new(),
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
                let p = self.planned.entry(window).or_default();
                p.rect = Some(rect);
                // Positioning a window takes it out of maximized and
                // minimized state.
                p.minimized = Some(false);
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

    /// Ends the step, returning its effects in order.
    pub(super) fn finish(self) -> Vec<Effect> {
        self.effects
    }

    fn planned(&self, window: usize) -> Planned {
        self.planned.get(&window).copied().unwrap_or_default()
    }

    pub(super) fn is_visible(&self, window: usize) -> bool {
        self.planned(window)
            .visible
            .unwrap_or_else(|| self.desktop.is_visible(window))
    }

    pub(super) fn is_minimized(&self, window: usize) -> bool {
        self.planned(window)
            .minimized
            .unwrap_or_else(|| self.desktop.is_minimized(window))
    }

    pub(super) fn is_maximized(&self, window: usize) -> bool {
        // A window placed this step is no longer maximized.
        self.planned(window).rect.is_none() && self.desktop.is_maximized(window)
    }

    pub(super) fn rect(&self, window: usize) -> Option<Rect> {
        self.planned(window)
            .rect
            .or_else(|| self.desktop.rect(window))
    }

    pub(super) fn owner(&self, window: usize) -> Option<usize> {
        self.desktop.owner(window)
    }

    pub(super) fn monitor_of(&self, window: usize) -> usize {
        self.desktop.monitor_of(window)
    }

    pub(super) fn cursor(&self) -> Option<(i32, i32)> {
        self.desktop.cursor()
    }

    pub(super) fn candidate(&self, window: usize) -> Option<WindowFacts> {
        self.desktop.candidate(window)
    }
}
