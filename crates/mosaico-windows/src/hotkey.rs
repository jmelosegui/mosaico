use std::sync::mpsc::Sender;

use mosaico_core::Action;
use mosaico_core::config::{Keybinding, Modifier};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT, MOD_WIN, RegisterHotKey,
    UnregisterHotKey,
};

use crate::keys;

/// A registered global hotkey.
struct Hotkey {
    id: i32,
    modifiers: HOT_KEY_MODIFIERS,
    vk: u32,
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
    paused: bool,
    pause_hotkey_id: Option<i32>,
}

impl HotkeyManager {
    /// Creates a new hotkey manager.
    ///
    /// Actions triggered by hotkeys are sent through `sender`.
    pub fn new(sender: Sender<Action>) -> Self {
        Self {
            hotkeys: Vec::new(),
            sender,
            paused: false,
            pause_hotkey_id: None,
        }
    }

    /// Registers keybindings from configuration.
    ///
    /// Each keybinding's key name is resolved to a virtual key code
    /// and its modifiers are converted to Win32 flags. Invalid key
    /// names are logged and skipped.
    pub fn register_from_config(&mut self, bindings: &[Keybinding]) {
        for (i, binding) in bindings.iter().enumerate() {
            let id = (i + 1) as i32;

            let Some(vk) = keys::vk_from_name(&binding.key) else {
                eprintln!("Unknown key name: {:?}", binding.key);
                continue;
            };

            let mut modifiers = MOD_NOREPEAT;
            for m in &binding.modifiers {
                modifiers |= modifier_to_flag(m);
            }

            self.register(id, modifiers, vk, binding.action);
        }

        self.pause_hotkey_id = self.hotkeys
            .iter()
            .find(|h| h.action == Action::TogglePause)
            .map(|h| h.id);
    }

    /// Dispatches a `WM_HOTKEY` message by hotkey ID.
    ///
    /// Called from the message pump when a `WM_HOTKEY` message arrives.
    pub fn dispatch(&self, hotkey_id: i32) {
        if let Some(hotkey) = self.hotkeys.iter().find(|h| h.id == hotkey_id) {
            let _ = self.sender.send(hotkey.action);
        }
    }

    /// Registers a single hotkey.
    fn register(&mut self, id: i32, modifiers: HOT_KEY_MODIFIERS, vk: u32, action: Action) {
        // SAFETY: RegisterHotKey registers a system-wide hotkey on the
        // current thread's message queue. We use unique IDs to avoid
        // collisions.
        let result = unsafe { RegisterHotKey(None, id, modifiers, vk) };

        if result.is_err() {
            eprintln!("Failed to register hotkey {id} (vk=0x{vk:02X})");
            return;
        }

        self.hotkeys.push(Hotkey { id, modifiers, vk, action });
    }

    /// Returns true if hotkeys are currently paused.
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Unregisters all hotkeys except the toggle-pause one (if configured).
    ///
    /// No-op if already paused. When no `toggle-pause` keybinding is configured,
    /// all hotkeys are unregistered — the user must unpause via the CLI.
    pub fn pause(&mut self) {
        if self.paused {
            return;
        }
        self.paused = true;
        for hotkey in &self.hotkeys {
            if Some(hotkey.id) == self.pause_hotkey_id {
                continue;
            }
            // SAFETY: UnregisterHotKey removes a previously registered hotkey.
            // Failures are ignored — the OS will silently release it on process exit.
            unsafe {
                let _ = UnregisterHotKey(None, hotkey.id);
            }
        }
    }

    /// Re-registers all hotkeys that were unregistered by `pause()`.
    ///
    /// No-op if not paused.
    pub fn unpause(&mut self) {
        if !self.paused {
            return;
        }
        self.paused = false;
        for hotkey in &self.hotkeys {
            if Some(hotkey.id) == self.pause_hotkey_id {
                continue;
            }
            // SAFETY: RegisterHotKey registers a system-wide hotkey on the
            // current thread's message queue using the original id, modifiers, vk.
            let result = unsafe { RegisterHotKey(None, hotkey.id, hotkey.modifiers, hotkey.vk) };
            if result.is_err() {
                eprintln!("Failed to re-register hotkey {} (vk=0x{:02X})", hotkey.id, hotkey.vk);
            }
        }
    }

    /// Toggles pause state: pauses if unpaused, unpauses if paused.
    pub fn toggle_pause(&mut self) {
        if self.paused {
            self.unpause();
        } else {
            self.pause();
        }
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

/// Converts a platform-agnostic modifier to a Win32 hotkey flag.
fn modifier_to_flag(modifier: &Modifier) -> HOT_KEY_MODIFIERS {
    match modifier {
        Modifier::Alt => MOD_ALT,
        Modifier::Shift => MOD_SHIFT,
        Modifier::Ctrl => MOD_CONTROL,
        Modifier::Win => MOD_WIN,
    }
}
