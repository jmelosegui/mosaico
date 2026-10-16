//! Echoes of a workspace switch whose hide or show lands late, posted
//! behind frames on their way to the window.

use std::time::Instant;

use crate::WindowEvent;
use crate::config::HidingBehaviour;
use crate::frames::ASYNC_GRACE;
use crate::id::wid;

use super::*;

/// The tiler hides window `id` by minimizing it, and shows it again.
fn away_and_back(ex: &mut Expectations, id: u64, now: Instant) {
    let (window, how) = (wid(id), HidingBehaviour::Minimize);
    ex.applied(&Effect::Hide { window, how }, now);
    ex.applied(&Effect::Show { window, how }, now);
}

#[test]
fn a_hide_reported_after_its_window_was_shown_again_is_an_echo_for_a_while() {
    // Arrange -- each minimize waited behind a frame, and the show behind
    // the minimize.
    let now = Instant::now();
    let mut ex = Expectations::default();
    for window in [1, 2] {
        away_and_back(&mut ex, window, now);
    }

    // Act -- window 1's minimize is reported in time, twice, seconds after
    // the show; window 2's too late.
    let soon = now + ASYNC_GRACE / 2;
    let one = ex.echo(&WindowEvent::Minimized { window: wid(1) }, soon);
    let again = ex.echo(&WindowEvent::Minimized { window: wid(1) }, soon);
    let late = now + ASYNC_GRACE;
    let two = ex.echo(&WindowEvent::Minimized { window: wid(2) }, late);

    // Assert -- taken for ours once, and only in time.
    assert!(one.is_some());
    assert!(again.is_none());
    assert!(two.is_none());
}

#[test]
fn a_restore_reported_seconds_after_the_show_is_an_echo_for_as_long() {
    // Arrange
    let now = Instant::now();
    let mut ex = Expectations::default();
    for window in [1, 2] {
        away_and_back(&mut ex, window, now);
    }

    // Act -- window 1's restore is reported in time, window 2's too late.
    let (soon, late) = (now + ASYNC_GRACE / 2, now + ASYNC_GRACE);
    let one = ex.echo(&WindowEvent::Restored { window: wid(1) }, soon);
    let two = ex.echo(&WindowEvent::Restored { window: wid(2) }, late);

    // Assert
    assert!(one.is_some());
    assert!(two.is_none());
}

#[test]
fn hiding_a_window_again_or_destroying_it_ends_what_its_show_was_to_report() {
    // Arrange -- both windows are still to report their minimize and their
    // restore. Window 1 is hidden again by cloaking, which reports nothing;
    // window 2 is destroyed.
    let now = Instant::now();
    let mut ex = Expectations::default();
    for window in [1, 2] {
        away_and_back(&mut ex, window, now);
    }
    let (window, how) = (wid(1), HidingBehaviour::Cloak);
    ex.applied(&Effect::Hide { window, how }, now);
    ex.echo(&WindowEvent::Destroyed { window: wid(2) }, now);

    // Act
    let reports = [1, 2].map(|id| {
        let window = wid(id);
        let hidden = ex.echo(&WindowEvent::Minimized { window }, now);
        let restored = ex.echo(&WindowEvent::Restored { window }, now);
        (hidden, restored)
    });

    // Assert -- none of them is ours any more.
    assert_eq!(reports, [(None, None), (None, None)]);
}

#[test]
fn a_restore_is_expected_only_from_a_show_that_restores_the_window() {
    // Arrange -- three windows hidden by minimizing. The hides of windows
    // 1 and 2 are reported; the user restored window 1 from the taskbar
    // before its show, and window 2 is still minimized at its show. The
    // hide of window 3 is still to land.
    let now = Instant::now();
    let mut ex = Expectations::default();
    let how = HidingBehaviour::Minimize;
    for id in [1, 2, 3] {
        ex.applied(
            &Effect::Hide {
                window: wid(id),
                how,
            },
            now,
        );
    }
    for id in [1, 2] {
        ex.echo(&WindowEvent::Minimized { window: wid(id) }, now);
    }
    for (id, minimized) in [(1, false), (2, true), (3, false)] {
        ex.showing(wid(id), minimized);
        ex.applied(
            &Effect::Show {
                window: wid(id),
                how,
            },
            now,
        );
    }

    // Act
    let restored = |id| WindowEvent::Restored { window: wid(id) };
    let echoes = [1, 2, 3].map(|id| ex.echo(&restored(id), now).is_some());

    // Assert -- only the shows that restore their window report it.
    assert_eq!(echoes, [false, true, true]);
}
