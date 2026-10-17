//! Clicks with other buttons, around redraws and turns of the wheel.

use crate::bar::{BarSide, WidgetRef};

use super::super::{BarTarget, Button, PointerEvent, Press};
use super::support::{clicks, drawn, press, release};

#[test]
fn a_release_of_another_button_clicks_nothing() {
    // Arrange
    let right = PointerEvent::Release {
        button: Button::Right,
        x: 5,
        y: 10,
    };

    // Act
    let clicked = clicks(&[press(5), right, release(5)], &drawn());

    // Assert -- the right button's release ends the press too.
    assert!(clicked.is_empty());
}

#[test]
fn a_click_on_a_widget_names_the_widget_and_the_button() {
    // Arrange
    let middle = |pressed| {
        let (x, y, button) = (100, 10, Button::Middle);
        if pressed {
            PointerEvent::Press { button, x, y }
        } else {
            PointerEvent::Release { button, x, y }
        }
    };

    // Act
    let clicked = clicks(&[middle(true), middle(false)], &drawn());

    // Assert
    let pill = BarTarget::Widget(WidgetRef {
        side: BarSide::Left,
        index: 1,
    });
    assert_eq!(clicked, [(Button::Middle, pill)]);
}

#[test]
fn a_redraw_between_press_and_release_clicks_the_label_still_there() {
    // Arrange
    let mut state = Press::default();
    state.take(press(35), &drawn());

    // Act -- drawn again in the same place, as every refresh does.
    let clicked = state.take(release(35), &drawn());

    // Assert
    assert_eq!(clicked, Some((Button::Left, BarTarget::Workspace(1))));
}

#[test]
fn a_redraw_that_moves_another_label_under_the_pointer_clicks_nothing() {
    // Arrange -- the labels shift: workspace 0 is now where 1 was.
    let mut state = Press::default();
    state.take(press(35), &drawn());
    let mut moved = drawn();
    moved[0].rect.x = 30;
    moved[1].rect.x = 0;

    // Act
    let clicked = state.take(release(35), &moved);

    // Assert
    assert_eq!(clicked, None);
}

#[test]
fn the_wheel_does_not_end_a_press() {
    // Arrange
    let wheel = PointerEvent::Wheel {
        notches: 1.0,
        x: 5,
        y: 10,
    };

    // Act
    let clicked = clicks(&[press(5), wheel, release(5)], &drawn());

    // Assert
    assert_eq!(clicked, [(Button::Left, BarTarget::Workspace(0))]);
}
