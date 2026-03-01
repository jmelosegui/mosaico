use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Accessibility::HWINEVENTHOOK;

use crate::event;

use super::EVENT_SENDER;

/// The WinEvent callback.
pub(crate) unsafe extern "system" fn win_event_proc(
    _hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    id_object: i32,
    _id_child: i32,
    _event_thread: u32,
    _event_time: u32,
) {
    if let Some(window_event) = event::translate(event, hwnd, id_object) {
        EVENT_SENDER.with(|cell| {
            if let Some(sender) = cell.borrow().as_ref() {
                let _ = sender.send(window_event);
            }
        });
    }
}
