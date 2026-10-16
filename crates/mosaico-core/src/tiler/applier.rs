//! Applies the tiler's effects through the platform.
//!
//! Effects that change a window or move the pointer are carried out by
//! [`apply_window_effect`] through the platform's window control, and
//! borders and window corners by [`apply_decoration_effect`] through its
//! decorations. Most effects are applied as they arrive. Two kinds are
//! held back while the user is still pressing keys, and only the latest
//! of each is applied once input goes quiet:
//!
//! - focus changes, because focusing another application's window is a
//!   cross-process call that often takes 50-150 ms;
//! - coalesced placements from moves, so a burst of moves repaints each
//!   window once. A window's border and cursor moves wait with it, so
//!   the border never shows at a slot the window has not reached yet.
//!
//! Every applied effect is recorded in [`Expectations`] so the events it
//! causes can be told apart from the user's.

use std::time::Instant;

use crate::HidingSupport;

use super::{
    Coalescer, Effect, Expectations, HiddenFile, HiddenWindows, Platform, apply_decoration_effect,
    apply_window_effect,
};

#[path = "applier_hidden.rs"]
mod hidden;

/// Executes effects and remembers what the OS should report back.
#[derive(Default)]
pub(super) struct Applier {
    /// Effects held back until input goes quiet.
    held: Coalescer,
    expectations: Expectations,
    /// Windows hidden by workspace switching, so the next daemon can
    /// show them again if this one is killed.
    hidden: HiddenWindows,
    /// Where `hidden` is kept, if the instance has a data directory.
    hidden_file: Option<HiddenFile>,
}

impl Applier {
    /// An applier that keeps its record of hidden windows in `file`, on
    /// a desktop whose ways of hiding report what `hiding` says.
    pub(super) fn new(file: Option<HiddenFile>, hiding: HidingSupport) -> Self {
        Self {
            expectations: Expectations::new(hiding),
            hidden_file: file,
            ..Self::default()
        }
    }

    fn save_hidden(&self) {
        if let Some(file) = &self.hidden_file {
            file.save(&self.hidden);
        }
    }

    /// Applies effects in order, holding back the ones that may wait.
    pub(super) fn apply(
        &mut self,
        effects: Vec<Effect>,
        platform: &mut dyn Platform,
        now: Instant,
    ) {
        for effect in effects {
            if matches!(effect, Effect::Focus { .. }) {
                self.expectations.held(&effect);
            }
            if let Some(effect) = self.held.admit(effect, now) {
                self.run(&effect, platform, now);
            }
        }
    }

    /// When what is held back started being held, if anything is.
    pub(super) fn held_since(&self) -> Option<Instant> {
        self.held.held_since()
    }

    /// Applies everything held back.
    pub(super) fn flush(&mut self, platform: &mut dyn Platform, now: Instant) {
        for effect in self.held.take() {
            self.run(&effect, platform, now);
        }
    }

    /// Why an event is an echo of our own effects, if it is one.
    pub(super) fn echo(
        &mut self,
        event: &crate::WindowEvent,
        now: Instant,
    ) -> Option<&'static str> {
        // Nothing held for a window that no longer exists may be applied.
        if let crate::WindowEvent::Destroyed { window } = *event {
            self.held.forget(window);
        }
        self.expectations.echo(event, now)
    }

    fn run(&mut self, effect: &Effect, platform: &mut dyn Platform, now: Instant) {
        match effect {
            Effect::Place { .. }
            | Effect::PrePlace { .. }
            | Effect::Hide { .. }
            | Effect::Show { .. }
            | Effect::Reveal { .. }
            | Effect::Focus { .. }
            | Effect::CenterCursor { .. }
            | Effect::Close { .. }
            | Effect::Minimize { .. }
            | Effect::Repaint { .. }
            | Effect::PutBehind { .. } => {
                apply_window_effect(effect, platform.desktop(), platform.control())
            }
            Effect::Decorate { .. }
            | Effect::Undecorate { .. }
            | Effect::Border { .. }
            | Effect::DropBorder { .. } => apply_decoration_effect(effect, platform.decorations()),
        }
        self.expectations.applied(effect, now);
        if self
            .hidden
            .applied(effect, |w| platform.desktop().identity(w))
        {
            self.save_hidden();
        }
    }
}
