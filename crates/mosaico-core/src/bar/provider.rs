//! The contract every bar follows, whichever program draws it.

use std::time::Instant;

use crate::Action;
use crate::action::Origin;
use crate::api::State;
use crate::config::BarConfig;
use crate::tiler::{MonitorPlace, Reach};

use super::{BarId, PointerEvent};

/// A bar mosaico shows its state on, and keeps tiles clear of: the
/// built-in one, a bar another program draws, or none.
///
/// The daemon owns one provider at a time and is its only caller, on its
/// own thread. Every provider is told the same [`State`], so a bar can
/// show only what the bar API carries.
///
/// # Contract for implementations
///
/// - [`place`](Self::place) comes before the first [`show`](Self::show),
///   and again whenever the monitors or their work areas change. A
///   provider that draws makes or moves its bars there.
/// - [`reach`](Self::reach) is how far the bars reach into each monitor
///   as they are placed now. It must not change between two calls to
///   `place` unless the bar's own config changed, since the tiler only
///   asks for it then.
/// - [`show`](Self::show) is only called with a state that differs from
///   the one shown last, or after `place`. A provider keeps the last state
///   if it needs to draw again on its own.
/// - Nothing here may block: a provider talking to another program queues
///   what it sends and drops it when that program falls behind.
pub trait BarProvider {
    /// The name `[bar] provider` gives it, such as `"mosaico"`.
    fn name(&self) -> &str;

    /// Makes or moves the bars for the monitors `places`, left to right.
    fn place(&mut self, places: &[MonitorPlace]);

    /// How far the bars reach into each of `places` as they are placed.
    fn reach(&self, places: &[MonitorPlace]) -> Reach;

    /// Shows `state`.
    fn show(&mut self, state: &State);

    /// Whether anything would come of [`show`](Self::show) now. The
    /// daemon builds no state while it is `false`, and shows the state
    /// it missed once it turns `true`.
    fn wants_state(&self) -> bool {
        true
    }

    /// Draws again what changes on its own, such as a clock. Called about
    /// once a second.
    fn refresh(&mut self) {}

    /// Takes what the pointer did on the bar `bar` at `now`, and returns
    /// the action it asks for, if any. Only a provider that draws through
    /// mosaico's renderer gets these.
    fn pointer(
        &mut self,
        _bar: BarId,
        _event: PointerEvent,
        _now: Instant,
    ) -> Option<(Action, Origin)> {
        None
    }

    /// When the provider next has actions to hand over on its own, such
    /// as a workspace switch the wheel asked for once it went quiet.
    fn due(&self) -> Option<Instant> {
        None
    }

    /// The actions due at `now`, see [`due`](Self::due).
    fn take_due(&mut self, _now: Instant) -> Vec<(Action, Origin)> {
        Vec::new()
    }

    /// Takes in a new `bar.toml`. A provider that does not draw ignores
    /// it. Call [`reach`](Self::reach) again after this.
    fn reload(&mut self, _config: BarConfig) {}

    /// Takes the bars off screen as the daemon stops or switches to
    /// another provider.
    fn stop(&mut self);
}

/// No bar: nothing is shown and nothing kept clear.
#[derive(Debug, Default)]
pub struct NoBar;

impl BarProvider for NoBar {
    fn name(&self) -> &str {
        "none"
    }

    fn place(&mut self, _places: &[MonitorPlace]) {}

    fn reach(&self, _places: &[MonitorPlace]) -> Reach {
        Reach::none()
    }

    fn show(&mut self, _state: &State) {}

    fn wants_state(&self) -> bool {
        false
    }

    fn stop(&mut self) {}
}

/// A bar another program draws, which reads the state through the bar
/// API. Until it says how far it reaches, tiles keep to the work areas
/// the system gives.
#[derive(Debug, Default)]
pub struct ApiBar;

impl BarProvider for ApiBar {
    fn name(&self) -> &str {
        "external"
    }

    fn place(&mut self, _places: &[MonitorPlace]) {}

    fn reach(&self, _places: &[MonitorPlace]) -> Reach {
        Reach::none()
    }

    fn show(&mut self, _state: &State) {}

    fn wants_state(&self) -> bool {
        false
    }

    fn stop(&mut self) {}
}
