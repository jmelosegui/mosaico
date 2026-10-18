//! The pointer on a bar's own window: how it is made, switched, never
//! activated, and the buttons pressed on it.

use mosaico_core::bar::{BarSurface, Button, PointerEvent};
use windows::Win32::UI::WindowsAndMessaging::{
    IsWindowVisible, MA_NOACTIVATE, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP,
    WM_MOUSEACTIVATE, WM_RBUTTONDOWN, WM_RBUTTONUP, WS_EX_NOACTIVATE, WS_EX_TRANSPARENT,
};

use super::input_support::{at, ex_style, hidden_bar, holds_pointer, send};

#[test]
fn a_bar_is_made_hidden_never_activated_and_letting_the_pointer_through() {
    // Arrange
    let Some((bar, _sink)) = hidden_bar() else {
        return;
    };

    // Act
    let style = ex_style(&bar);
    // SAFETY: reads the state of a window this thread created.
    let visible = unsafe { IsWindowVisible(bar.hwnd).as_bool() };

    // Assert
    assert_ne!(style & WS_EX_NOACTIVATE.0, 0, "the bar can be activated");
    assert_ne!(style & WS_EX_TRANSPARENT.0, 0, "the bar takes the pointer");
    assert!(!visible && bar.input.click_through());
}

#[test]
fn a_bar_switched_to_take_the_pointer_is_no_longer_click_through() {
    // Arrange
    let Some((mut bar, _sink)) = hidden_bar() else {
        return;
    };

    // Act
    bar.set_click_through(false)
        .expect("the bar takes the pointer");
    let taking = ex_style(&bar);
    bar.set_click_through(true)
        .expect("the bar lets it through");
    let through = ex_style(&bar);

    // Assert
    assert_eq!(taking & WS_EX_TRANSPARENT.0, 0);
    assert_ne!(through & WS_EX_TRANSPARENT.0, 0);
    assert!(bar.input.click_through());
}

#[test]
fn a_click_never_activates_the_bar() {
    // Arrange
    let Some((bar, _sink)) = hidden_bar() else {
        return;
    };

    // Act
    let answer = send(&bar, WM_MOUSEACTIVATE, 0, 0);

    // Assert
    assert_eq!(answer, MA_NOACTIVATE as isize);
}

#[test]
fn a_press_holds_the_pointer_until_its_release() {
    // Arrange
    let Some((bar, sink)) = hidden_bar() else {
        return;
    };

    // Act -- MK_LBUTTON is down with the press, and nothing after.
    send(&bar, WM_LBUTTONDOWN, 0x0001, at(30, 10));
    let held = holds_pointer(&bar);
    send(&bar, WM_LBUTTONUP, 0, at(40, -5));

    // Assert -- the release is off the bar, above it.
    assert!(held && !holds_pointer(&bar));
    assert_eq!(
        sink.take(),
        [
            PointerEvent::Press {
                button: Button::Left,
                x: 30,
                y: 10
            },
            PointerEvent::Release {
                button: Button::Left,
                x: 40,
                y: -5
            },
        ]
    );
}

#[test]
fn the_right_and_middle_buttons_are_reported_as_such() {
    // Arrange
    let Some((bar, sink)) = hidden_bar() else {
        return;
    };

    // Act
    for msg in [WM_RBUTTONDOWN, WM_RBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP] {
        send(&bar, msg, 0, at(5, 5));
    }

    // Assert
    let buttons: Vec<Button> = sink
        .take()
        .into_iter()
        .filter_map(|event| match event {
            PointerEvent::Press { button, .. } | PointerEvent::Release { button, .. } => {
                Some(button)
            }
            _ => None,
        })
        .collect();
    let expected = [Button::Right, Button::Right, Button::Middle, Button::Middle];
    assert_eq!(buttons, expected);
}
