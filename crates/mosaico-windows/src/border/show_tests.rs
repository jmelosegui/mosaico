use windows::Win32::Foundation::RECT;
use windows::Win32::UI::WindowsAndMessaging::{GetWindowRect, IsWindowVisible};

use super::*;

/// A slot far off every monitor, so drawing there shows nothing.
const OFF_SCREEN: Rect = Rect {
    x: -20000,
    y: -20000,
    width: 40,
    height: 30,
};

/// Whether the overlay is shown.
fn visible(border: &Border) -> bool {
    // SAFETY: IsWindowVisible only reads the state of a window this
    // thread owns.
    unsafe { IsWindowVisible(border.hwnd).as_bool() }
}

#[test]
fn restacking_a_hidden_border_shows_it_again() {
    // Arrange -- a second overlay of our own, never drawn, stands for the
    // window the border surrounds. Without a desktop to make them on,
    // skip.
    let (Ok(border), Ok(window)) = (Border::new(), Border::new()) else {
        return;
    };
    let color = Color {
        r: 0x11,
        g: 0x22,
        b: 0x33,
    };
    border
        .show(&OFF_SCREEN, color, 4, 8, window.hwnd)
        .expect("an overlay of our own renders");
    border.hide();
    let hidden = !visible(&border);

    // Act -- what drawing it again the same way does.
    border.restack(&OFF_SCREEN, 4, window.hwnd);

    // Assert
    assert!(hidden, "hiding did not hide the border");
    assert!(visible(&border), "the border stayed hidden");
}

/// Where the overlay is on screen.
fn bounds(border: &Border) -> Rect {
    let mut r = RECT::default();
    // SAFETY: GetWindowRect only reads the rect of a window this thread
    // owns, into a RECT on the stack.
    unsafe { GetWindowRect(border.hwnd, &mut r) }.expect("an overlay of our own has a rect");
    Rect::new(r.left, r.top, r.right - r.left, r.bottom - r.top)
}

#[test]
fn a_hidden_border_restacked_at_another_slot_shows_only_there() {
    // Arrange -- drawn at one slot and hidden there, then shown again the
    // same way around another slot of the same size, as when its window
    // moves while it has no border. Without a desktop, skip.
    let (Ok(border), Ok(window)) = (Border::new(), Border::new()) else {
        return;
    };
    let color = Color {
        r: 0x11,
        g: 0x22,
        b: 0x33,
    };
    border
        .show(&OFF_SCREEN, color, 4, 8, window.hwnd)
        .expect("an overlay of our own renders");
    border.hide();
    let elsewhere = Rect::new(OFF_SCREEN.x + 100, OFF_SCREEN.y, 40, 30);

    // Act -- the two calls restack makes, looked at in between.
    border.join_band_of(window.hwnd);
    let shown_before_moving = visible(&border);
    border.place_behind(&elsewhere, 4, window.hwnd);

    // Assert
    assert!(
        !shown_before_moving,
        "the border showed where it was hidden"
    );
    assert!(visible(&border), "the border stayed hidden");
    assert_eq!(
        bounds(&border),
        Rect::new(OFF_SCREEN.x + 96, OFF_SCREEN.y - 4, 48, 38)
    );
}
