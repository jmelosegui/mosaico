use mosaico_core::WindowEvent;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    EVENT_OBJECT_CREATE, EVENT_OBJECT_DESTROY, EVENT_OBJECT_HIDE, EVENT_OBJECT_LOCATIONCHANGE,
    EVENT_OBJECT_NAMECHANGE, EVENT_OBJECT_SHOW, EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_MINIMIZEEND,
    EVENT_SYSTEM_MINIMIZESTART, EVENT_SYSTEM_MOVESIZEEND,
};

/// Object ID indicating the event applies to the window itself,
/// not a child element like a scrollbar or menu item.
const OBJID_WINDOW: i32 = 0;

/// Translates a raw Win32 event into a platform-agnostic `WindowEvent`.
///
/// Returns `None` for events we don't care about (e.g. child object
/// events, or event types not relevant to tiling).
///
/// Most events are filtered to `id_object == OBJID_WINDOW` so we only
/// process top-level window events. However, some applications (notably
/// WPF-based apps like Visual Studio) fire creation and name-change
/// events with non-zero `id_object` for their main window. For those
/// event types we skip the filter and let downstream handlers
/// (`is_tileable`) decide whether the window is real.
pub fn translate(event: u32, hwnd: HWND, id_object: i32) -> Option<WindowEvent> {
    let is_window = id_object == OBJID_WINDOW;
    let hwnd_val = hwnd.0 as usize;

    match event {
        // Creation / show — relaxed filter for WPF compatibility.
        e if e == EVENT_OBJECT_SHOW || e == EVENT_OBJECT_CREATE => {
            Some(WindowEvent::Created { hwnd: hwnd_val })
        }
        // Name change — relaxed filter so WPF windows are discovered
        // when they set their title for the first time.
        e if e == EVENT_OBJECT_NAMECHANGE => Some(WindowEvent::TitleChanged { hwnd: hwnd_val }),
        // The remaining events are strict: only top-level windows.
        _ if !is_window => None,
        e if e == EVENT_OBJECT_DESTROY || e == EVENT_OBJECT_HIDE => {
            Some(WindowEvent::Destroyed { hwnd: hwnd_val })
        }
        e if e == EVENT_SYSTEM_FOREGROUND => Some(WindowEvent::Focused { hwnd: hwnd_val }),
        e if e == EVENT_SYSTEM_MOVESIZEEND => Some(WindowEvent::Moved { hwnd: hwnd_val }),
        e if e == EVENT_SYSTEM_MINIMIZESTART => Some(WindowEvent::Minimized { hwnd: hwnd_val }),
        e if e == EVENT_SYSTEM_MINIMIZEEND => Some(WindowEvent::Restored { hwnd: hwnd_val }),
        e if e == EVENT_OBJECT_LOCATIONCHANGE => {
            Some(WindowEvent::LocationChanged { hwnd: hwnd_val })
        }
        _ => None,
    }
}
