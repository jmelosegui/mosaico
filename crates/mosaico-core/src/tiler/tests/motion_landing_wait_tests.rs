//! The wait by time for a window to land. One that moved since its
//! placement was posted, or is there, lands once the wait is over; one that
//! has not moved, as its application has not got to the placement, keeps
//! its border where it is and lands on its next report. A wait the window
//! leaves, or that a placement made without one ends, draws nothing later.

use crate::id::wid;
use crate::tiler::Platform;
use crate::tiler::applier::landing::LANDING_WAIT;
use crate::tiler::tests::fake_platform::Call;
use crate::{Action, WindowEvent};

use super::super::posted_tests::slow_swap;
use super::super::{
    REFRESH, animated, bordered_at, frame_by_frame, step, swap, two, wait_for_landing,
};
use super::bordered;

#[test]
fn a_window_that_has_not_moved_once_the_wait_is_over_keeps_its_border_until_it_reports() {
    // Arrange -- window 1 stops responding once it got to the first frame
    // of a swap, which ends the swap with its placement queued behind the
    // frame.
    let (platform, mut manager) = two(&animated());
    let right = platform.window(2).rect;
    swap(&mut manager);
    platform.advance(REFRESH);
    manager.frame();
    platform.edit(1, |w| w.hung = true);
    frame_by_frame(&platform, &mut manager);
    let placed = platform.now();
    let due = manager.frame_due(Some(placed));

    // Act -- the wait by time ends; later its application responds, gets
    // to what is queued for it, and reports where that leaves the window:
    // 10 pixels wider than placed, as a console sized to whole text cells
    // may be.
    wait_for_landing(&platform, &mut manager);
    let waited = platform.take_calls();
    let after = manager.frame_due(Some(platform.now()));
    platform.edit(1, |w| w.hung = false);
    platform.catch_up(1);
    platform.edit(1, |w| w.rect.width += 10);
    manager.handle_event(&WindowEvent::LocationChanged { window: wid(1) });

    // Assert -- nothing is drawn ahead of it, the daemon no longer wakes
    // for it, and it lands on its report, wherever that leaves it.
    assert_eq!(due, Some(placed + LANDING_WAIT));
    let repainted = waited.contains(&Call::Repaint(wid(1)));
    assert!(!bordered(&waited, 1) && !repainted, "{waited:?}");
    assert_eq!(after, None);
    let calls = platform.take_calls();
    assert!(calls.contains(&Call::Repaint(wid(1))), "{calls:?}");
    assert!(bordered_at(&calls, 1, right), "{calls:?}");
}

#[test]
fn a_window_that_moved_without_reporting_lands_once_the_wait_is_over() {
    // Arrange -- window 1, slow, gets to its placement, and its report is
    // missed.
    let (platform, mut manager, [_, right]) = slow_swap();
    platform.catch_up(1);
    platform.take_calls();

    // Act
    wait_for_landing(&platform, &mut manager);

    // Assert
    let calls = platform.take_calls();
    assert!(calls.contains(&Call::Repaint(wid(1))), "{calls:?}");
    assert!(bordered_at(&calls, 1, right), "{calls:?}");
}

#[test]
fn a_window_its_placement_does_not_move_from_about_its_slot_lands_once_the_wait_is_over() {
    // Arrange -- at 160 ms, window 1's last frame leaves it a pixel short
    // of its slot, and it keeps that pixel, as a window that rounds its
    // size may, so its placement neither moves it nor reports.
    let mut config = animated();
    config.animations.duration = 160;
    let (platform, mut manager) = two(&config);
    let right = platform.window(2).rect;
    swap(&mut manager);
    while step(&platform, &mut manager) {}
    platform.edit(1, |w| w.queued.clear());
    platform.take_calls();

    // Act
    wait_for_landing(&platform, &mut manager);

    // Assert -- it is about there, so the border is not ahead of it.
    assert_eq!(platform.window(1).rect.x, right.x - 1);
    let calls = platform.take_calls();
    assert!(calls.contains(&Call::Repaint(wid(1))), "{calls:?}");
    assert!(bordered_at(&calls, 1, right), "{calls:?}");
}

#[test]
fn a_window_the_user_grabs_on_its_way_is_no_longer_waited_for() {
    // Arrange -- window 1, slow, has its placement on its way.
    let (platform, mut manager, _) = slow_swap();

    // Act -- the user grabs it, and drags it away.
    manager.handle_event(&WindowEvent::move_started(wid(1)));
    platform.edit(1, |w| w.rect.y += 100);
    manager.handle_event(&WindowEvent::LocationChanged { window: wid(1) });

    // Assert -- nothing is left to draw for it.
    assert_eq!(manager.frame_due(Some(platform.now())), None);
}

#[test]
fn a_window_placed_again_where_it_was_sent_and_found_there_is_no_longer_waited_for() {
    // Arrange -- window 1, slow, gets to its placement without reporting it,
    // two pixels narrower, as a window that rounds its size may.
    let (platform, mut manager, _) = slow_swap();
    platform.catch_up(1);
    platform.edit(1, |w| w.rect.width -= 2);

    // Act -- laid out again, it is placed where it was sent last, and found
    // there: that placement is not waited for, nor is the one before.
    manager.handle_action(&Action::Retile);
    platform.take_calls();
    manager.handle_event(&WindowEvent::LocationChanged { window: wid(1) });

    // Assert
    assert_eq!(manager.frame_due(Some(platform.now())), None);
    let calls = platform.take_calls();
    assert!(!calls.contains(&Call::Repaint(wid(1))), "{calls:?}");
}
