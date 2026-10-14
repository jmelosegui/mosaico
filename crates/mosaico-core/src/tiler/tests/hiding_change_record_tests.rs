//! The record of hidden windows when the hiding strategy changes while
//! windows are hidden (#92), as the tiling manager keeps it.
//!
//! The next daemon shows each window in the record the way it was
//! hidden, so the record has to keep how each one was hidden, whatever
//! the configuration says by then.

use crate::WindowEvent;
use crate::config::{Config, HidingBehaviour, WorkspaceMode};

use super::fake_platform::{Call, FakePlatform};
use super::manager_support::{TempRecord, start};
use super::*;

/// A configuration that hides windows `how`, one monitor at a time.
fn hiding(how: HidingBehaviour) -> Config {
    let mut config = Config::default();
    config.layout.hiding = how;
    config.workspaces.mode = WorkspaceMode::PerMonitor;
    config
}

/// How the record has the next daemon show each window, by id.
fn recorded(platform: &FakePlatform, record: &TempRecord) -> Vec<(u64, HidingBehaviour)> {
    record
        .file()
        .load()
        .restore(|w| platform.identity(w))
        .into_iter()
        .filter_map(|e| match e {
            Effect::Show { window, how } => Some((window.raw(), how)),
            _ => None,
        })
        .collect()
}

#[test]
fn the_record_of_hidden_windows_keeps_how_each_one_was_hidden() {
    // Arrange -- windows 2 and 3 are hidden with "hide"; after the config
    // changes to "minimize", window 1 is minimized.
    let record = TempRecord::new("hiding-change-each");
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0), (3, 0)]);
    let mut manager = start(
        &platform,
        Some(record.file()),
        &hiding(HidingBehaviour::Hide),
    );
    manager.handle_action(&Action::SendToWorkspace(2));
    manager.reload_config(&hiding(HidingBehaviour::Minimize));

    // Act
    manager.handle_action(&Action::GoToWorkspace(3));

    // Assert
    assert_eq!(
        recorded(&platform, &record),
        [
            (1, HidingBehaviour::Minimize),
            (2, HidingBehaviour::Hide),
            (3, HidingBehaviour::Hide),
        ]
    );
}

#[test]
fn a_hidden_window_moved_to_another_hidden_workspace_is_recorded_as_hidden() {
    // #92
    // Arrange -- sending window 3 to workspace 2 on monitor 1 hides
    // window 1 with "hide". After the config changes to "minimize", the
    // primary monitor leaves workspace 1, minimizing window 2. Each focus
    // report comes once the change before it has landed, as the user's.
    let record = TempRecord::new("hiding-change-moved");
    let platform = FakePlatform::new(2, &[(1, 1), (2, 0), (3, 1)]);
    let mut manager = start(
        &platform,
        Some(record.file()),
        &hiding(HidingBehaviour::Hide),
    );
    manager.flush_pending();
    manager.handle_event(&WindowEvent::Focused { window: wid(3) });
    manager.handle_action(&Action::SendToWorkspace(2));
    manager.reload_config(&hiding(HidingBehaviour::Minimize));
    manager.flush_pending();
    platform.advance(FOCUS_ECHO_TTL);
    manager.handle_event(&WindowEvent::Focused { window: wid(2) });
    manager.handle_action(&Action::GoToWorkspace(3));
    platform.take_calls();

    // Act -- monitor 1 goes away, so windows 1 and 3 move to the
    // primary's workspaces 1 and 2, which are both hidden.
    manager.handle_display_change(displays(1), 0, &[]);

    // Assert -- window 1 is still recorded as hidden, not as minimized.
    let hides: Vec<Call> = platform
        .calls()
        .into_iter()
        .filter(|c| matches!(c, Call::Hide(..)))
        .collect();
    assert_eq!(
        hides,
        [
            Call::Hide(wid(1), HidingBehaviour::Hide),
            Call::Hide(wid(3), HidingBehaviour::Minimize),
        ]
    );
    assert_eq!(
        recorded(&platform, &record),
        [
            (1, HidingBehaviour::Hide),
            (2, HidingBehaviour::Minimize),
            (3, HidingBehaviour::Minimize),
        ]
    );
}
