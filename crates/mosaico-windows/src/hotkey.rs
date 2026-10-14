//! Global hotkeys registered with `RegisterHotKey`.

use std::marker::PhantomData;

use mosaico_core::Result;
use mosaico_core::hotkey::{HotkeyId, HotkeyRegistrar, KeyCombo, Modifiers};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT, MOD_WIN, RegisterHotKey,
    UnregisterHotKey,
};

use crate::error::OrPlatform;
use crate::keys;

/// Registers hotkeys on the message queue of the thread that made it.
///
/// `RegisterHotKey` without a window posts `WM_HOTKEY` to the thread that
/// registered the key, with the id it was registered under in `wParam`,
/// and only that thread can unregister it. So the registrar is not
/// `Send`: it stays on the thread that pumps those messages. Every hotkey
/// is registered with `MOD_NOREPEAT`, so holding the keys down is one
/// press.
pub(crate) struct Win32Hotkeys {
    _thread_bound: PhantomData<*const ()>,
}

impl Win32Hotkeys {
    pub(crate) fn new() -> Self {
        Self {
            _thread_bound: PhantomData,
        }
    }
}

impl HotkeyRegistrar for Win32Hotkeys {
    fn register(&mut self, id: HotkeyId, combo: KeyCombo) -> Result<()> {
        let vk = keys::virtual_key(combo.key);
        let modifiers = hotkey_modifiers(combo.modifiers) | MOD_NOREPEAT;
        // SAFETY: with no window, RegisterHotKey registers the hotkey on
        // this thread's message queue. It reads nothing but its arguments.
        unsafe { RegisterHotKey(None, i32::from(id.raw()), modifiers, vk) }
            .or_platform("register a hotkey")
    }

    fn unregister(&mut self, id: HotkeyId) -> Result<()> {
        // SAFETY: with no window, UnregisterHotKey removes a hotkey that
        // this thread registered. It reads nothing but its arguments.
        unsafe { UnregisterHotKey(None, i32::from(id.raw())) }.or_platform("unregister a hotkey")
    }
}

/// The `RegisterHotKey` flags for `modifiers`.
fn hotkey_modifiers(modifiers: Modifiers) -> HOT_KEY_MODIFIERS {
    let flags = [
        (Modifiers::CTRL, MOD_CONTROL),
        (Modifiers::ALT, MOD_ALT),
        (Modifiers::SHIFT, MOD_SHIFT),
        (Modifiers::WIN, MOD_WIN),
    ];
    flags
        .into_iter()
        .filter(|(modifier, _)| modifiers.contains(*modifier))
        .fold(HOT_KEY_MODIFIERS::default(), |all, (_, flag)| all | flag)
}

/// The hotkey a `WM_HOTKEY` message is about, from its `wParam`.
///
/// The system's own hotkeys report negative ids (`IDHOT_SNAPDESKTOP` and
/// `IDHOT_SNAPWINDOW`), which are none of ours.
pub(crate) fn hotkey_id(wparam: usize) -> Option<HotkeyId> {
    u16::try_from(wparam).ok().map(HotkeyId::from_raw)
}

#[cfg(test)]
mod tests {
    use mosaico_core::hotkey::{HotkeyId, Modifiers};
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_SHIFT, MOD_WIN,
    };

    use super::{hotkey_id, hotkey_modifiers};

    #[test]
    fn each_modifier_registers_its_own_flag() {
        // Assert
        assert_eq!(hotkey_modifiers(Modifiers::CTRL), MOD_CONTROL);
        assert_eq!(hotkey_modifiers(Modifiers::ALT), MOD_ALT);
        assert_eq!(hotkey_modifiers(Modifiers::SHIFT), MOD_SHIFT);
        assert_eq!(hotkey_modifiers(Modifiers::WIN), MOD_WIN);
        assert_eq!(hotkey_modifiers(Modifiers::NONE), HOT_KEY_MODIFIERS(0));
    }

    #[test]
    fn modifiers_held_together_register_their_flags_together() {
        // Act
        let flags = hotkey_modifiers(Modifiers::ALT | Modifiers::SHIFT);

        // Assert
        assert_eq!(flags, MOD_ALT | MOD_SHIFT);
    }

    #[test]
    fn a_press_of_a_system_hotkey_is_none_of_ours() {
        // Arrange -- IDHOT_SNAPWINDOW is -1 and IDHOT_SNAPDESKTOP is -2.
        let snap_window = usize::MAX;
        let snap_desktop = usize::MAX - 1;

        // Assert
        assert_eq!(hotkey_id(snap_window), None);
        assert_eq!(hotkey_id(snap_desktop), None);
        assert_eq!(hotkey_id(7), Some(HotkeyId::from_raw(7)));
    }
}
