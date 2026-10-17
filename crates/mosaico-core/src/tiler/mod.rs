//! The tiling state machine.
//!
//! [`Tiler`] owns every tiling decision: which windows are managed,
//! where they go, which workspace is on screen and what has focus. It
//! never touches the OS. Each [`Tiler::step`] reads the desktop through
//! [`Desktop`], updates the tiler's state and returns the [`Effect`]s
//! that bring the desktop in line with it. [`TilingManager`] runs a
//! tiler on a [`Platform`]: it applies the effects, and records them in
//! an [`Expectations`] so that the events they cause are recognized as
//! ours.

mod actions;
mod adopt;
mod applier;
mod bar_states;
mod border_style;
mod borders;
mod coalesce;
mod corners;
mod created;
mod decoration_effects;
mod desktop;
mod display;
mod display_monocle;
mod display_rebuild;
mod drag;
mod early_move;
mod echo;
mod echo_focus;
mod effect;
mod empty_focus;
mod empty_focus_park;
mod enumerator;
mod events;
mod events_focus;
mod focus;
mod hidden;
mod hidden_file;
mod hidden_how;
mod input;
mod last_drawn;
mod layout;
mod layout_actions;
mod lifecycle;
mod lookup;
mod manager;
mod navigation;
mod navigation_helpers;
mod placement;
mod plan;
mod platform;
mod queries;
mod reconnect;
mod release;
mod removal;
mod snapshot;
mod startup;
mod state;
mod step;
mod window_effects;
mod work_area;
mod workspace;

use std::collections::HashSet;

use crate::config::{
    AnimationConfig, BorderConfig, HidingBehaviour, LayoutConfig, WindowRule, WorkspaceMode,
};
use crate::{HidingSupport, LayoutKind, WindowId};

pub use border_style::{BorderStyle, PreviewStyle};
pub use coalesce::Coalescer;
pub use decoration_effects::apply_decoration_effect;
pub use desktop::{Desktop, Display, WindowFacts};
pub use echo::{Expectations, HIDE_FOCUS_ECHO_TTL};
pub use echo_focus::FOCUS_ECHO_TTL;
pub use effect::Effect;
pub use enumerator::{Enumerator, Insets, WindowDetails};
pub use hidden::{HiddenWindows, WindowIdentity};
pub use hidden_file::HiddenFile;
pub use input::{BarReservation, Input};
pub use last_drawn::LastDrawn;
pub use manager::TilingManager;
pub use placement::{Timing, Transition};
pub use platform::Platform;
pub use release::{MAX_HOLD, QUIET, release_at};
pub use startup::Settings;
pub use state::MonitorState;
pub use window_effects::apply_window_effect;

use plan::Plan;
use state::SpatialTarget;

/// Tiled windows across all connected monitors.
pub struct Tiler {
    monitors: Vec<MonitorState>,
    /// Layout of each workspace on a monitor seen for the first time,
    /// from `[workspaces.layouts]` falling back to `[layout] default`.
    workspace_layouts: Vec<LayoutKind>,
    /// Monitors that disconnected and the windows displaced from them.
    reconnect: reconnect::ReconnectMemory,
    /// Layout settings, with `hiding` the way windows are really hidden:
    /// the configured one, or what the platform has instead.
    layout: LayoutConfig,
    /// The hiding strategy the configuration asks for.
    configured_hiding: HidingBehaviour,
    /// The ways the platform can hide windows, to choose from again when
    /// the configured strategy changes.
    hiding_support: HidingSupport,
    /// How each window the tiler hid, for a workspace switch or a display
    /// change, was hidden, so it is shown again the same way.
    hidden_how: hidden_how::HiddenHow,
    /// Whether workspace switches affect one monitor or all monitors.
    workspace_mode: WorkspaceMode,
    workspace_wrap: bool,
    rules: Vec<WindowRule>,
    border_config: BorderConfig,
    /// Whether placements may animate, and how, see
    /// [`Transition`](crate::tiler::Transition).
    animations: AnimationConfig,
    /// Whether no step has run since the tiler started. The bar reserving
    /// its space right after the start lays out as the start does.
    fresh: bool,
    /// Windows that currently have a border, so the ones that leave the
    /// screen can have theirs destroyed.
    bordered: HashSet<WindowId>,
    /// Windows given the configured corners, so that only those get the
    /// system's corners back. A minimized window leaves the layout but
    /// stays here; one dropped from it for no longer being shown does
    /// not.
    decorated: HashSet<WindowId>,
    mouse: drag::Mouse,
    focused_monitor: usize,
    focused_window: Option<WindowId>,
    /// Whether the focused window was maximized when last looked at, so
    /// that `LocationChanged`, which fires on every animation frame,
    /// reacts only when the window enters or leaves maximized state.
    focused_maximized: bool,
    filling: bar_states::Filling,
    /// Pins `focused_monitor` while its active workspace is empty. See
    /// [`empty_focus`].
    empty_focus_claim: Option<empty_focus::EmptyFocusClaim>,
    /// Windows the rules already rejected, so the deferred events that
    /// keep arriving for them are not judged again. Cleared when the
    /// rules change; entries are removed when the window is destroyed.
    adopt_rejected: HashSet<WindowId>,
}

#[cfg(test)]
mod tests;
