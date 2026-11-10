//! What the tiling manager shows the bars: the bar API's state, built
//! again only when something it shows may have changed.

use std::path::Path;

use crate::api::{self, MAX_TITLE_CHARS};
use crate::{WindowEvent, WindowId};

use super::super::{Desktop, Input, Platform};
use super::TilingManager;

/// The state last shown, and what may have changed since.
#[derive(Debug, Default)]
pub(super) struct ApiCache {
    state: api::State,
    /// Whether `state` was ever built.
    built: bool,
    /// Whether a step may have changed the monitors or the focus.
    dirty: bool,
    /// The focused window's title and program, read when it got focus or
    /// its title last changed.
    window: Option<api::FocusedWindow>,
    /// Whether the focused window's title changed since it was read.
    title_stale: bool,
}

impl ApiCache {
    /// Notes that `input` is about to be stepped. A window moving, which
    /// comes on every frame of a drag or an animation, changes nothing a
    /// bar shows unless it starts or stops filling its monitor, which
    /// [`touch`](Self::touch) is told of.
    pub(super) fn note(&mut self, input: &Input) {
        let moving = matches!(
            input,
            Input::Event(WindowEvent::LocationChanged { .. } | WindowEvent::MoveStarted { .. })
        );
        self.dirty |= !moving;
    }

    /// Notes that something a bar shows changed.
    pub(super) fn touch(&mut self) {
        self.dirty = true;
    }

    /// Notes that `window` changed its title, whether or not the tiler
    /// takes the event in.
    pub(super) fn title_changed(&mut self, window: WindowId) {
        if self.window.as_ref().is_some_and(|w| w.id == window) {
            self.title_stale = true;
            self.dirty = true;
        }
    }
}

impl<P: Platform> TilingManager<P> {
    /// The bar API's state with `daemon` and `theme`, when it differs from
    /// what this returned last, or when this is the first call; `None`
    /// when nothing changed. `daemon`'s workspace settings are replaced
    /// with the tiler's.
    ///
    /// The desktop is asked only when a step since the last call may have
    /// changed what the bars show, as the tiler sees it: a window that a
    /// hide or a minimize the tiler posted is on its way to does not count
    /// as maximized. The focused window's title is read only when the
    /// focus moves or the title changes.
    pub fn bar_state(&mut self, daemon: &api::Daemon, theme: &api::Theme) -> Option<&api::State> {
        let mut changed = !self.api.built;
        if self.api.dirty || !self.api.built {
            changed |= self.rebuild_api();
        }
        let daemon = self.tiler.api_daemon(daemon);
        changed |= replace(&mut self.api.state.daemon, daemon);
        changed |= replace(&mut self.api.state.theme, theme.clone());
        self.api.built = true;
        changed.then_some(&self.api.state)
    }

    /// The bar API's state as [`bar_state`](Self::bar_state) last
    /// returned it.
    pub fn last_bar_state(&self) -> &api::State {
        &self.api.state
    }

    /// Builds the monitors and the focus again, and tells whether they
    /// changed.
    fn rebuild_api(&mut self) -> bool {
        let focused = self.tiler.focused_window();
        let stale = self.api.title_stale || (self.api.window.as_ref()).map(|w| w.id) != focused;
        let now = self.platform.now();
        let tiler = &self.tiler;
        let (maximized, window) = self.applier.read_view(&self.platform, now, |desktop| {
            let window = focused
                .filter(|_| stale)
                .map(|id| focused_window(desktop, id));
            (tiler.maximized_monitors(desktop), window)
        });
        if stale {
            self.api.window = window;
            self.api.title_stale = false;
        }
        self.api.dirty = false;
        let monitors = self.tiler.api_monitors(&maximized);
        let focus = self.tiler.api_focus(self.api.window.clone());
        let state = &mut self.api.state;
        replace(&mut state.monitors, monitors) | replace(&mut state.focus, focus)
    }
}

/// The focused window `id` as the bar API shows it, as `desktop` has it.
fn focused_window(desktop: &dyn Desktop, id: WindowId) -> api::FocusedWindow {
    let facts = desktop.facts(id).unwrap_or_default();
    let program = Path::new(&facts.path).file_name().unwrap_or_default();
    api::FocusedWindow {
        id,
        title: facts.title.chars().take(MAX_TITLE_CHARS).collect(),
        program: program.to_string_lossy().into_owned(),
    }
}

/// Puts `new` in `slot` and tells whether it differed.
fn replace<T: PartialEq>(slot: &mut T, new: T) -> bool {
    let changed = *slot != new;
    if changed {
        *slot = new;
    }
    changed
}
