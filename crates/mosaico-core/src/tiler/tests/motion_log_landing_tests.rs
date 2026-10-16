//! What the log is told of a window a transition placed behind what was on
//! its way to it, as it lands: once it reports being there; once the wait
//! for it by time is over, if it moved since; once it reports being
//! elsewhere with nothing on its way; or on its next report, if it had not
//! moved by the end of the wait. A window whose placement ends no
//! transition is not logged as it lands.

use std::time::Duration;

use crate::WindowEvent;
use crate::frames::ASYNC_GRACE;
use crate::id::wid;
use crate::tiler::tests::fake_platform::FakePlatform;
use crate::tiler::tests::manager_support::start;

use super::super::posted_tests::slow_swap;
use super::super::{animated, settle, step, swap, wait_for_landing};
use super::starting;

/// The report that window 1 is somewhere.
fn report() -> WindowEvent {
    WindowEvent::LocationChanged { window: wid(1) }
}

#[test]
fn a_window_that_reports_being_in_place_is_logged_as_in_place() {
    // Arrange -- window 1 fell behind, its placement queued behind its
    // frames, and nothing but its landing is left to log.
    let (platform, mut manager, _) = slow_swap();
    manager.transition_lines();

    // Act -- its application gets to the placement, which it reports.
    platform.catch_up(1);
    manager.handle_event(&report());

    // Assert
    let lines = manager.transition_lines();
    let landed = "transition: 0x1 is in place, 0 ms after its placement was posted";
    assert_eq!(lines, [landed], "{lines:#?}");
}

#[test]
fn a_window_that_moved_without_reporting_is_logged_as_taken_to_be_there() {
    // Arrange -- its application gets to the placement, and the report of
    // it is missed.
    let (platform, mut manager, _) = slow_swap();
    platform.catch_up(1);
    manager.transition_lines();

    // Act
    wait_for_landing(&platform, &mut manager);

    // Assert -- taken to be there half a second after the placement.
    let lines = manager.transition_lines();
    let overdue = "transition: 0x1 is taken to be in place, not reported there 500 ms after its \
                   placement was posted";
    assert_eq!(lines, [overdue], "{lines:#?}");
}

#[test]
fn a_window_that_had_not_moved_by_the_end_of_the_wait_is_logged_as_it_reports() {
    // Arrange -- its application has not got to the placement once the
    // wait by time is over.
    let (platform, mut manager, _) = slow_swap();
    manager.transition_lines();
    wait_for_landing(&platform, &mut manager);
    let waited = manager.transition_lines();

    // Act -- a second later it gets there, 10 pixels wider than placed, as a
    // console sized to whole text cells may be, and reports it.
    platform.advance(Duration::from_secs(1));
    platform.catch_up(1);
    platform.edit(1, |w| w.rect.width += 10);
    manager.handle_event(&report());

    // Assert -- nothing until it reports.
    assert!(waited.is_empty(), "{waited:#?}");
    let lines = manager.transition_lines();
    let late = "transition: 0x1 is taken to be in place as it moves at last, 1500 ms after its \
                placement was posted";
    assert_eq!(lines, [late], "{lines:#?}");
}

#[test]
fn a_window_kept_from_where_it_was_placed_is_logged_once_nothing_is_on_its_way() {
    // Arrange -- window 1 lands 10 pixels wider than placed, as a window
    // with a minimum size does, and reports it.
    let (platform, mut manager, _) = slow_swap();
    manager.transition_lines();
    platform.catch_up(1);
    platform.edit(1, |w| w.rect.width += 10);
    manager.handle_event(&report());
    let landed = manager.transition_lines();

    // Act -- seconds later, with no frame step taken meanwhile, it moves a
    // little, and reports that too.
    platform.advance(ASYNC_GRACE * 2);
    platform.edit(1, |w| w.rect.x += 1);
    manager.handle_event(&report());

    // Assert -- nothing while what was posted to it may be on its way.
    assert!(landed.is_empty(), "{landed:#?}");
    let lines = manager.transition_lines();
    let away = "transition: 0x1 is not where it was placed, with nothing on its way, ";
    assert_eq!(starting(&lines, away), 1, "{lines:#?}");
}

#[test]
fn a_window_placed_after_a_move_that_does_not_wait_is_not_logged_as_landing() {
    // Arrange -- three windows, as window 2 is restored into another slot;
    // windows 1 and 2 swapped, and window 2 was minimized on its way.
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0), (3, 0)]);
    platform.land_pre_placements();
    let mut manager = start(&platform, None, &animated());
    swap(&mut manager);
    step(&platform, &mut manager);
    platform.edit(2, |w| w.minimized = true);
    manager.handle_event(&WindowEvent::Minimized { window: wid(2) });
    settle(&platform, &mut manager);
    manager.transition_lines();

    // Act -- restored, it is moved without waiting, then placed in full,
    // behind the frames that were on their way to it, and lands.
    platform.edit(2, |w| w.minimized = false);
    manager.handle_event(&WindowEvent::Restored { window: wid(2) });
    settle(&platform, &mut manager);
    manager.handle_event(&WindowEvent::LocationChanged { window: wid(2) });

    // Assert -- it had no transition to end.
    let lines = manager.transition_lines();
    assert_eq!(starting(&lines, "transition: 0x2 is "), 0, "{lines:#?}");
}
