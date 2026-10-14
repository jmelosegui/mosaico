//! A reload that turns global mode on and changes other settings in the
//! same edit (#128).
//!
//! The monitors are lined up once the new settings are in place: a
//! window is hidden the new way and shown again the way it was hidden
//! (#92), and the monitor that switches is laid out once, with the new
//! gap, between showing its new workspace and hiding the old one.

use crate::config::{Config, HidingBehaviour, WorkspaceMode};

use super::*;

/// In per-monitor mode, monitor 0 shows workspace 1, with window 2, and
/// has window 1 hidden `how` on workspace 2. Monitor 1 shows workspace
/// 2, with focus, and has window 3 hidden on workspace 1.
fn out_of_step(how: HidingBehaviour) -> Harness {
    let mut settings = Settings::default();
    settings.layout.hiding = how;
    settings.workspaces.mode = WorkspaceMode::PerMonitor;
    let mut h = Harness::with_settings(settings, 2, &[(1, 0), (2, 0), (3, 1)]);
    // Window 1 has focus at the start.
    h.action(Action::SendToWorkspace(2));
    h.action(Action::GoToWorkspace(1));
    h.event(WindowEvent::Focused { window: wid(3) });
    h.action(Action::GoToWorkspace(2));
    h
}

/// Reloads `config`, with global mode turned on.
fn turn_global_mode_on(h: &mut Harness, mut config: Config) {
    config.workspaces.mode = WorkspaceMode::Global;
    h.step(Input::Config(Box::new(config)));
}

/// Where `wanted` is in the last step's effects.
fn find(h: &Harness, wanted: &Effect) -> Option<usize> {
    h.effects.iter().position(|e| e == wanted)
}

/// Turns global mode on while changing `hiding` from `old` to `new`, and
/// checks that window 2 is hidden the new way and window 1 shown the way
/// it was hidden.
fn change_hiding(old: HidingBehaviour, new: HidingBehaviour) {
    // Arrange
    let mut h = out_of_step(old);
    let mut config = Config::default();
    config.layout.hiding = new;

    // Act
    turn_global_mode_on(&mut h, config);

    // Assert
    let hide = Effect::Hide {
        window: wid(2),
        how: new,
    };
    let show = Effect::Show {
        window: wid(1),
        how: old,
    };
    assert!(find(&h, &hide).is_some(), "{hide:?}");
    assert!(find(&h, &show).is_some(), "{show:?}");
}

#[test]
fn turning_global_mode_on_while_switching_from_cloak_to_hide() {
    change_hiding(HidingBehaviour::Cloak, HidingBehaviour::Hide);
}

#[test]
fn turning_global_mode_on_while_switching_from_hide_to_minimize() {
    change_hiding(HidingBehaviour::Hide, HidingBehaviour::Minimize);
}

#[test]
fn turning_global_mode_on_while_switching_from_minimize_to_cloak() {
    change_hiding(HidingBehaviour::Minimize, HidingBehaviour::Cloak);
}

#[test]
fn turning_global_mode_on_while_changing_the_gap_places_with_the_new_gap() {
    // Arrange
    let mut h = out_of_step(HidingBehaviour::Cloak);
    let mut config = Config::default();
    config.layout.gap = 20;

    // Act
    turn_global_mode_on(&mut h, config);

    // Assert -- window 1, alone on monitor 0's workspace 2, fills it
    // inside the new gap, and is never placed with the old one.
    let slot = Rect::new(20, 20, MON_W - 40, MON_H - 40);
    assert_eq!(h.rect_of(1), slot);
    let elsewhere = |e: &Effect| matches!(*e, Effect::Place { window, rect, .. } if window == wid(1) && rect != slot);
    assert!(!h.effects.iter().any(elsewhere));
}

#[test]
fn turning_global_mode_on_lays_out_a_monitor_that_switches_once() {
    // Arrange
    let how = HidingBehaviour::Cloak;
    let mut h = out_of_step(how);

    // Act
    turn_global_mode_on(&mut h, Config::default());

    // Assert -- shown, placed once, then the old workspace hidden, as in
    // a workspace switch.
    let places: Vec<usize> = h
        .effects
        .iter()
        .enumerate()
        .filter(|(_, e)| matches!(**e, Effect::Place { window, .. } if window == wid(1)))
        .map(|(i, _)| i)
        .collect();
    assert_eq!(places.len(), 1, "placed once");
    let show = find(
        &h,
        &Effect::Show {
            window: wid(1),
            how,
        },
    );
    let hide = find(
        &h,
        &Effect::Hide {
            window: wid(2),
            how,
        },
    );
    assert!(show < Some(places[0]), "shown before it is placed");
    assert!(
        Some(places[0]) < hide,
        "placed before the old one is hidden"
    );
}
