use mosaico_core::WindowEvent;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    EVENT_OBJECT_CREATE, EVENT_OBJECT_DESTROY, EVENT_OBJECT_HIDE, EVENT_OBJECT_LOCATIONCHANGE,
    EVENT_OBJECT_NAMECHANGE, EVENT_OBJECT_SHOW, EVENT_SYSTEM_FOREGROUND,
    EVENT_SYSTEM_MINIMIZEEND, EVENT_SYSTEM_MINIMIZESTART, EVENT_SYSTEM_MOVESIZEEND,
};

/// Object ID indicating the event applies to the window itself,
/// not a child element like a scrollbar or menu item.
const OBJID_WINDOW: i32 = 0;

/// Translates a raw Win32 event into a platform-agnostic `WindowEvent`.
///
/// Returns `None` for events we don't care about (e.g. child object
/// events, or event types not relevant to tiling).
pub fn translate(event: u32, hwnd: HWND, id_object: i32) -> Option<WindowEvent> {
    // Ignore events on child objects (scrollbars, buttons, etc.).
    // We only care about top-level window events.
    if id_object != OBJID_WINDOW {
        return None;
    }

    let hwnd_val = hwnd.0 as usize;

    match event {
        e if e == EVENT_OBJECT_SHOW || e == EVENT_OBJECT_CREATE => {
            Some(WindowEvent::Created { hwnd: hwnd_val })
        }
        e if e == EVENT_OBJECT_DESTROY || e == EVENT_OBJECT_HIDE => {
            Some(WindowEvent::Destroyed { hwnd: hwnd_val })
        }
        e if e == EVENT_SYSTEM_FOREGROUND => Some(WindowEvent::Focused { hwnd: hwnd_val }),
        e if e == EVENT_SYSTEM_MOVESIZEEND => Some(WindowEvent::Moved { hwnd: hwnd_val }),
        e if e == EVENT_SYSTEM_MINIMIZESTART => Some(WindowEvent::Minimized { hwnd: hwnd_val }),
        e if e == EVENT_SYSTEM_MINIMIZEEND => Some(WindowEvent::Restored { hwnd: hwnd_val }),
        e if e == EVENT_OBJECT_NAMECHANGE => Some(WindowEvent::TitleChanged { hwnd: hwnd_val }),
        e if e == EVENT_OBJECT_LOCATIONCHANGE => {
            Some(WindowEvent::LocationChanged { hwnd: hwnd_val })
        }
        _ => None,
    }
}
