//! Resizes handled as before, with the window going back to its slot:
//! with `resize_splits` off, cancelled with Escape, interrupted by a key,
//! in monocle, and with the option turned off by a reload; and a resize
//! whose end has no pointer, which moves the split all the same.

use crate::config::Config;
use crate::id::wid;
use crate::tiler::tests::Harness;
use crate::tiler::{Input, Settings};
use crate::{Action, Rect, WindowEvent};

use super::super::support::center;
use super::support::{RIGHT, grab_to, holding, resize_to, resizing, unmoved};

/// Windows 1 and 2 side by side, with `settings`.
fn pair(settings: Settings) -> Harness {
    Harness::with_settings(settings, 1, &[(1, 0), (2, 0)])
}

/// Window 1 of `h`, 300 pixels wider.
fn widened(h: &Harness) -> Rect {
    let left = h.rect_of(1);
    Rect {
        width: left.width + 300,
        ..left
    }
}

#[test]
fn with_resize_splits_off_a_resized_window_goes_back() {
    // Arrange -- the near miss: the same resize, the option off.
    let mut h = pair(Settings::default());
    let (left, right) = (h.rect_of(1), h.rect_of(2));

    // Act
    let wider = widened(&h);
    resize_to(&mut h, 1, RIGHT, wider);

    // Assert
    assert_eq!((h.rect_of(1), h.rect_of(2)), (left, right));
    assert!(unmoved(&h));
}

#[test]
fn escape_during_a_resize_changes_nothing() {
    // Arrange
    let mut h = pair(resizing());
    let (left, right, wider) = (h.rect_of(1), h.rect_of(2), widened(&h));
    grab_to(&mut h, 1, RIGHT, wider);

    // Act -- the system puts it back where it started, the pointer is
    // elsewhere.
    h.window(1).rect = left;
    let pointer = Some(center(right));
    h.event(WindowEvent::Moved {
        window: wid(1),
        pointer,
    });

    // Assert
    assert_eq!((h.rect_of(1), h.rect_of(2)), (left, right));
    assert!(unmoved(&h));
}

#[test]
fn a_key_pressed_during_a_resize_leaves_its_end_as_before() {
    // Arrange
    let mut h = pair(resizing());
    let (left, wider) = (h.rect_of(1), widened(&h));
    grab_to(&mut h, 1, RIGHT, wider);

    // Act
    h.action(Action::Retile);
    let pointer = Some(holding(wider, RIGHT));
    h.event(WindowEvent::Moved {
        window: wid(1),
        pointer,
    });

    // Assert -- back in its slot.
    assert_eq!(h.rect_of(1), left);
    assert!(unmoved(&h));
}

#[test]
fn a_window_resized_in_monocle_goes_back_and_moves_no_split() {
    // Arrange -- window 1 fills the monitor.
    let mut h = pair(resizing());
    h.action(Action::ToggleMonocle);
    let front = h.rect_of(1);

    // Act -- its right edge 300 pixels left.
    let narrower = Rect {
        width: front.width - 300,
        ..front
    };
    resize_to(&mut h, 1, RIGHT, narrower);

    // Assert
    assert_eq!(h.rect_of(1), front);
    assert!(unmoved(&h));
}

#[test]
fn a_resize_whose_end_has_no_pointer_still_moves_the_split() {
    // Arrange -- the split comes from where the window is, not the pointer.
    let mut h = pair(resizing());
    let wider = widened(&h);
    grab_to(&mut h, 1, RIGHT, wider);

    // Act
    let pointer = None;
    h.event(WindowEvent::Moved {
        window: wid(1),
        pointer,
    });

    // Assert
    assert_eq!(h.rect_of(1), wider);
    assert!(!unmoved(&h));
}

#[test]
fn a_config_read_again_with_resize_splits_off_has_the_next_resize_go_back() {
    // Arrange -- on at the start, off in the config read again.
    let mut h = pair(resizing());
    let left = h.rect_of(1);
    let mut config = Config::default();
    config.mouse.resize_splits = false;
    h.step(Input::Config(Box::new(config)));

    // Act
    let wider = widened(&h);
    resize_to(&mut h, 1, RIGHT, wider);

    // Assert
    assert_eq!(h.rect_of(1), left);
    assert!(unmoved(&h));
}
