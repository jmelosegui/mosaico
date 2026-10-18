use windows::Win32::UI::WindowsAndMessaging::{
    HTBORDER, HTCLIENT, HTCLOSE, HTMAXBUTTON, HTMINBUTTON, HTNOWHERE, HTSYSMENU,
};

use super::*;

/// Edges, as `left, top, right, bottom`.
pub(super) fn edges(left: bool, top: bool, right: bool, bottom: bool) -> Option<Grab> {
    Some(Grab::Edges(Edges {
        left,
        top,
        right,
        bottom,
    }))
}

#[test]
fn the_caption_and_each_sizing_edge_and_corner_say_what_is_grabbed() {
    // Arrange
    let cases = [
        (HTCAPTION, Some(Grab::TitleBar)),
        (HTLEFT, edges(true, false, false, false)),
        (HTRIGHT, edges(false, false, true, false)),
        (HTTOP, edges(false, true, false, false)),
        (HTBOTTOM, edges(false, false, false, true)),
        (HTTOPLEFT, edges(true, true, false, false)),
        (HTTOPRIGHT, edges(false, true, true, false)),
        (HTBOTTOMLEFT, edges(true, false, false, true)),
        (HTBOTTOMRIGHT, edges(false, false, true, true)),
    ];

    for (code, expected) in cases {
        // Act
        let grab = grab_of(code);

        // Assert
        assert_eq!(grab, expected, "hit test code {code}");
    }
}

#[test]
fn any_other_part_of_a_window_grabs_nothing_the_tiler_can_tell() {
    // Arrange -- the client area, as for a drag the application starts
    // itself, the buttons, a border that does not size, nothing, and the
    // error code -2 a window may answer with.
    let codes = [
        HTCLIENT,
        HTNOWHERE,
        HTSYSMENU,
        HTMINBUTTON,
        HTMAXBUTTON,
        HTCLOSE,
        HTBORDER,
        -2i32 as u32,
    ];

    for code in codes {
        // Act
        let grab = grab_of(code);

        // Assert
        assert_eq!(grab, None, "hit test code {code}");
    }
}

#[test]
fn a_point_is_packed_as_a_mouse_message_carries_it() {
    // Arrange -- on a monitor left of and above the primary one.
    let (x, y) = (-1920, -5);

    // Act
    let LPARAM(packed) = point_param(x, y);

    // Assert -- each word read back as a signed number is the coordinate.
    assert_eq!(packed as u16 as i16, -1920);
    assert_eq!((packed >> 16) as u16 as i16, -5);
    assert_eq!(point_param(800, 600).0, (600 << 16) | 800);
}
