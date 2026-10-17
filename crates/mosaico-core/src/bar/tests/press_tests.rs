//! Which presses and releases click an item of a bar.

use super::super::{BarTarget, Button, PointerEvent};
use super::support::{clicks, drawn, press, release};

#[test]
fn a_press_and_a_release_on_the_same_label_click_it() {
    // Act
    let clicked = clicks(&[press(35), release(45)], &drawn());

    // Assert
    assert_eq!(clicked, [(Button::Left, BarTarget::Workspace(1))]);
}

#[test]
fn a_press_on_one_label_and_a_release_on_another_click_nothing() {
    // Act
    let clicked = clicks(&[press(5), release(35)], &drawn());

    // Assert
    assert!(clicked.is_empty());
}

#[test]
fn a_release_off_every_item_clicks_nothing() {
    // Act -- released in the gap after the first label, and off the bar.
    let gap = clicks(&[press(5), release(25)], &drawn());
    let off = clicks(&[press(5), release(-40)], &drawn());

    // Assert
    assert!(gap.is_empty() && off.is_empty());
}

#[test]
fn a_release_without_a_press_clicks_nothing() {
    // Act
    let clicked = clicks(&[release(5)], &drawn());

    // Assert
    assert!(clicked.is_empty());
}

#[test]
fn a_press_in_a_gap_clicks_nothing_on_release_over_a_label() {
    // Act
    let clicked = clicks(&[press(25), release(5)], &drawn());

    // Assert
    assert!(clicked.is_empty());
}

#[test]
fn the_pointer_leaving_forgets_the_press() {
    // Act
    let clicked = clicks(&[press(5), PointerEvent::Leave, release(5)], &drawn());

    // Assert
    assert!(clicked.is_empty());
}

#[test]
fn a_click_ends_the_press() {
    // Act
    let clicked = clicks(&[press(5), release(5), release(5)], &drawn());

    // Assert
    assert_eq!(clicked.len(), 1);
}
