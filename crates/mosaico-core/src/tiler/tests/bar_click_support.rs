//! Tilers whose pointer follows focus, and what their steps did with it,
//! for the tests of actions from the bar and of stepping through
//! workspaces.

use crate::config::WorkspaceMode;
use crate::{Action, Direction, WindowId};

use super::super::tests::Harness;
use super::super::{BarReservation, Effect, Input, Settings};

/// The pointer follows focus, so a step that moved it would say so.
pub(super) fn following(mode: WorkspaceMode) -> Settings {
    let mut settings = Settings {
        mouse_follows_focus: true,
        ..Settings::default()
    };
    settings.workspaces.mode = mode;
    settings
}

/// Settings with workspaces switched in `mode`, and wrapping round with
/// `wrap`.
pub(super) fn stepping(mode: WorkspaceMode, wrap: bool) -> Settings {
    let mut settings = Settings::default();
    settings.workspaces.mode = mode;
    settings.workspaces.wrap = wrap;
    settings
}

/// Window 1 on monitor 0 and window 2 on monitor 1, each on workspace 2,
/// with both monitors back on workspace 1 and focus on monitor 0.
pub(super) fn both_on_second(mode: WorkspaceMode) -> Harness {
    let mut h = Harness::with_settings(following(mode), 2, &[(1, 0), (2, 1), (3, 0), (4, 1)]);
    h.action(Action::SendToWorkspace(2));
    h.action(Action::GoToWorkspace(1));
    h.action(Action::Focus(Direction::Right));
    h.action(Action::SendToWorkspace(2));
    h.action(Action::GoToWorkspace(1));
    h.action(Action::Focus(Direction::Left));
    h
}

/// Whether any effect of the last step moves the pointer.
pub(super) fn moves_pointer(h: &Harness) -> bool {
    h.effects.iter().any(|e| {
        matches!(
            e,
            Effect::CenterCursor { .. }
                | Effect::Focus {
                    move_cursor: true,
                    ..
                }
        )
    })
}

/// The windows the last step asked to focus, in order.
pub(super) fn focused(h: &Harness) -> Vec<WindowId> {
    h.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Focus { window, .. } => Some(*window),
            _ => None,
        })
        .collect()
}

/// The workspace on screen on each monitor of `h`, numbered from 1.
pub(super) fn on_screen(h: &Harness) -> Vec<usize> {
    h.tiler
        .monitors()
        .iter()
        .map(|m| m.active_workspace() + 1)
        .collect()
}

/// Reserves a 30 pixel bar at the top of every monitor.
pub(super) fn reserve_bar(h: &mut Harness) {
    h.step(Input::Bar(BarReservation {
        height: 30,
        monitors: Vec::new(),
    }));
}
