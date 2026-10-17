//! The preview of where a dragged window lands, hidden as the drag ends or
//! is interrupted, shown again behind the window as a focus change leaves
//! the drag going, and never shown with `drag_to_swap` off.

use crate::id::wid;
use crate::tiler::tests::Harness;
use crate::tiler::{Effect, Input, Settings};
use crate::{Action, Direction, WindowEvent};

use super::super::support::{center, drop_at, grab, report_at, shown, swapping};

/// Something that ends or interrupts the drag of window 1.
type End = fn(&mut Harness);

/// The user drags window 1 over window 2, and the preview shows there.
fn showing() -> Harness {
    let mut h = Harness::with_settings(swapping(), 1, &[(1, 0), (2, 0)]);
    let right = h.rect_of(2);
    grab(&mut h, 1);
    report_at(&mut h, 1, center(right));
    h
}

#[test]
fn the_preview_is_hidden_as_the_window_is_dropped() {
    // Arrange
    let mut h = showing();
    let right = h.rect_of(2);

    // Act
    let effects = drop_at(&mut h, 1, center(right));

    // Assert
    assert_eq!(effects.first(), Some(&Effect::HidePreview));
    assert_eq!(h.desktop.preview, None);
}

#[test]
fn whatever_ends_or_interrupts_the_drag_hides_the_preview() {
    // Arrange -- each of them, before the drop.
    let ends: [(&str, End); 6] = [
        ("a key", |h| {
            h.action(Action::Focus(Direction::Right));
        }),
        ("a display change noted", |h| {
            h.step(Input::DisplayChanging);
        }),
        ("the tiler stopping", |h| {
            h.step(Input::Shutdown);
        }),
        ("another window taking focus", |h| {
            h.event(WindowEvent::Focused { window: wid(2) });
        }),
        ("the window closing", |h| {
            h.desktop.windows.remove(&wid(1));
            h.event(WindowEvent::Destroyed { window: wid(1) });
        }),
        ("another drag", |h| {
            h.event(WindowEvent::move_started(wid(2)));
        }),
    ];

    for (what, end) in ends {
        let mut h = showing();

        // Act
        end(&mut h);

        // Assert
        assert_eq!(h.desktop.preview, None, "{what}");
    }
}

#[test]
fn the_dragged_window_reporting_focus_has_the_preview_shown_behind_it_again() {
    // Arrange -- as it comes back in front of a window the tiler gave
    // focus meanwhile.
    let mut h = showing();
    let right = h.rect_of(2);

    // Act
    let effects = h.event(WindowEvent::Focused { window: wid(1) }).to_vec();

    // Assert -- where it showed.
    assert_eq!(shown(&effects), Some(right));
}

#[test]
fn a_window_that_opens_mid_drag_reporting_its_focus_has_the_preview_shown_again() {
    // Arrange -- window 3 opens while the preview shows, and the tiler
    // gives it focus.
    let mut h = showing();
    let right = h.rect_of(2);
    h.open(3, 0);

    // Act -- window 3 reports the focus it was given.
    let effects = h.event(WindowEvent::Focused { window: wid(3) }).to_vec();

    // Assert -- just behind the dragged window, where it showed.
    assert_eq!(shown(&effects), Some(right));
}

#[test]
fn a_drag_a_key_interrupted_shows_no_preview_as_it_goes_on() {
    // Arrange
    let mut h = showing();
    let right = h.rect_of(2);
    h.action(Action::Focus(Direction::Right));

    // Act -- window 2 reports the focus the key gave it, and the window
    // goes on moving over it.
    let mut effects = h.event(WindowEvent::Focused { window: wid(2) }).to_vec();
    effects.extend(report_at(&mut h, 1, (right.x + 20, right.y + 30)));

    // Assert
    assert_eq!(shown(&effects), None);
    assert_eq!(h.desktop.preview, None);
}

#[test]
fn with_drag_to_swap_off_no_preview_shows() {
    // Arrange -- the near miss.
    let mut h = Harness::with_settings(Settings::default(), 1, &[(1, 0), (2, 0)]);
    let right = h.rect_of(2);
    grab(&mut h, 1);

    // Act
    let effects = report_at(&mut h, 1, center(right));

    // Assert
    assert_eq!(shown(&effects), None);
}
