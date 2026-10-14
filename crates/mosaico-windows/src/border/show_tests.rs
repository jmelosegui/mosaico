use windows::Win32::UI::WindowsAndMessaging::IsWindowVisible;

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
fn a_hidden_border_put_back_with_its_bitmap_is_shown_again() {
    // Arrange -- a second overlay of our own stands for the window the
    // border surrounds. Without a desktop to make them on, skip.
    let (Ok(border), Ok(window)) = (Border::new(), Border::new()) else {
        return;
    };
    let color = Color {
        r: 0x11,
        g: 0x22,
        b: 0x33,
    };
    border.show(&OFF_SCREEN, color, 4, 8, window.hwnd);
    border.hide();
    let hidden = !visible(&border);

    // Act -- drawn again the same way, which keeps the bitmap.
    border.restack(&OFF_SCREEN, 4, window.hwnd);

    // Assert
    assert!(hidden, "hiding did not hide the border");
    assert!(visible(&border), "the border stayed hidden");
}
