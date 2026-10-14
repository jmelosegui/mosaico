//! The tiling manager on a fake platform: starting, hiding, reading the
//! work areas again and giving the windows back.

use crate::config::{Config, HidingBehaviour};

use super::fake_platform::{Call, FakePlatform};
use super::manager_support::{start, tiled};
use super::*;

#[test]
fn startup_tiles_the_windows_the_platform_lists() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);

    // Act
    let manager = start(&platform, None, &Config::default());

    // Assert -- placed side by side through the window control, with
    // their corners set through the decorations.
    assert_eq!(tiled(&manager, 0), vec![1, 2]);
    assert_ne!(platform.window(1).rect, platform.window(2).rect);
    let calls = platform.calls();
    let corners = Config::default().borders.corner_style;
    for window in [wid(1), wid(2)] {
        assert!(
            calls
                .iter()
                .any(|c| matches!(c, Call::Place(w, _) if *w == window)),
            "{window} was not placed: {calls:?}"
        );
        assert!(
            calls.contains(&Call::SetCorners(window, corners)),
            "{calls:?}"
        );
    }
}

#[test]
fn startup_fails_when_the_monitors_cannot_be_listed() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0)]);
    platform.fail_monitors();

    // Act
    let started = TilingManager::new(platform.clone(), None, &Config::default(), Vec::new());

    // Assert
    assert!(started.is_err());
    assert!(platform.calls().is_empty(), "{:?}", platform.calls());
}

#[test]
fn windows_are_hidden_in_a_way_the_platform_has() {
    // Arrange -- the config asks for cloaking, which this platform lacks.
    let platform = FakePlatform::new(1, &[(1, 0)]);
    platform.lack(HidingBehaviour::Cloak);
    let mut manager = start(&platform, None, &Config::default());

    // Act
    manager.handle_action(&Action::GoToWorkspace(2));

    // Assert
    let calls = platform.calls();
    assert!(
        calls.contains(&Call::Hide(wid(1), HidingBehaviour::Hide)),
        "{calls:?}"
    );
    assert!(!platform.window(1).visible);
}

#[test]
fn a_window_a_display_change_brings_onto_monocle_is_put_behind_it() {
    // Arrange -- window 1 is in monocle on the primary, window 2 is on
    // the second monitor.
    let platform = FakePlatform::new(2, &[(1, 0), (2, 1)]);
    let mut manager = start(&platform, None, &Config::default());
    manager.handle_action(&Action::ToggleMonocle);
    platform.take_calls();

    // Act -- the second monitor leaves.
    manager.handle_display_change(displays(1), 0, &[]);

    // Assert -- through the window control.
    let calls = platform.calls();
    assert!(
        calls.contains(&Call::PutBehind(wid(2), wid(1))),
        "{calls:?}"
    );
}

#[test]
fn the_work_areas_are_read_again_from_the_platform() {
    // Arrange -- the taskbar grew after the manager started.
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let mut manager = start(&platform, None, &Config::default());
    let smaller = Rect::new(0, 0, MON_W, MON_H - 48);
    platform.set_work_area(0, smaller);

    // Act
    let read = manager.reset_and_adjust_work_areas(0, &[]);

    // Assert
    assert!(read.is_ok());
    assert_eq!(manager.tiler().monitors()[0].work_area(), smaller);
    assert!(platform.window(1).rect.y + platform.window(1).rect.height <= smaller.height);
}

#[test]
fn the_work_areas_stay_when_the_monitors_cannot_be_read_again() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let mut manager = start(&platform, None, &Config::default());
    let before = manager.tiler().monitors()[0].work_area();
    platform.fail_monitors();

    // Act
    let read = manager.reset_and_adjust_work_areas(0, &[]);

    // Assert -- the windows are laid out on the areas last read.
    assert!(read.is_err());
    assert_eq!(manager.tiler().monitors()[0].work_area(), before);
}

#[test]
fn shutdown_shows_every_window_and_gives_back_their_corners() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    let mut manager = start(&platform, None, &Config::default());
    manager.handle_action(&Action::SendToWorkspace(2));
    platform.take_calls();

    // Act
    manager.restore_all_windows();

    // Assert
    let calls = platform.calls();
    for window in [wid(1), wid(2)] {
        assert!(calls.contains(&Call::Reveal(window)), "{calls:?}");
        assert!(calls.contains(&Call::ResetCorners(window)), "{calls:?}");
    }
    assert!(!platform.window(1).cloaked && !platform.window(2).cloaked);
}
