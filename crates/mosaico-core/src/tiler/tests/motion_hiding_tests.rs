//! Moving windows hidden for a workspace switch and shown again while a
//! frame is on its way: by cloaking, which never waits, and by minimizing,
//! which waits behind the frame, as the show after it waits behind the
//! minimize.

use std::time::Duration;

use crate::config::{Config, HidingBehaviour};
use crate::id::wid;
use crate::tiler::tests::fake_desktop::Queued;
use crate::tiler::tests::manager_support::tiled;
use crate::{Action, WindowEvent};

use super::{REFRESH, animated, settle, swap, two};

/// A config with transitions on, whose workspace switches hide windows by
/// minimizing them.
fn minimizing() -> Config {
    let mut config = animated();
    config.layout.hiding = HidingBehaviour::Minimize;
    config
}

#[test]
fn a_window_hidden_and_shown_while_a_frame_is_on_its_way_is_placed_once_it_lands() {
    // Arrange -- a swap one frame in, window 1 slow to handle its frames.
    let (platform, mut manager) = two(&animated());
    let (left, right) = (platform.window(1).rect, platform.window(2).rect);
    platform.edit(1, |w| w.frame_delay = 3);
    swap(&mut manager);
    platform.advance(REFRESH);
    manager.frame();

    // Act -- away to another workspace and back; then window 1's
    // application gets to what is queued for it.
    manager.handle_action(&Action::GoToWorkspace(2));
    manager.handle_action(&Action::GoToWorkspace(1));
    manager.flush_pending();
    settle(&platform, &mut manager);
    platform.catch_up(1);

    // Assert -- in the slots of the swap: its placement lands after the
    // frame on its way.
    assert_eq!(platform.window(1).rect, right);
    assert_eq!(platform.window(2).rect, left);
}

#[test]
fn a_window_minimized_on_its_way_and_shown_straight_back_stays_tiled_in_its_slot() {
    // Arrange -- a swap one frame in, window 1 slow to handle its frames.
    let (platform, mut manager) = two(&minimizing());
    let right = platform.window(2).rect;
    platform.edit(1, |w| w.frame_delay = 3);
    swap(&mut manager);
    platform.advance(REFRESH);
    manager.frame();

    // Act -- away and straight back: the minimize waits behind the frame,
    // and the show behind the minimize. Window 2 reports where it is, and
    // two seconds later the application gets to the minimize, which the
    // platform reports, the layout is laid out again, and then the
    // application gets to the rest.
    manager.handle_action(&Action::GoToWorkspace(2));
    manager.handle_action(&Action::GoToWorkspace(1));
    manager.flush_pending();
    manager.handle_event(&WindowEvent::LocationChanged { window: wid(2) });
    platform.advance(Duration::from_secs(2));
    platform.catch_up_through(1, Queued::Minimize);
    manager.handle_event(&WindowEvent::Minimized { window: wid(1) });
    manager.handle_action(&Action::Retile);
    let minimized = tiled(&manager, 0);
    platform.catch_up(1);
    manager.handle_event(&WindowEvent::Restored { window: wid(1) });
    settle(&platform, &mut manager);

    // Assert -- the late minimize and restore are the tiler's own: window 1
    // stays tiled throughout, and is shown in its slot.
    assert_eq!(minimized, [2, 1]);
    assert_eq!(tiled(&manager, 0), [2, 1]);
    let one = platform.window(1);
    assert!(one.visible && !one.minimized, "{one:?}");
    assert_eq!(one.rect, right);
}

#[test]
fn a_window_minimized_on_its_way_is_placed_behind_the_show_behind_the_minimize() {
    // Arrange -- window 1's application is busy: it gets to what is
    // posted to it only when the test says, and a frame is on its way.
    let (platform, mut manager) = two(&minimizing());
    let right = platform.window(2).rect;
    platform.edit(1, |w| w.frame_delay = 40);
    swap(&mut manager);
    platform.advance(REFRESH);
    manager.frame();

    // Act -- away: the minimize waits behind the frame, and both land.
    // Back: the show waits behind the minimize, and the placement behind
    // the show. Then the application gets to both.
    manager.handle_action(&Action::GoToWorkspace(2));
    platform.catch_up(1);
    manager.handle_event(&WindowEvent::Minimized { window: wid(1) });
    manager.handle_action(&Action::GoToWorkspace(1));
    manager.flush_pending();
    settle(&platform, &mut manager);
    let waiting = platform.window(1);
    platform.catch_up(1);

    // Assert -- still minimized until then, and shown in its slot after.
    let queued: Vec<Queued> = waiting.queued.iter().map(|&(q, _)| q).collect();
    assert!(waiting.minimized, "{waiting:?}");
    assert_eq!(queued, [Queued::Show, Queued::Place(right)]);
    let one = platform.window(1);
    assert!(one.visible && !one.minimized, "{one:?}");
    assert_eq!(one.rect, right);
}

#[test]
fn a_window_restored_from_the_taskbar_then_minimized_and_restored_is_tiled_again() {
    // Arrange -- transitions off, workspace switches hide by minimizing,
    // and both windows are hidden on workspace 1 while the user is on
    // workspace 2. The user picks window 2 on the taskbar, which restores
    // it, and the tiler switches to it.
    let mut config = Config::default();
    config.layout.hiding = HidingBehaviour::Minimize;
    let (platform, mut manager) = two(&config);
    manager.handle_action(&Action::GoToWorkspace(2));
    for id in [1, 2] {
        manager.handle_event(&WindowEvent::Minimized { window: wid(id) });
    }
    platform.advance(Duration::from_secs(1));
    platform.edit(2, |w| w.minimized = false);
    manager.handle_event(&WindowEvent::Restored { window: wid(2) });
    manager.handle_event(&WindowEvent::Focused { window: wid(2) });

    // Act -- a second later the user minimizes it, and restores it a
    // second after that.
    platform.advance(Duration::from_secs(1));
    platform.edit(2, |w| w.minimized = true);
    manager.handle_event(&WindowEvent::Minimized { window: wid(2) });
    let minimized = tiled(&manager, 0);
    platform.advance(Duration::from_secs(1));
    platform.edit(2, |w| w.minimized = false);
    manager.handle_event(&WindowEvent::Restored { window: wid(2) });

    // Assert -- out of the layout while minimized, and back once restored,
    // as before transitions.
    assert_eq!(minimized, [1]);
    assert_eq!(tiled(&manager, 0), [1, 2]);
}
