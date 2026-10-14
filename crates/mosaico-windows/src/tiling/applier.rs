//! Applies the tiler's effects on the Windows desktop.
//!
//! Effects that change a window or move the pointer are carried out by
//! [`apply_window_effect`] through the [`WindowControl`] it is given,
//! and borders and window corners by [`apply_decoration_effect`]
//! through the applier's own [`Win32Decorations`]. Most effects are
//! applied as they arrive. Two kinds are held back while the user is
//! still pressing keys, and only the latest of each is applied once
//! input goes quiet:
//!
//! - focus changes, because `SetForegroundWindow` is a cross-process
//!   call that often takes 50-150 ms;
//! - coalesced placements from moves, so a burst of moves repaints each
//!   window once. A window's border and cursor moves wait with it, so
//!   the border never shows at a slot the window has not reached yet.
//!
//! Every applied effect is recorded in [`Expectations`] so the events it
//! causes can be told apart from the user's.

use std::time::Instant;

use mosaico_core::tiler::{
    Coalescer, Desktop, Effect, Expectations, HiddenWindows, apply_decoration_effect,
    apply_window_effect,
};
use mosaico_core::{HidingSupport, WindowControl, WindowId};

use crate::decorations::Win32Decorations;

use super::hidden_file::HiddenFile;

/// Executes effects and remembers what the OS should report back.
#[derive(Default)]
pub(super) struct Applier {
    /// The border overlays, and the corners of managed windows.
    decorations: Win32Decorations,
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

    /// Shows again the windows a previous daemon hid and never showed,
    /// because it was killed or crashed, then clears its record.
    ///
    /// Run before the first windows are enumerated, so they are adopted
    /// like any other window.
    pub(super) fn restore_previous(
        &mut self,
        desktop: &dyn Desktop,
        control: &dyn WindowControl,
        now: Instant,
    ) {
        let Some(file) = &self.hidden_file else {
            return;
        };
        let effects = file.load().restore(|w| desktop.identity(w));
        if !effects.is_empty() {
            mosaico_core::log_info!(
                "showing {} window(s) a previous daemon left hidden",
                effects.len()
            );
        }
        for effect in &effects {
            self.run(effect, desktop, control, now);
        }
        self.save_hidden();
    }

    /// Forgets a window that was destroyed.
    pub(super) fn forget(&mut self, window: WindowId) {
        if self.hidden.forget(window) {
            self.save_hidden();
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
        desktop: &dyn Desktop,
        control: &dyn WindowControl,
        now: Instant,
    ) {
        for effect in effects {
            if matches!(effect, Effect::Focus { .. }) {
                self.expectations.held(&effect);
            }
            if let Some(effect) = self.held.admit(effect, now) {
                self.run(&effect, desktop, control, now);
            }
        }
    }

    /// When what is held back started being held, if anything is.
    pub(super) fn held_since(&self) -> Option<Instant> {
        self.held.held_since()
    }

    /// Applies everything held back.
    pub(super) fn flush(
        &mut self,
        desktop: &dyn Desktop,
        control: &dyn WindowControl,
        now: Instant,
    ) {
        for effect in self.held.take() {
            self.run(&effect, desktop, control, now);
        }
    }

    /// Why an event is an echo of our own effects, if it is one.
    pub(super) fn echo(
        &mut self,
        event: &mosaico_core::WindowEvent,
        now: Instant,
    ) -> Option<&'static str> {
        // Nothing held for a window that no longer exists may be applied.
        if let mosaico_core::WindowEvent::Destroyed { window } = *event {
            self.held.forget(window);
        }
        self.expectations.echo(event, now)
    }

    fn run(
        &mut self,
        effect: &Effect,
        desktop: &dyn Desktop,
        control: &dyn WindowControl,
        now: Instant,
    ) {
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
            | Effect::Repaint { .. } => apply_window_effect(effect, desktop, control),
            Effect::Decorate { .. }
            | Effect::Undecorate { .. }
            | Effect::Border { .. }
            | Effect::DropBorder { .. } => apply_decoration_effect(effect, &mut self.decorations),
        }
        self.expectations.applied(effect, now);
        if self.hidden.applied(effect, |w| desktop.identity(w)) {
            self.save_hidden();
        }
    }
}
