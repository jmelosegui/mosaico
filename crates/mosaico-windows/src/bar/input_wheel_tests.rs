//! The wheel over a bar's own window, the pointer leaving it, and the
//! moves it does not report.

use mosaico_core::bar::PointerEvent;
use windows::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture;
use windows::Win32::UI::WindowsAndMessaging::{
    SWP_NOACTIVATE, SWP_NOZORDER, SetWindowPos, WM_CAPTURECHANGED, WM_LBUTTONDOWN, WM_LBUTTONUP,
    WM_MOUSEHWHEEL, WM_MOUSEMOVE, WM_MOUSEWHEEL,
};

use super::super::input_messages::WM_MOUSELEAVE;
use super::input_support::{at, hidden_bar, send};

#[test]
fn the_wheel_is_reported_in_notches_where_the_pointer_is_on_the_bar() {
    // Arrange -- the bar at (100, 200) on the screen, still hidden.
    let Some((bar, sink)) = hidden_bar() else {
        return;
    };
    // SAFETY: moves a window this thread created, without showing it.
    unsafe {
        let _ = SetWindowPos(
            bar.hwnd,
            None,
            100,
            200,
            300,
            40,
            SWP_NOACTIVATE | SWP_NOZORDER,
        );
    }

    // Act -- two notches toward the user, then a tilt to the left, both
    // at (130, 210) on the screen.
    send(
        &bar,
        WM_MOUSEWHEEL,
        (-240i16 as u16 as usize) << 16,
        at(130, 210),
    );
    send(
        &bar,
        WM_MOUSEHWHEEL,
        (-120i16 as u16 as usize) << 16,
        at(130, 210),
    );

    // Assert
    let (x, y) = (30, 10);
    assert_eq!(
        sink.take(),
        [
            PointerEvent::Wheel { notches: 2.0, x, y },
            PointerEvent::Wheel {
                notches: -1.0,
                x,
                y
            },
        ]
    );
}

#[test]
fn the_pointer_leaving_is_reported_unless_a_press_holds_it() {
    // Arrange
    let Some((bar, sink)) = hidden_bar() else {
        return;
    };

    // Act
    send(&bar, WM_MOUSELEAVE, 0, 0);
    send(&bar, WM_LBUTTONDOWN, 0x0001, at(5, 5));
    send(&bar, WM_MOUSELEAVE, 0, 0);
    send(&bar, WM_LBUTTONUP, 0, at(5, 5));

    // Assert
    let leaves = sink
        .take()
        .into_iter()
        .filter(|event| *event == PointerEvent::Leave)
        .count();
    assert_eq!(leaves, 1);
}

#[test]
fn losing_the_pointer_during_a_press_counts_as_leaving() {
    // Arrange
    let Some((bar, sink)) = hidden_bar() else {
        return;
    };
    send(&bar, WM_LBUTTONDOWN, 0x0001, at(5, 5));
    sink.take();

    // Act -- another window took the pointer.
    send(&bar, WM_CAPTURECHANGED, 0, 0);
    // SAFETY: gives back the pointer this thread's window took.
    unsafe {
        let _ = ReleaseCapture();
    }

    // Assert
    assert_eq!(sink.take(), [PointerEvent::Leave]);
}

#[test]
fn moves_over_the_bar_are_not_reported() {
    // Arrange
    let Some((bar, sink)) = hidden_bar() else {
        return;
    };

    // Act
    send(&bar, WM_MOUSEMOVE, 0, at(10, 10));
    send(&bar, WM_MOUSEMOVE, 0, at(20, 10));

    // Assert
    assert!(sink.take().is_empty());
}
