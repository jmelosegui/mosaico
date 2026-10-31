use super::*;

#[test]
fn a_click_or_the_keyboard_picks_the_default() {
    // Act
    let mouse = click(0, NIN_SELECT as isize | (1 << 16));
    let keyboard = click(0, NIN_KEYSELECT as isize);

    // Assert
    assert_eq!(mouse, Some(Click::Default));
    assert_eq!(keyboard, Some(Click::Default));
}

#[test]
fn the_menu_opens_where_the_shell_anchors_it_even_left_of_the_first_monitor() {
    // Arrange -- x = -1200, y = 1050 as two signed words.
    let wparam = (usize::from(1050_u16) << 16) | usize::from((-1200_i16) as u16);

    // Act
    let menu = click(wparam, WM_CONTEXTMENU as isize);

    // Assert
    assert_eq!(menu, Some(Click::Menu(-1200, 1050)));
}

#[test]
fn other_mouse_messages_ask_for_nothing() {
    // Act -- WM_MOUSEMOVE, sent while the pointer rests on the icon.
    let moved = click(0, 0x0200);

    // Assert
    assert_eq!(moved, None);
}

#[test]
fn text_that_does_not_fit_is_cut_at_a_whole_character() {
    // Arrange -- the emoji takes two units, and only one is left.
    let mut buffer = [0xFFFF_u16; 4];

    // Act
    copy_wide(&mut buffer, "ab\u{1F600}");

    // Assert
    assert_eq!(buffer, [u16::from(b'a'), u16::from(b'b'), 0, 0xFFFF]);
}

#[test]
fn text_that_fits_is_copied_with_a_nul() {
    // Arrange
    let mut buffer = [0xFFFF_u16; 8];

    // Act
    copy_wide(&mut buffer, "mosaico");

    // Assert
    let expected: Vec<u16> = "mosaico".encode_utf16().chain([0]).collect();
    assert_eq!(buffer.as_slice(), expected.as_slice());
}
