//! Changing the workspace mode through a config reload (#128).
//!
//! Global mode shows the same workspace number on every monitor, so a
//! reload that turns it on switches every monitor to the focused one's
//! workspace, the way a workspace switch does. Any other reload leaves
//! each monitor on the workspace it shows.

use crate::config::{Config, HidingBehaviour, WorkspaceMode};

use super::*;

/// Where the only window on monitor 0 goes, with the default gap.
const PRIMARY_SLOT: Rect = Rect {
    x: 8,
    y: 8,
    width: MON_W - 16,
    height: MON_H - 16,
};

fn settings(how: HidingBehaviour, mode: WorkspaceMode) -> Settings {
    let mut settings = Settings::with_gap(8);
    settings.layout.hiding = how;
    settings.workspaces.mode = mode;
    settings
}

/// Reloads a configuration with workspace mode `mode`, that hides
/// windows `how`.
fn reload(h: &mut Harness, how: HidingBehaviour, mode: WorkspaceMode) {
    let mut config = Config::with_gap(8);
    config.layout.hiding = how;
    config.workspaces.mode = mode;
    h.step(Input::Config(Box::new(config)));
}

/// In per-monitor mode, monitor 0 shows workspace 1, with window 2, and
/// has window 1 hidden `how` on workspace 2. Monitor 1 shows workspace
/// 2, with focus, and has window 3 hidden on workspace 1.
fn monitors_on_different_workspaces(how: HidingBehaviour) -> Harness {
    let settings = settings(how, WorkspaceMode::PerMonitor);
    let mut h = Harness::with_settings(settings, 2, &[(1, 0), (2, 0), (3, 1)]);
    // Window 1 has focus at the start.
    h.action(Action::SendToWorkspace(2));
    h.action(Action::GoToWorkspace(1));
    h.event(WindowEvent::Focused { window: wid(3) });
    h.action(Action::GoToWorkspace(2));
    h
}

/// Both monitors show workspace 2 in global mode, with window 1 and 2
/// hidden `how` on their workspace 1, and focus is on monitor 0.
fn global_on_workspace_2(how: HidingBehaviour) -> Harness {
    let settings = settings(how, WorkspaceMode::Global);
    let mut h = Harness::with_settings(settings, 2, &[(1, 0), (2, 1)]);
    h.action(Action::GoToWorkspace(2));
    h
}

/// The index of the workspace each monitor shows: 1 is workspace 2.
fn in_view(h: &Harness) -> Vec<usize> {
    h.tiler
        .monitors()
        .iter()
        .map(MonitorState::active_workspace)
        .collect()
}

/// Whether window `id` is out of view the way `how` hides it.
fn hidden(h: &Harness, id: u64, how: HidingBehaviour) -> bool {
    let w = &h.desktop.windows[&wid(id)];
    match how {
        HidingBehaviour::Cloak => w.cloaked,
        HidingBehaviour::Hide => !w.visible,
        HidingBehaviour::Minimize => w.minimized,
    }
}

/// Whether window `id` is in view.
fn in_sight(h: &Harness, id: u64) -> bool {
    let w = &h.desktop.windows[&wid(id)];
    w.visible && !w.cloaked && !w.minimized
}

/// The windows the last step hid, and the ones it showed, in order.
fn hid_and_shown(h: &Harness) -> (Vec<u64>, Vec<u64>) {
    let (mut hid, mut shown) = (Vec::new(), Vec::new());
    for effect in &h.effects {
        match *effect {
            Effect::Hide { window, .. } => hid.push(window.raw()),
            Effect::Show { window, .. } => shown.push(window.raw()),
            _ => {}
        }
    }
    (hid, shown)
}

/// Turns global mode on while the monitors show different workspaces,
/// and checks that monitor 0 switches to workspace 2, the focused
/// monitor's, the way a workspace switch does.
fn turn_global_mode_on(how: HidingBehaviour) {
    // Arrange
    let mut h = monitors_on_different_workspaces(how);

    // Act
    reload(&mut h, how, WorkspaceMode::Global);

    // Assert -- #128: the monitors used to stay on workspaces 1 and 2.
    assert_eq!(in_view(&h), vec![1, 1]);
    assert_eq!(hid_and_shown(&h), (vec![2], vec![1]));
    assert_eq!(h.active(0), vec![1]);
    assert_eq!(h.rect_of(1), PRIMARY_SLOT);
    assert!(in_sight(&h, 1) && hidden(&h, 2, how) && hidden(&h, 3, how));
    assert_eq!(h.tiler.focused_monitor(), 1);
}

#[test]
fn turning_global_mode_on_switches_every_monitor_to_the_focused_workspace() {
    turn_global_mode_on(HidingBehaviour::Cloak);
}

#[test]
fn turning_global_mode_on_shows_hidden_windows_of_the_workspace_switched_to() {
    turn_global_mode_on(HidingBehaviour::Hide);
}

#[test]
fn turning_global_mode_on_restores_minimized_windows_of_the_workspace_switched_to() {
    turn_global_mode_on(HidingBehaviour::Minimize);
}

#[test]
fn a_per_monitor_reload_leaves_each_monitor_on_its_workspace() {
    // Arrange
    let how = HidingBehaviour::Cloak;
    let mut h = monitors_on_different_workspaces(how);

    // Act
    reload(&mut h, how, WorkspaceMode::PerMonitor);

    // Assert
    assert_eq!(in_view(&h), vec![0, 1]);
    assert_eq!(hid_and_shown(&h), (vec![], vec![]));
    assert!(in_sight(&h, 2) && hidden(&h, 1, how));
}

#[test]
fn a_reload_that_keeps_global_mode_switches_nothing() {
    // Arrange
    let how = HidingBehaviour::Cloak;
    let mut h = global_on_workspace_2(how);

    // Act
    reload(&mut h, how, WorkspaceMode::Global);

    // Assert
    assert_eq!(in_view(&h), vec![1, 1]);
    assert_eq!(hid_and_shown(&h), (vec![], vec![]));
}

#[test]
fn turning_global_mode_off_leaves_each_monitor_on_its_workspace() {
    // Arrange
    let how = HidingBehaviour::Cloak;
    let mut h = global_on_workspace_2(how);

    // Act
    reload(&mut h, how, WorkspaceMode::PerMonitor);

    // Assert -- nothing switches, and from now on a switch only moves
    // the focused monitor.
    assert_eq!(in_view(&h), vec![1, 1]);
    assert_eq!(hid_and_shown(&h), (vec![], vec![]));
    h.action(Action::GoToWorkspace(1));
    assert_eq!(in_view(&h), vec![0, 1]);
    assert!(in_sight(&h, 1) && hidden(&h, 2, how));
}
