use mosaico_core::Rect;
use mosaico_core::bar::{BarSide, BarTarget, WidgetRef};
use windows::Win32::UI::WindowsAndMessaging::WM_MOUSEWHEEL;

use super::*;

/// The `lparam` of a mouse message at (`x`, `y`).
fn at(x: i16, y: i16) -> LPARAM {
    LPARAM(((y as u16 as isize) << 16) | x as u16 as isize)
}

/// The `wparam` of a wheel message that turned by `delta`.
fn turned(delta: i16) -> WPARAM {
    WPARAM((delta as u16 as usize) << 16)
}

#[test]
fn each_button_message_names_its_button_and_whether_it_went_down() {
    // Act / Assert
    assert_eq!(button(WM_LBUTTONDOWN), Some((Button::Left, true)));
    assert_eq!(button(WM_LBUTTONUP), Some((Button::Left, false)));
    assert_eq!(button(WM_RBUTTONDOWN), Some((Button::Right, true)));
    assert_eq!(button(WM_RBUTTONUP), Some((Button::Right, false)));
    assert_eq!(button(WM_MBUTTONDOWN), Some((Button::Middle, true)));
    assert_eq!(button(WM_MBUTTONUP), Some((Button::Middle, false)));
    assert_eq!(button(WM_MOUSELEAVE), None);
}

#[test]
fn a_point_is_read_signed_so_one_past_the_left_or_top_is_negative() {
    // Act / Assert
    assert_eq!(point(at(30, 10)), (30, 10));
    assert_eq!(point(at(-4, -1)), (-4, -1));
}

#[test]
fn turning_the_wheel_toward_the_user_scrolls_down() {
    // Act -- one notch toward the user, and two away from them.
    let down = notches(WM_MOUSEWHEEL, turned(-120));
    let up = notches(WM_MOUSEWHEEL, turned(240));

    // Assert
    assert_eq!((down, up), (1.0, -2.0));
}

#[test]
fn tilting_the_wheel_to_the_right_counts_as_down() {
    // Act
    let right = notches(WM_MOUSEHWHEEL, turned(120));
    let left = notches(WM_MOUSEHWHEEL, turned(-120));

    // Assert
    assert_eq!((right, left), (1.0, -1.0));
}

#[test]
fn a_touchpad_reports_fractions_of_a_notch() {
    // Act
    let scrolled = notches(WM_MOUSEWHEEL, turned(-30));

    // Assert
    assert_eq!(scrolled, 0.25);
}

#[test]
fn the_buttons_still_down_are_read_from_wparam() {
    // Act / Assert -- MK_LBUTTON, MK_RBUTTON, MK_MBUTTON and none.
    assert!(buttons_down(WPARAM(0x0001)));
    assert!(buttons_down(WPARAM(0x0002)));
    assert!(buttons_down(WPARAM(0x0010)));
    assert!(!buttons_down(WPARAM(0x0004)));
}

#[test]
fn the_hit_test_code_is_the_low_word() {
    // Act
    let code = hit_test(LPARAM((0x0201 << 16) | 1));

    // Assert
    assert_eq!(code, 1);
}

#[test]
fn the_hand_shows_only_over_an_item_that_takes_clicks() {
    // Arrange -- a label that takes clicks, then a pill that does not.
    let regions = [
        ItemRegion {
            target: BarTarget::Workspace(0),
            clickable: true,
            rect: Rect::new(0, 0, 20, 30),
        },
        ItemRegion {
            target: BarTarget::Widget(WidgetRef {
                side: BarSide::Left,
                index: 1,
            }),
            clickable: false,
            rect: Rect::new(30, 0, 100, 30),
        },
    ];

    // Act / Assert
    assert!(over_clickable(&regions, 10, 5));
    assert!(!over_clickable(&regions, 40, 5));
    assert!(!over_clickable(&regions, 25, 5));
}
