//! The values the hooks and the broadcast window receive in situations
//! met with real applications, and the events they must become.

use mosaico_core::{WindowEvent, WindowId};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    EVENT_OBJECT_CREATE, EVENT_OBJECT_DESTROY, EVENT_OBJECT_FOCUS, EVENT_OBJECT_HIDE,
    EVENT_OBJECT_LOCATIONCHANGE, EVENT_OBJECT_NAMECHANGE, EVENT_OBJECT_REORDER, EVENT_OBJECT_SHOW,
    EVENT_OBJECT_STATECHANGE, EVENT_SYSTEM_CAPTURESTART, EVENT_SYSTEM_FOREGROUND,
    EVENT_SYSTEM_MENUSTART, EVENT_SYSTEM_MINIMIZEEND, EVENT_SYSTEM_MINIMIZESTART,
    EVENT_SYSTEM_MOVESIZEEND, OBJID_CARET, OBJID_CLIENT, OBJID_CURSOR, OBJID_VSCROLL, OBJID_WINDOW,
    SPI_SETDESKWALLPAPER, SPI_SETWORKAREA, WM_DISPLAYCHANGE, WM_SETTINGCHANGE, WM_THEMECHANGED,
};

use super::{translate, translate_broadcast};

/// The handle of a top-level window, as the hooks report it.
const TOP: usize = 0x001A_0B2C;

fn window() -> WindowId {
    WindowId::from_raw(TOP as u64)
}

/// Feeds hook reports, as `(event, window handle, object id)`, through
/// the translation in order.
fn replay(reports: &[(u32, usize, i32)]) -> Vec<WindowEvent> {
    reports
        .iter()
        .filter_map(|&(event, hwnd, object)| translate(event, HWND(hwnd as *mut _), object))
        .collect()
}

#[test]
fn a_hide_while_an_app_recreates_its_surface_is_reported_as_hidden() {
    // Arrange -- GPU-accelerated applications (Zed, browsers) report
    // their top-level window hidden and shown again while they replace
    // the surface they draw on. The window never leaves the screen.
    let reports = [
        (EVENT_OBJECT_HIDE, TOP, OBJID_WINDOW.0),
        (EVENT_OBJECT_SHOW, TOP, OBJID_WINDOW.0),
    ];

    // Act
    let events = replay(&reports);

    // Assert -- a hide, never a destroy: the tiler checks that the
    // window is really invisible before it takes it out of the layout.
    assert_eq!(
        events,
        vec![
            WindowEvent::Hidden { window: window() },
            WindowEvent::Created { window: window() },
        ]
    );
}

#[test]
fn each_report_about_a_window_becomes_its_event() {
    // Arrange
    let w = window();
    let cases = [
        (EVENT_OBJECT_CREATE, WindowEvent::Created { window: w }),
        (EVENT_OBJECT_SHOW, WindowEvent::Created { window: w }),
        (EVENT_OBJECT_DESTROY, WindowEvent::Destroyed { window: w }),
        (EVENT_OBJECT_HIDE, WindowEvent::Hidden { window: w }),
        (EVENT_SYSTEM_FOREGROUND, WindowEvent::Focused { window: w }),
        (EVENT_SYSTEM_MOVESIZEEND, WindowEvent::Moved { window: w }),
        (
            EVENT_SYSTEM_MINIMIZESTART,
            WindowEvent::Minimized { window: w },
        ),
        (
            EVENT_SYSTEM_MINIMIZEEND,
            WindowEvent::Restored { window: w },
        ),
        (
            EVENT_OBJECT_NAMECHANGE,
            WindowEvent::TitleChanged { window: w },
        ),
        (
            EVENT_OBJECT_LOCATIONCHANGE,
            WindowEvent::LocationChanged { window: w },
        ),
    ];

    for (event, expected) in cases {
        // Act
        let translated = replay(&[(event, TOP, OBJID_WINDOW.0)]);

        // Assert
        assert_eq!(translated, vec![expected], "event 0x{event:04X}");
    }
}

#[test]
fn creation_and_title_reports_on_an_inner_object_are_kept() {
    // Arrange -- WPF windows (Visual Studio) report being created, shown
    // and titled on an object inside the window, never on the window
    // itself, so a strict filter would never find them.
    let reports = [
        (EVENT_OBJECT_CREATE, TOP, OBJID_CLIENT.0),
        (EVENT_OBJECT_SHOW, TOP, OBJID_CLIENT.0),
        (EVENT_OBJECT_NAMECHANGE, TOP, OBJID_CLIENT.0),
    ];

    // Act
    let events = replay(&reports);

    // Assert
    assert_eq!(
        events,
        vec![
            WindowEvent::Created { window: window() },
            WindowEvent::Created { window: window() },
            WindowEvent::TitleChanged { window: window() },
        ]
    );
}

#[test]
fn other_reports_on_an_inner_object_are_dropped() {
    // Arrange -- client areas, scroll bars and carets report all the
    // time; for these events only the window itself matters.
    let events = [
        EVENT_OBJECT_DESTROY,
        EVENT_OBJECT_HIDE,
        EVENT_SYSTEM_FOREGROUND,
        EVENT_SYSTEM_MOVESIZEEND,
        EVENT_SYSTEM_MINIMIZESTART,
        EVENT_SYSTEM_MINIMIZEEND,
        EVENT_OBJECT_LOCATIONCHANGE,
    ];

    for object in [OBJID_CLIENT.0, OBJID_VSCROLL.0, OBJID_CARET.0] {
        for event in events {
            // Act
            let translated = replay(&[(event, TOP, object)]);

            // Assert
            assert_eq!(translated, vec![], "event 0x{event:04X} on {object}");
        }
    }
}

#[test]
fn reports_about_the_mouse_pointer_are_dropped() {
    // Arrange -- the pointer reports being shown, and changing shape, on
    // its own object and with no window. Creation and title reports on
    // inner objects are kept, but these name no window at all.
    let reports = [
        (EVENT_OBJECT_SHOW, 0, OBJID_CURSOR.0),
        (EVENT_OBJECT_NAMECHANGE, 0, OBJID_CURSOR.0),
    ];

    // Act
    let events = replay(&reports);

    // Assert
    assert_eq!(events, vec![]);
}

#[test]
fn reports_tiling_does_not_use_are_dropped() {
    // Arrange -- the hooks cover two whole ranges of event codes.
    let unused = [
        EVENT_SYSTEM_CAPTURESTART,
        EVENT_SYSTEM_MENUSTART,
        EVENT_OBJECT_REORDER,
        EVENT_OBJECT_FOCUS,
        EVENT_OBJECT_STATECHANGE,
    ];

    for event in unused {
        // Act
        let translated = replay(&[(event, TOP, OBJID_WINDOW.0)]);

        // Assert
        assert_eq!(translated, vec![], "event 0x{event:04X}");
    }
}

#[test]
fn a_display_change_is_reported() {
    // Act
    let event = translate_broadcast(WM_DISPLAYCHANGE, 0);

    // Assert
    assert_eq!(event, Some(WindowEvent::DisplayChanged));
}

#[test]
fn of_all_setting_changes_only_the_work_area_is_reported() {
    // Arrange -- a setting change names the setting in its wParam.
    let work_area = SPI_SETWORKAREA.0 as usize;
    let wallpaper = SPI_SETDESKWALLPAPER.0 as usize;

    // Act
    let work_area = translate_broadcast(WM_SETTINGCHANGE, work_area);
    let wallpaper = translate_broadcast(WM_SETTINGCHANGE, wallpaper);
    let policy = translate_broadcast(WM_SETTINGCHANGE, 0);

    // Assert
    assert_eq!(work_area, Some(WindowEvent::WorkAreaChanged));
    assert_eq!(wallpaper, None);
    assert_eq!(policy, None);
}

#[test]
fn other_broadcasts_are_dropped() {
    // Act
    let theme = translate_broadcast(WM_THEMECHANGED, 0);
    let lookalike = translate_broadcast(WM_THEMECHANGED, SPI_SETWORKAREA.0 as usize);

    // Assert
    assert_eq!(theme, None);
    assert_eq!(lookalike, None);
}
