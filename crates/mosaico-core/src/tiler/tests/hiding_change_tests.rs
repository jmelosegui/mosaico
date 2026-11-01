//! Changing the hiding strategy while windows are hidden (#92).
//!
//! A window has to be shown again the way it was hidden, whatever the
//! configuration says by then: uncloaking a window that was hidden, or
//! showing one that was cloaked, leaves it out of view.

use crate::config::{Config, HidingBehaviour, WorkspaceMode};

use super::*;

fn hiding(how: HidingBehaviour) -> Settings {
    let mut settings = Settings::with_gap(8);
    settings.layout.hiding = how;
    settings.workspaces.mode = WorkspaceMode::PerMonitor;
    settings
}

/// Reloads a configuration that hides windows `how`.
fn reload(h: &mut Harness, how: HidingBehaviour) {
    let mut config = Config::with_gap(8);
    config.layout.hiding = how;
    h.step(Input::Config(Box::new(config)));
}

/// Connects the first `count` monitors of [`displays`].
fn connect(h: &mut Harness, count: usize) {
    h.step(Input::Displays {
        monitors: displays(count),
        bar: BarReservation::default(),
    });
}

/// The windows the last step showed, in order, and how.
fn shows(h: &Harness) -> Vec<(u64, HidingBehaviour)> {
    h.effects
        .iter()
        .filter_map(|e| match *e {
            Effect::Show { window, how } => Some((window.raw(), how)),
            _ => None,
        })
        .collect()
}

/// The windows the last step hid, in order, and how.
fn hides(h: &Harness) -> Vec<(u64, HidingBehaviour)> {
    h.effects
        .iter()
        .filter_map(|e| match *e {
            Effect::Hide { window, how } => Some((window.raw(), how)),
            _ => None,
        })
        .collect()
}

#[test]
fn windows_hidden_before_cloaking_was_chosen_are_shown_as_hidden() {
    // #92
    // Arrange -- windows 1 and 2 are hidden by a switch, then the config
    // changes from "hide" to "cloak".
    let mut h = Harness::with_settings(hiding(HidingBehaviour::Hide), 1, &[(1, 0), (2, 0)]);
    h.action(Action::GoToWorkspace(2));
    reload(&mut h, HidingBehaviour::Cloak);

    // Act
    h.action(Action::GoToWorkspace(1));

    // Assert -- uncloaking them would leave them hidden.
    assert_eq!(
        shows(&h),
        [(1, HidingBehaviour::Hide), (2, HidingBehaviour::Hide)]
    );
}

#[test]
fn windows_cloaked_before_hiding_was_chosen_are_uncloaked() {
    // #92
    // Arrange -- window 1 is cloaked by a switch, then the config changes
    // from "cloak" to "hide".
    let mut h = Harness::with_settings(hiding(HidingBehaviour::Cloak), 1, &[(1, 0)]);
    h.action(Action::GoToWorkspace(2));
    reload(&mut h, HidingBehaviour::Hide);

    // Act
    h.action(Action::GoToWorkspace(1));

    // Assert -- showing it as if it had been hidden would leave it cloaked.
    assert_eq!(shows(&h), [(1, HidingBehaviour::Cloak)]);
}

#[test]
fn sending_a_window_back_shows_the_ones_left_there_as_they_were_hidden() {
    // #92
    // Arrange -- window 1 is sent to workspace 2, which hides window 2,
    // then the config changes from "hide" to "minimize".
    let mut h = Harness::with_settings(hiding(HidingBehaviour::Hide), 1, &[(1, 0), (2, 0)]);
    h.action(Action::SendToWorkspace(2));
    reload(&mut h, HidingBehaviour::Minimize);

    // Act
    h.action(Action::SendToWorkspace(1));

    // Assert -- window 1 was never hidden, so it is shown the configured
    // way, as before.
    assert_eq!(
        shows(&h),
        [(2, HidingBehaviour::Hide), (1, HidingBehaviour::Minimize)]
    );
}

#[test]
fn a_window_hidden_while_its_monitor_was_away_is_shown_as_hidden_when_it_returns() {
    // #92
    // Arrange -- the primary monitor is on workspace 2 when monitor 1
    // goes away, so window 1 lands on a hidden workspace and is hidden.
    // Then the config changes from "hide" to "cloak".
    let mut h = Harness::with_settings(hiding(HidingBehaviour::Hide), 2, &[(1, 1)]);
    h.action(Action::GoToWorkspace(2));
    connect(&mut h, 1);
    assert!(h.effects.contains(&Effect::Hide {
        window: wid(1),
        how: HidingBehaviour::Hide,
    }));
    reload(&mut h, HidingBehaviour::Cloak);

    // Act -- the monitor comes back, and the window with it.
    connect(&mut h, 2);

    // Assert
    assert_eq!(shows(&h), [(1, HidingBehaviour::Hide)]);
}

/// Sending window 3 to workspace 2 on monitor 1 hides window 1 with
/// "hide". After the config changes to "minimize", the primary monitor
/// leaves workspace 1, minimizing window 2. Monitor 1 is still there.
fn window_1_hidden_on_monitor_1() -> Harness {
    let settings = hiding(HidingBehaviour::Hide);
    let mut h = Harness::with_settings(settings, 2, &[(1, 1), (2, 0), (3, 1)]);
    h.event(WindowEvent::Focused { window: wid(3) });
    h.action(Action::SendToWorkspace(2));
    reload(&mut h, HidingBehaviour::Minimize);
    h.event(WindowEvent::Focused { window: wid(2) });
    h.action(Action::GoToWorkspace(3));
    h
}

#[test]
fn a_hidden_window_moved_to_another_hidden_workspace_stays_hidden_the_same_way() {
    // #92
    // Arrange
    let mut h = window_1_hidden_on_monitor_1();

    // Act -- monitor 1 goes away, so windows 1 and 3 move to the
    // primary's workspaces 1 and 2, which are both hidden.
    connect(&mut h, 1);

    // Assert -- minimizing window 1 on top of how it was hidden would
    // leave a hide that showing it the minimized way does not undo.
    assert_eq!(
        hides(&h),
        [(1, HidingBehaviour::Hide), (3, HidingBehaviour::Minimize)]
    );
}

#[test]
fn a_hidden_window_moved_to_another_hidden_workspace_is_shown_as_hidden() {
    // #92
    // Arrange
    let mut h = window_1_hidden_on_monitor_1();
    connect(&mut h, 1);

    // Act
    h.action(Action::GoToWorkspace(1));

    // Assert
    assert_eq!(
        shows(&h),
        [(2, HidingBehaviour::Minimize), (1, HidingBehaviour::Hide)]
    );
}

#[test]
fn a_window_shown_again_is_hidden_the_new_way_next_time() {
    // Arrange -- window 1 is hidden with "hide", the config changes to
    // "minimize", and switching back shows it.
    let mut h = Harness::with_settings(hiding(HidingBehaviour::Hide), 1, &[(1, 0)]);
    h.action(Action::GoToWorkspace(2));
    reload(&mut h, HidingBehaviour::Minimize);
    h.action(Action::GoToWorkspace(1));

    // Act
    h.action(Action::GoToWorkspace(2));

    // Assert -- once it is shown, how it was hidden before is over.
    assert_eq!(hides(&h), [(1, HidingBehaviour::Minimize)]);
}

#[test]
fn an_unchanged_strategy_shows_windows_as_before() {
    // Arrange -- the config is reloaded with "minimize" kept and the gap
    // changed. Not the default, so falling back to it would show.
    let mut h = Harness::with_settings(hiding(HidingBehaviour::Minimize), 1, &[(1, 0), (2, 0)]);
    h.action(Action::GoToWorkspace(2));
    let mut config = Config::with_gap(8);
    config.layout.hiding = HidingBehaviour::Minimize;
    config.layout.gap = 16;
    h.step(Input::Config(Box::new(config)));

    // Act
    h.action(Action::GoToWorkspace(1));

    // Assert
    assert_eq!(
        shows(&h),
        [
            (1, HidingBehaviour::Minimize),
            (2, HidingBehaviour::Minimize)
        ]
    );
}

#[test]
fn a_window_the_tiler_did_not_hide_is_shown_the_configured_way() {
    // Arrange
    let mut h = Harness::with_settings(hiding(HidingBehaviour::Hide), 1, &[(1, 0), (2, 0)]);
    reload(&mut h, HidingBehaviour::Minimize);

    // Act -- window 1 follows itself to workspace 2 without being hidden.
    h.action(Action::SendToWorkspace(2));

    // Assert
    assert_eq!(shows(&h), [(1, HidingBehaviour::Minimize)]);
}
