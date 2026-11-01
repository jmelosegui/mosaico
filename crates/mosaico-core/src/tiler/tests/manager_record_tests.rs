//! The record the tiling manager keeps of the windows it hid, so that
//! the next daemon can show them again.

use crate::WindowEvent;
use crate::config::{Config, HidingBehaviour};

use super::fake_platform::FakePlatform;
use super::manager_support::{TempRecord, start};
use super::*;

/// A record of `windows`, hidden by cloaking, as they are on `platform`
/// now.
fn cloaked(platform: &FakePlatform, windows: &[u64]) -> HiddenWindows {
    let mut record = HiddenWindows::default();
    for &window in windows {
        let hide = Effect::Hide {
            window: wid(window),
            how: HidingBehaviour::Cloak,
        };
        record.applied(&hide, |w| platform.identity(w));
    }
    record
}

#[test]
fn a_window_hidden_by_a_workspace_switch_is_recorded() {
    // Arrange
    let record = TempRecord::new("recorded");
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let mut manager = start(&platform, Some(record.file()), &Config::with_gap(8));

    // Act
    manager.handle_action(&Action::GoToWorkspace(2));

    // Assert
    assert_eq!(record.file().load(), cloaked(&platform, &[1]));
}

#[test]
fn a_window_shown_again_leaves_the_record() {
    // Arrange
    let record = TempRecord::new("shown-again");
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let mut manager = start(&platform, Some(record.file()), &Config::with_gap(8));
    manager.handle_action(&Action::GoToWorkspace(2));

    // Act
    manager.handle_action(&Action::GoToWorkspace(1));

    // Assert
    assert!(record.file().load().is_empty());
    assert!(!record.path().exists());
}

#[test]
fn a_hidden_window_that_is_destroyed_leaves_the_record() {
    // Arrange
    let record = TempRecord::new("destroyed");
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    let mut manager = start(&platform, Some(record.file()), &Config::with_gap(8));
    manager.handle_action(&Action::GoToWorkspace(2));

    // Act
    platform.close_window(1);
    manager.handle_event(&WindowEvent::Destroyed { window: wid(1) });

    // Assert
    assert_eq!(record.file().load(), cloaked(&platform, &[2]));
}
