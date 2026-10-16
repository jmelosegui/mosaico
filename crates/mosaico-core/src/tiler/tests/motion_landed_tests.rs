//! The end of a motion that does not leave the window where it was going
//! at once. A window placed by a posted placement is repainted again once it
//! reports being there, since a posted repaint only repairs what was wrong
//! before the changes queued ahead of it, and has the border its motion
//! kept drawn then, which until then stays where the window was last found.
//! A window stopped on its way starts its next motion with a fresh border.

use crate::frames::ASYNC_GRACE;
use crate::id::wid;
use crate::tiler::tests::fake_platform::Call;
use crate::{Action, Direction, WindowEvent};

use super::posted_tests::slow_swap;
use super::{REFRESH, animated, frame_by_frame, settle, swap, two};

/// Whether `calls` repaint window `id`.
fn repaints(calls: &[Call], id: u64) -> bool {
    calls.contains(&Call::Repaint(wid(id)))
}

#[test]
fn a_window_placed_behind_its_frames_is_repainted_again_once_it_reports_being_there() {
    // Arrange
    let (platform, mut manager, _) = slow_swap();
    let report = WindowEvent::LocationChanged { window: wid(1) };

    // Act -- window 1 reports where it is as its frame lands, then as it
    // gets to its placement.
    let frame = platform.window(1).queued[0].0;
    platform.catch_up_through(1, frame);
    manager.handle_event(&report);
    let on_its_way = platform.take_calls();
    platform.catch_up(1);
    manager.handle_event(&report);
    let there = platform.take_calls();

    // Assert -- repainted once there, and not before.
    assert!(
        !on_its_way.contains(&Call::Repaint(wid(1))),
        "{on_its_way:?}"
    );
    assert!(there.contains(&Call::Repaint(wid(1))), "{there:?}");
}

#[test]
fn a_window_that_rounds_its_size_is_repainted_again_once_it_reports_being_about_there() {
    // Arrange
    let (platform, mut manager, _) = slow_swap();
    platform.catch_up(1);

    // Act -- it lands 3 pixels narrower than placed, as a window that
    // rounds its size to its text cells does, and reports where it is.
    platform.edit(1, |w| w.rect.width -= 3);
    manager.handle_event(&WindowEvent::LocationChanged { window: wid(1) });

    // Assert
    assert!(repaints(&platform.take_calls(), 1));
}

#[test]
fn a_window_kept_away_from_where_it_was_placed_is_repainted_once_nothing_may_be_on_its_way() {
    // Arrange -- window 1 lands 10 pixels wider than placed, as a window
    // with a minimum size does, and reports it.
    let (platform, mut manager, _) = slow_swap();
    platform.catch_up(1);
    platform.edit(1, |w| w.rect.width += 10);
    let report = WindowEvent::LocationChanged { window: wid(1) };
    manager.handle_event(&report);
    let landed = platform.take_calls();

    // Act -- seconds later it moves a little, and reports that too.
    platform.advance(ASYNC_GRACE * 2);
    platform.edit(1, |w| w.rect.x += 1);
    manager.handle_event(&report);

    // Assert -- not while what was posted to it may be on its way.
    assert!(!repaints(&landed, 1), "{landed:?}");
    assert!(repaints(&platform.take_calls(), 1));
}

#[test]
fn the_border_of_a_window_that_fell_behind_is_drawn_once_it_reports_being_in_place() {
    // Arrange -- window 1, focused, stops responding once it got to the
    // first frame of a swap, which ends the swap with its placement queued
    // behind the frame.
    let (platform, mut manager) = two(&animated());
    let right = platform.window(2).rect;
    swap(&mut manager);
    platform.advance(REFRESH);
    manager.frame();
    platform.edit(1, |w| w.hung = true);
    let ended = frame_by_frame(&platform, &mut manager).concat();

    // Act -- its application responds again, gets to what is queued for
    // it, and reports where that leaves the window.
    platform.edit(1, |w| w.hung = false);
    platform.catch_up(1);
    manager.handle_event(&WindowEvent::LocationChanged { window: wid(1) });

    // Assert -- its border is drawn at its slot only then.
    let drawn = |calls: &[Call]| {
        let at_slot =
            |call: &Call| matches!(call, Call::ShowBorder(w, s) if *w == wid(1) && s.rect == right);
        calls.iter().any(at_slot)
    };
    assert!(!drawn(&ended), "{ended:?}");
    assert!(drawn(&platform.take_calls()));
}

#[test]
fn a_window_maximized_on_its_way_starts_its_next_motion_with_a_fresh_border() {
    // Arrange -- only the focused window has a border. Window 1, on its
    // way right, has it hidden as focus moves to window 2, and is
    // maximized two frames in; then it is restored.
    let mut config = animated();
    config.borders.colors.unfocused = "none".into();
    let (platform, mut manager) = two(&config);
    swap(&mut manager);
    manager.handle_action(&Action::Focus(Direction::Left));
    for _ in 0..2 {
        platform.advance(REFRESH);
        manager.frame();
    }
    platform.edit(1, |w| w.maximized = true);
    settle(&platform, &mut manager);
    platform.edit(1, |w| w.maximized = false);
    manager.handle_event(&WindowEvent::LocationChanged { window: wid(1) });
    settle(&platform, &mut manager);

    // Act -- focused again, and moved back left.
    manager.handle_action(&Action::Focus(Direction::Right));
    manager.handle_action(&Action::Move(Direction::Left));
    manager.flush_pending();
    platform.take_calls();
    settle(&platform, &mut manager);

    // Assert -- its border follows it this time.
    let calls = platform.take_calls();
    let moved = |call: &Call| matches!(call, Call::MoveBorder(window, _) if *window == wid(1));
    assert!(calls.iter().any(moved), "{calls:?}");
}
