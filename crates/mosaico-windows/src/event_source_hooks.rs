//! The two WinEvent hooks and the callback they share.

use mosaico_core::{Error, Result};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent};
use windows::Win32::UI::WindowsAndMessaging::{WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS};

use crate::event;

/// System event range: FOREGROUND (0x0003) through MINIMIZEEND (0x0017).
///
/// Covers: EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_MOVESIZESTART,
/// EVENT_SYSTEM_MOVESIZEEND, EVENT_SYSTEM_MINIMIZESTART,
/// EVENT_SYSTEM_MINIMIZEEND.
const SYSTEM_EVENT_MIN: u32 = 0x0003;
const SYSTEM_EVENT_MAX: u32 = 0x0017;

/// Object event range: CREATE (0x8000) through NAMECHANGE (0x800C).
///
/// Covers: EVENT_OBJECT_CREATE, EVENT_OBJECT_DESTROY, EVENT_OBJECT_SHOW,
/// EVENT_OBJECT_HIDE, EVENT_OBJECT_LOCATIONCHANGE, EVENT_OBJECT_NAMECHANGE.
const OBJECT_EVENT_MIN: u32 = 0x8000;
const OBJECT_EVENT_MAX: u32 = 0x800C;

/// The hooks of the event thread. Dropping them unhooks.
pub(super) struct Hooks {
    system: HWINEVENTHOOK,
    object: HWINEVENTHOOK,
}

impl Hooks {
    /// Installs both hooks on the calling thread, whose message pump
    /// then runs the callback.
    pub(super) fn install() -> Result<Self> {
        let flags = WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS;

        // Use two targeted hooks instead of one broad range (0x0003 to
        // 0x800C) to avoid receiving thousands of irrelevant events in the
        // gap (0x0018 to 0x7FFF) that translate() would just discard.
        // Under heavy system load this noise can starve the message pump.
        // SAFETY: SetWinEventHook installs out-of-process event hooks
        // for system and object events. The callback function pointer is
        // valid for the lifetime of the thread. Hooks are unhooked on drop.
        let system = unsafe {
            SetWinEventHook(
                SYSTEM_EVENT_MIN,
                SYSTEM_EVENT_MAX,
                None,
                Some(win_event_proc),
                0,
                0,
                flags,
            )
        };
        // SAFETY: Second SetWinEventHook for object events (same safety as above).
        let object = unsafe {
            SetWinEventHook(
                OBJECT_EVENT_MIN,
                OBJECT_EVENT_MAX,
                None,
                Some(win_event_proc),
                0,
                0,
                flags,
            )
        };

        // Dropping removes the hook that worked when the other did not.
        let hooks = Self { system, object };
        if system.is_invalid() || object.is_invalid() {
            let reason = "SetWinEventHook failed";
            return Err(Error::platform("set the window event hooks", reason));
        }
        Ok(hooks)
    }
}

impl Drop for Hooks {
    fn drop(&mut self) {
        for hook in [self.system, self.object] {
            if !hook.is_invalid() {
                // SAFETY: UnhookWinEvent removes a hook that install set
                // on this thread; Hooks cannot leave it.
                unsafe {
                    let _ = UnhookWinEvent(hook);
                }
            }
        }
    }
}

/// The WinEvent callback.
unsafe extern "system" fn win_event_proc(
    _hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    id_object: i32,
    _id_child: i32,
    _event_thread: u32,
    _event_time: u32,
) {
    if let Some(window_event) = event::translate(event, hwnd, id_object) {
        super::send(crate::drag_facts::read(window_event, hwnd));
    }
}
