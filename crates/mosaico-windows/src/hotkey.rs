use std::sync::mpsc::Sender;

use mosaico_core::Action;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT, RegisterHotKey, UnregisterHotKey,
};

/// A registered global hotkey.
struct Hotkey {
    id: i32,
    action: Action,
}

/// Manages global hotkey registrations.
///
/// Hotkeys are registered on the current thread's message queue.
/// `WM_HOTKEY` messages arrive via the Win32 message pump running
/// on the same thread.
pub struct HotkeyManager {
    hotkeys: Vec<Hotkey>,
    sender: Sender<Action>,
}

impl HotkeyManager {
    /// Creates a new hotkey manager.
    ///
    /// Actions triggered by hotkeys are sent through `sender`.
    pub fn new(sender: Sender<Action>) -> Self {
        Self {
            hotkeys: Vec::new(),
            sender,
        }
    }

    /// Registers default keybindings.
    ///
    /// Default bindings use Alt+Shift as the modifier prefix:
    /// - Alt+Shift+J: Focus next
    /// - Alt+Shift+K: Focus previous
    /// - Alt+Shift+Enter: Swap next
    /// - Alt+Shift+Shift+Enter: Swap previous (Alt+Ctrl+Enter)
    /// - Alt+Shift+R: Retile
    pub fn register_defaults(&mut self) {
        let alt_shift = MOD_ALT | MOD_SHIFT | MOD_NOREPEAT;
        let alt_ctrl = MOD_ALT | MOD_CONTROL | MOD_NOREPEAT;

        self.register(1, alt_shift, 0x4A, Action::FocusNext); // J
        self.register(2, alt_shift, 0x4B, Action::FocusPrev); // K
        self.register(3, alt_shift, 0x0D, Action::SwapNext); // Enter
        self.register(4, alt_ctrl, 0x0D, Action::SwapPrev); // Enter
        self.register(5, alt_shift, 0x52, Action::Retile); // R
    }

    /// Returns a reference to the action sender.
    pub fn sender(&self) -> &Sender<Action> {
        &self.sender
    }

    /// Dispatches a `WM_HOTKEY` message by hotkey ID.
    ///
    /// Called from the message pump when a `WM_HOTKEY` message arrives.
    pub fn dispatch(&self, hotkey_id: i32) {
        if let Some(hotkey) = self.hotkeys.iter().find(|h| h.id == hotkey_id) {
            let _ = self.sender.send(hotkey.action.clone());
        }
    }

    /// Registers a single hotkey.
    fn register(
        &mut self,
        id: i32,
        modifiers: windows::Win32::UI::Input::KeyboardAndMouse::HOT_KEY_MODIFIERS,
        vk: u32,
        action: Action,
    ) {
        // SAFETY: RegisterHotKey registers a system-wide hotkey on the
        // current thread's message queue. We use unique IDs to avoid
        // collisions.
        let result = unsafe { RegisterHotKey(None, id, modifiers, vk) };

        if result.is_err() {
            eprintln!("Failed to register hotkey {id} (vk=0x{vk:02X})");
            return;
        }

        self.hotkeys.push(Hotkey { id, action });
    }
}

impl Drop for HotkeyManager {
    fn drop(&mut self) {
        for hotkey in &self.hotkeys {
            // SAFETY: UnregisterHotKey removes the hotkey registration.
            unsafe {
                let _ = UnregisterHotKey(None, hotkey.id);
            }
        }
    }
}
