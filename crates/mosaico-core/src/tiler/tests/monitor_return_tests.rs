//! A hidden window a display change brings onto a workspace in view
//! (#119).
//!
//! The window is out of view because the tiler hid it, so the retile
//! that follows the change must keep it in the layout and put it in its
//! slot, not drop it as a window that vanished. A window that did vanish,
//! or changed state on its own, is still dropped.

use crate::config::{Config, HidingBehaviour, WorkspaceMode};

use super::*;

/// Where the only window on monitor 0 goes, with the default gap.
const PRIMARY_SLOT: Rect = Rect {
    x: 8,
    y: 8,
    width: MON_W - 16,
    height: MON_H - 16,
};

/// Where the only window on monitor 1 goes, with the default gap.
const SECOND_SLOT: Rect = Rect {
    x: MON_W + 8,
    y: 8,
    width: MON_W - 16,
    height: MON_H - 16,
};

fn settings(how: HidingBehaviour) -> Settings {
    let mut settings = Settings::default();
    settings.layout.hiding = how;
    settings.workspaces.mode = WorkspaceMode::PerMonitor;
    settings
}

/// Connects the first `count` monitors of [`displays`].
fn connect(h: &mut Harness, count: usize) {
    h.step(Input::Displays {
        monitors: displays(count),
        bar: BarReservation::default(),
    });
}

/// Reloads a configuration that hides windows `how`.
fn reload(h: &mut Harness, how: HidingBehaviour) {
    let mut config = Config::default();
    config.layout.hiding = how;
    h.step(Input::Config(Box::new(config)));
}

/// Window 1 sits on monitor 1. The primary monitor is on workspace 2
/// when monitor 1 goes away, so window 1 moves to the primary's
/// workspace 1, which is not in view, and is hidden `how`.
fn window_1_hidden_while_monitor_1_is_away(how: HidingBehaviour) -> Harness {
    let mut h = Harness::with_settings(settings(how), 2, &[(1, 1)]);
    h.action(Action::GoToWorkspace(2));
    connect(&mut h, 1);
    h
}

/// Window 1 sits on monitor 1's workspace 1, hidden `how` because
/// monitor 1 switched to workspace 2. The primary monitor stays on
/// workspace 1.
fn window_1_hidden_on_monitor_1(how: HidingBehaviour) -> Harness {
    let mut h = Harness::with_settings(settings(how), 2, &[(1, 1)]);
    h.event(WindowEvent::Focused { window: wid(1) });
    h.action(Action::GoToWorkspace(2));
    h
}

/// Where in the last step's effects window 1 was placed at `slot`, and
/// where it was shown.
fn place_and_show(h: &Harness, slot: Rect) -> (Option<usize>, Option<usize>) {
    let place = h.effects.iter().position(
        |e| matches!(*e, Effect::Place { window, rect, .. } if window == wid(1) && rect == slot),
    );
    let show = h
        .effects
        .iter()
        .position(|e| matches!(*e, Effect::Show { window, .. } if window == wid(1)));
    (place, show)
}

/// Whether the last step placed window 1 anywhere.
fn placed_window_1(h: &Harness) -> bool {
    h.effects
        .iter()
        .any(|e| matches!(*e, Effect::Place { window, .. } if window == wid(1)))
}

#[test]
fn a_hidden_window_a_returning_monitor_brings_back_is_tiled_in_its_slot() {
    // #119
    // Arrange
    let mut h = window_1_hidden_while_monitor_1_is_away(HidingBehaviour::Hide);

    // Act -- monitor 1 comes back, and window 1 with it, onto the
    // workspace monitor 1 shows.
    connect(&mut h, 2);

    // Assert -- placed while still hidden, so it appears in its slot.
    assert_eq!(h.active(1), vec![1]);
    let (place, show) = place_and_show(&h, SECOND_SLOT);
    assert!(place.is_some(), "window 1 was not placed: {:?}", h.effects);
    assert!(show.is_some(), "window 1 was not shown: {:?}", h.effects);
    assert!(place < show, "window 1 was shown before it was in its slot");
}

#[test]
fn a_minimized_window_a_returning_monitor_brings_back_is_tiled_in_its_slot() {
    // #119
    // Arrange
    let mut h = window_1_hidden_while_monitor_1_is_away(HidingBehaviour::Minimize);

    // Act
    connect(&mut h, 2);

    // Assert -- a minimized window is restored before it is placed, as on
    // a workspace switch, since its slot cannot be set while minimized.
    assert_eq!(h.active(1), vec![1]);
    let (place, show) = place_and_show(&h, SECOND_SLOT);
    assert!(place.is_some(), "window 1 was not placed: {:?}", h.effects);
    assert!(show.is_some(), "window 1 was not shown: {:?}", h.effects);
    assert!(show < place, "window 1 was placed while still minimized");
}

#[test]
fn a_hidden_window_a_leaving_monitor_sends_to_a_workspace_in_view_is_tiled_there() {
    // #119
    // Arrange
    let mut h = window_1_hidden_on_monitor_1(HidingBehaviour::Hide);

    // Act -- monitor 1 goes away, so window 1 moves to the primary's
    // workspace 1, which is in view.
    connect(&mut h, 1);

    // Assert
    assert_eq!(h.active(0), vec![1]);
    let (place, show) = place_and_show(&h, PRIMARY_SLOT);
    assert!(place.is_some(), "window 1 was not placed: {:?}", h.effects);
    assert!(show.is_some(), "window 1 was not shown: {:?}", h.effects);
    assert!(place < show, "window 1 was shown before it was in its slot");
}

#[test]
fn a_minimized_window_a_leaving_monitor_sends_to_a_workspace_in_view_is_tiled_there() {
    // #119
    // Arrange
    let mut h = window_1_hidden_on_monitor_1(HidingBehaviour::Minimize);

    // Act
    connect(&mut h, 1);

    // Assert
    assert_eq!(h.active(0), vec![1]);
    let (place, show) = place_and_show(&h, PRIMARY_SLOT);
    assert!(place.is_some(), "window 1 was not placed: {:?}", h.effects);
    assert!(show.is_some(), "window 1 was not shown: {:?}", h.effects);
    assert!(show < place, "window 1 was placed while still minimized");
}

#[test]
fn a_cloaked_window_a_returning_monitor_brings_back_is_tiled_as_before() {
    // Arrange
    let mut h = window_1_hidden_while_monitor_1_is_away(HidingBehaviour::Cloak);

    // Act
    connect(&mut h, 2);

    // Assert -- a cloaked window still counts as shown, so it is placed,
    // then uncloaked in its slot.
    assert_eq!(h.active(1), vec![1]);
    let (place, show) = place_and_show(&h, SECOND_SLOT);
    assert!(place.is_some(), "window 1 was not placed: {:?}", h.effects);
    assert!(show.is_some(), "window 1 was not shown: {:?}", h.effects);
    assert!(place < show, "window 1 was shown before it was in its slot");
}

#[test]
fn a_hidden_window_closed_while_its_monitor_was_away_is_dropped_as_before() {
    // Arrange -- window 1 closes without a report while hidden.
    let mut h = window_1_hidden_while_monitor_1_is_away(HidingBehaviour::Hide);
    h.desktop.windows.remove(&wid(1));

    // Act
    connect(&mut h, 2);

    // Assert
    assert_eq!(h.tiler.window_count(), 0);
    assert!(!placed_window_1(&h), "{:?}", h.effects);
}

#[test]
fn a_minimized_window_closed_while_its_monitor_was_away_is_dropped_as_before() {
    // Arrange
    let mut h = window_1_hidden_while_monitor_1_is_away(HidingBehaviour::Minimize);
    h.desktop.windows.remove(&wid(1));

    // Act
    connect(&mut h, 2);

    // Assert
    assert_eq!(h.tiler.window_count(), 0);
    assert!(!placed_window_1(&h), "{:?}", h.effects);
}

#[test]
fn a_cloaked_window_that_minimized_itself_while_away_is_dropped_but_uncloaked() {
    // Arrange -- window 1 minimizes itself while cloaked on a workspace
    // not in view, which the tiler ignores there, and then the config
    // changes to "hide".
    let mut h = window_1_hidden_while_monitor_1_is_away(HidingBehaviour::Cloak);
    h.window(1).minimized = true;
    h.event(WindowEvent::Minimized { window: wid(1) });
    reload(&mut h, HidingBehaviour::Hide);

    // Act
    connect(&mut h, 2);

    // Assert -- minimized, it no longer belongs in the layout, as before,
    // but it is still uncloaked, the way it was hidden, rather than left
    // out of view.
    assert_eq!(h.tiler.window_count(), 0);
    assert!(!placed_window_1(&h), "{:?}", h.effects);
    assert!(
        h.effects.contains(&Effect::Show {
            window: wid(1),
            how: HidingBehaviour::Cloak,
        }),
        "{:?}",
        h.effects
    );
}
