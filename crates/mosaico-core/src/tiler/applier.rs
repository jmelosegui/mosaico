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

use std::collections::HashSet;
use std::time::Instant;

use crate::{HidingSupport, WindowId};

use super::window_effects::place_and_repaint;
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
    /// Windows moved on screen without waiting, whose next placement is
    /// made in full. See `run`.
    moved_on_screen: HashSet<WindowId>,
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
            self.release_dropped_focus();
        }
    }

    /// Stops the echo filter waiting for a focus change the coalescer no
    /// longer holds, because its window left. Otherwise every later focus
    /// report would be taken for the system moving focus while ours
    /// waits.
    fn release_dropped_focus(&mut self) {
        if !self.held.holds_focus() {
            self.expectations.release_held_focus();
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
            self.release_dropped_focus();
            self.moved_on_screen.remove(&window);
        }
        self.expectations.echo(event, now)
    }

    fn run(&mut self, effect: &Effect, platform: &mut dyn Platform, now: Instant) {
        match *effect {
            // A move that does not wait may land before the window's
            // placement comes, and a placement that finds its window in
            // place is skipped, repaint and all. Nothing has repainted a
            // window moved that way while on screen, so its next placement
            // is made in full, where it would have been made anyway.
            Effect::PrePlace { window, .. } if platform.desktop().is_visible(window) => {
                self.moved_on_screen.insert(window);
                apply_window_effect(effect, platform.desktop(), platform.control());
            }
            Effect::Place { window, rect, .. } if self.moved_on_screen.remove(&window) => {
                place_and_repaint(window, rect, platform.control());
            }
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
            // Admitting it dropped what was held for the window, and the
            // platform has nothing to do.
            Effect::Untile { .. } => {}
            Effect::Decorate { .. }
            | Effect::Undecorate { .. }
            | Effect::Border { .. }
            | Effect::DropBorder { .. }
            | Effect::RedrawBorders => apply_decoration_effect(effect, platform.decorations()),
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
