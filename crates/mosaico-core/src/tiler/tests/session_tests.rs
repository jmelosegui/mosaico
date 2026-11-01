//! A daemon killed with windows hidden, and the next one starting (#67).

use crate::config::{Config, HidingBehaviour, WorkspaceMode};

use super::fake_platform::{Call, FakePlatform};
use super::manager_support::{TempRecord, start, tiled};
use super::*;

fn hiding(how: HidingBehaviour) -> Config {
    let mut config = Config::with_gap(8);
    config.layout.hiding = how;
    config.workspaces.mode = WorkspaceMode::PerMonitor;
    config
}

/// Kills the daemon running `manager` without a clean shutdown and
/// starts the next one on the same desktop, with the same record: it
/// shows what the record says the old one hid, then tiles whatever
/// exists.
fn kill_and_restart(
    manager: TilingManager<FakePlatform>,
    platform: &FakePlatform,
    record: &TempRecord,
    config: &Config,
) -> TilingManager<FakePlatform> {
    drop(manager);
    start(platform, Some(record.file()), config)
}

#[test]
fn windows_hidden_by_a_killed_daemon_are_shown_and_tiled_by_the_next_one() {
    // #67
    // Arrange -- workspace 2 is on screen, so the windows of workspace 1
    // are cloaked when the daemon is killed.
    let record = TempRecord::new("killed-cloaked");
    let config = hiding(HidingBehaviour::Cloak);
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    let mut manager = start(&platform, Some(record.file()), &config);
    manager.handle_action(&Action::SendToWorkspace(2));
    manager.handle_action(&Action::GoToWorkspace(3));
    assert!(platform.window(1).cloaked && platform.window(2).cloaked);

    // Act
    let next = kill_and_restart(manager, &platform, &record, &config);

    // Assert -- nothing is left invisible, and the new daemon manages both.
    assert!(
        !platform.window(1).cloaked,
        "window 1 is still cloaked after the restart"
    );
    assert!(
        !platform.window(2).cloaked,
        "window 2 is still cloaked after the restart"
    );
    let mut managed = tiled(&next, 0);
    managed.sort();
    assert_eq!(managed, vec![1, 2]);
}

#[test]
fn the_next_daemon_clears_the_record_once_it_has_shown_the_windows() {
    // Arrange
    let record = TempRecord::new("restart-clears");
    let config = hiding(HidingBehaviour::Cloak);
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let mut manager = start(&platform, Some(record.file()), &config);
    manager.handle_action(&Action::GoToWorkspace(2));
    assert!(record.path().exists());

    // Act
    kill_and_restart(manager, &platform, &record, &config);

    // Assert -- were it killed again, there would be nothing to replay.
    assert!(!record.path().exists());
}

#[test]
fn windows_hidden_with_show_window_are_shown_by_the_next_daemon() {
    // #67
    // Arrange
    let record = TempRecord::new("killed-hidden");
    let config = hiding(HidingBehaviour::Hide);
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let mut manager = start(&platform, Some(record.file()), &config);
    manager.handle_action(&Action::GoToWorkspace(2));
    assert!(!platform.window(1).visible);

    // Act
    let next = kill_and_restart(manager, &platform, &record, &config);

    // Assert
    assert!(
        platform.window(1).visible,
        "window 1 is still hidden after the restart"
    );
    assert_eq!(tiled(&next, 0), vec![1]);
}

#[test]
fn a_clean_shutdown_leaves_nothing_to_restore() {
    // Arrange
    let record = TempRecord::new("clean-shutdown");
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let mut manager = start(&platform, Some(record.file()), &Config::with_gap(8));
    manager.handle_action(&Action::GoToWorkspace(2));

    // Act
    manager.restore_all_windows();

    // Assert
    assert!(record.file().load().is_empty());
}

#[test]
fn a_reused_handle_is_left_alone() {
    // Arrange -- the hidden window closed and another process got its
    // handle, cloaked by something else.
    let record = TempRecord::new("reused-handle");
    let config = Config::with_gap(8);
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let mut manager = start(&platform, Some(record.file()), &config);
    manager.handle_action(&Action::GoToWorkspace(2));
    platform.edit(1, |w| w.process = 4242);
    platform.take_calls();

    // Act
    kill_and_restart(manager, &platform, &record, &config);

    // Assert
    let calls = platform.calls();
    assert!(
        !calls
            .iter()
            .any(|c| matches!(c, Call::Show(w, _) | Call::Reveal(w) if *w == wid(1))),
        "a window that is not the one hidden must not be shown: {calls:?}"
    );
    assert!(platform.window(1).cloaked);
}
