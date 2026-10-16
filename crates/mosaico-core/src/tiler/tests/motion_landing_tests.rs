//! Waiting for a window to land where a posted placement takes it: not for
//! one the desktop has there already, where it was sent last, until
//! `LANDING_WAIT` is over for one that moved and never reports being
//! there, and for one whose application stops responding as it is placed.
//! A border decided meanwhile, for a placement that snaps too, is drawn
//! once the window lands.

use crate::frames::ASYNC_GRACE;
use crate::id::wid;
use crate::tiler::Platform;
use crate::tiler::tests::fake_platform::Call;
use crate::{Action, Direction, WindowEvent};

use super::posted_tests::slow_swap;
use super::{animated, bordered_at, frame_by_frame, settle, swap, two, wait_for_landing};

#[path = "motion_landing_order_tests.rs"]
mod order_tests;
#[path = "motion_landing_wait_tests.rs"]
mod wait_tests;

/// Whether `calls` draw a border for window `id`.
fn bordered(calls: &[Call], id: u64) -> bool {
    let drawn = |call: &Call| matches!(call, Call::ShowBorder(w, _) if w.raw() == id);
    calls.iter().any(drawn)
}

#[test]
fn a_window_its_frames_took_into_place_is_bordered_and_repainted_once_as_its_motion_ends() {
    // Arrange
    let (platform, mut manager) = two(&animated());
    let (left, right) = (platform.window(1).rect, platform.window(2).rect);
    swap(&mut manager);

    // Act -- the last frames take both windows into their slots, so their
    // placements move nothing, and nothing reports them.
    let steps = frame_by_frame(&platform, &mut manager);
    let after = platform.take_calls();

    // Assert -- each is bordered and repainted once, as its motion ends,
    // and nothing is left to wait for.
    let last = steps.last().expect("frame steps");
    for (id, slot) in [(1, right), (2, left)] {
        let repaints = last.iter().filter(|&c| *c == Call::Repaint(wid(id)));
        assert_eq!(repaints.count(), 1, "{last:?}");
        assert!(bordered_at(last, id, slot), "{last:?}");
    }
    assert!(after.is_empty(), "{after:?}");
    assert_eq!(manager.frame_due(Some(platform.now())), None);
}

#[test]
fn a_window_that_lands_away_from_its_slot_is_repainted_and_bordered_once_the_wait_is_over() {
    // Arrange -- window 1 lands 10 pixels wider than placed, as a console
    // sized to whole text cells may, and reports it while what was posted
    // to it may still be on its way. It reports nothing after that.
    let (platform, mut manager, [_, right]) = slow_swap();
    platform.catch_up(1);
    platform.edit(1, |w| w.rect.width += 10);
    manager.handle_event(&WindowEvent::LocationChanged { window: wid(1) });
    let reported = platform.take_calls();

    // Act
    wait_for_landing(&platform, &mut manager);

    // Assert -- only then.
    assert!(!reported.contains(&Call::Repaint(wid(1))), "{reported:?}");
    assert!(!bordered(&reported, 1), "{reported:?}");
    let calls = platform.take_calls();
    assert!(calls.contains(&Call::Repaint(wid(1))), "{calls:?}");
    assert!(bordered_at(&calls, 1, right), "{calls:?}");
}

#[test]
fn a_window_whose_application_stops_responding_as_it_is_placed_is_waited_for() {
    // Arrange -- a swap long over, after which window 1 moved, so changes
    // to it are made at once again. A display change is noted, so the next
    // placements snap.
    let (platform, mut manager) = two(&animated());
    swap(&mut manager);
    settle(&platform, &mut manager);
    platform.advance(ASYNC_GRACE * 2);
    platform.edit(1, |w| w.rect.x += 1);
    manager.display_changing();
    platform.take_calls();

    // Act -- the swap is undone, and window 1's application stops
    // responding just as its placement is made, which then waits for it.
    platform.edit(1, |w| w.hangs_on_place = true);
    manager.handle_action(&Action::Move(Direction::Left));
    manager.flush_pending();

    // Assert -- the platform says so right after the placement, so window
    // 1's border waits for it to land.
    let calls = platform.take_calls();
    let one = platform.window(1);
    assert!(one.hung && !one.queued.is_empty(), "{one:?}");
    assert!(!bordered(&calls, 1), "{calls:?}");
}

#[test]
fn a_border_decided_for_a_snap_placement_behind_a_slow_window_is_drawn_once_it_lands() {
    // Arrange -- window 1's application is busy: a swap ends with its
    // placement queued behind its frames, while the desktop still has it in
    // the slot it left. Then a display change is noted, so the next
    // placements snap.
    let (platform, mut manager) = two(&animated());
    let left = platform.window(1).rect;
    platform.edit(1, |w| w.frame_delay = 40);
    swap(&mut manager);
    frame_by_frame(&platform, &mut manager);
    manager.display_changing();
    platform.take_calls();

    // Act -- the swap is undone, which posts window 1's placement back
    // behind what takes it away first, though the desktop has it there
    // still; then it gets to all of it, and reports where that leaves it.
    manager.handle_action(&Action::Move(Direction::Left));
    manager.flush_pending();
    let undone = platform.take_calls();
    platform.catch_up(1);
    manager.handle_event(&WindowEvent::LocationChanged { window: wid(1) });

    // Assert -- its border waits for it, and is drawn where it lands.
    assert!(!bordered(&undone, 1), "{undone:?}");
    assert!(bordered_at(&platform.take_calls(), 1, left));
}
