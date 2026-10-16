//! Shows posted behind what is on its way to a window: the tiler's view
//! of the desktop has the window shown while the show is on its way, and
//! only while it is, and only when the show changes something.

use std::time::Duration;

use crate::config::HidingBehaviour;
use crate::id::wid;
use crate::tiler::tests::manager_support::tiled;
use crate::{Action, WindowEvent};

use super::{FakePlatform, Manager, REFRESH, animated, settle, swap, two};

/// Two windows side by side, hidden `how` for workspace switches, where
/// window 1's application is busy with a frame of a swap on its way. Away:
/// the hide waits behind the frame, then both land, and the hide is
/// reported. Back: the show waits behind the hide, and the layout is laid
/// out again before it lands.
fn shown_on_its_way(how: HidingBehaviour) -> (FakePlatform, Manager) {
    let mut config = animated();
    config.layout.hiding = how;
    let (platform, mut manager) = two(&config);
    platform.edit(1, |w| w.frame_delay = 40);
    swap(&mut manager);
    platform.advance(REFRESH);
    manager.frame();
    manager.handle_action(&Action::GoToWorkspace(2));
    platform.catch_up(1);
    let report = match how {
        HidingBehaviour::Minimize => WindowEvent::Minimized { window: wid(1) },
        _ => WindowEvent::Hidden { window: wid(1) },
    };
    manager.handle_event(&report);
    manager.handle_action(&Action::GoToWorkspace(1));
    manager.flush_pending();
    manager.handle_action(&Action::Retile);
    (platform, manager)
}

#[test]
fn a_window_whose_show_is_on_its_way_stays_in_the_layout_meanwhile() {
    // Arrange
    let (platform, mut manager) = shown_on_its_way(HidingBehaviour::Minimize);
    let meanwhile = (platform.window(1).minimized, tiled(&manager, 0));

    // Act -- its application gets to the show, which is reported, and the
    // user minimizes it.
    platform.catch_up(1);
    manager.handle_event(&WindowEvent::Restored { window: wid(1) });
    platform.edit(1, |w| w.minimized = true);
    manager.handle_event(&WindowEvent::Minimized { window: wid(1) });

    // Assert -- tiled while the tiler had shown it, and no longer once the
    // user minimized it after the show came.
    assert_eq!(meanwhile, (true, vec![2, 1]));
    assert_eq!(tiled(&manager, 0), [2]);
}

#[test]
fn a_window_hidden_by_hiding_whose_show_is_on_its_way_stays_in_the_layout_meanwhile() {
    // Arrange
    let (platform, mut manager) = shown_on_its_way(HidingBehaviour::Hide);
    let meanwhile = (platform.window(1).visible, tiled(&manager, 0));

    // Act -- its application gets to the show, which is reported, then
    // hides the window itself.
    platform.catch_up(1);
    manager.handle_event(&WindowEvent::Created { window: wid(1) });
    platform.edit(1, |w| w.visible = false);
    manager.handle_event(&WindowEvent::Hidden { window: wid(1) });

    // Assert
    assert_eq!(meanwhile, (false, vec![2, 1]));
    assert_eq!(tiled(&manager, 0), [2]);
}

#[test]
fn a_window_given_the_id_of_one_whose_show_was_on_its_way_is_seen_as_it_is() {
    // Arrange -- window 1 is closed without a report while its show is on
    // its way, and a minimized window of another thread gets its id.
    let (platform, mut manager) = shown_on_its_way(HidingBehaviour::Minimize);
    platform.edit(1, |w| {
        w.thread += 1000;
        w.queued.clear();
    });

    // Act
    manager.handle_action(&Action::Retile);

    // Assert -- the new window is minimized, so it leaves the layout.
    assert_eq!(tiled(&manager, 0), [2]);
}

#[test]
fn a_show_of_a_window_on_screen_leaves_the_users_minimize_to_take_it_out() {
    // Arrange -- workspace switches hide windows by minimizing. Half a
    // second after a swap, window 1 is sent to workspace 2, which shows it,
    // on screen already, behind what was posted to it.
    let mut config = animated();
    config.layout.hiding = HidingBehaviour::Minimize;
    let (platform, mut manager) = two(&config);
    swap(&mut manager);
    settle(&platform, &mut manager);
    platform.advance(Duration::from_millis(500));
    manager.handle_action(&Action::SendToWorkspace(2));
    let sent = tiled(&manager, 0);

    // Act -- an hour later the user minimizes it.
    platform.advance(Duration::from_secs(3600));
    platform.edit(1, |w| w.minimized = true);
    manager.handle_event(&WindowEvent::Minimized { window: wid(1) });

    // Assert -- it leaves the layout, as it does without transitions.
    assert_eq!(sent, [1]);
    assert!(tiled(&manager, 0).is_empty());
}

#[test]
fn a_show_that_landed_is_forgotten_before_it_is_reported() {
    // Arrange -- window 1's application gets to the show, and the layout
    // is laid out again before the restore is reported.
    let (platform, mut manager) = shown_on_its_way(HidingBehaviour::Minimize);
    platform.catch_up(1);
    manager.handle_action(&Action::Retile);

    // Act -- the user minimizes it.
    platform.edit(1, |w| w.minimized = true);
    manager.handle_event(&WindowEvent::Minimized { window: wid(1) });

    // Assert
    assert_eq!(tiled(&manager, 0), [2]);
}
