//! Win32 virtual-key codes for the keys a hotkey can be bound to.

use mosaico_core::hotkey::Key;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    VIRTUAL_KEY, VK_BACK, VK_DELETE, VK_DOWN, VK_END, VK_ESCAPE, VK_F1, VK_HOME, VK_INSERT,
    VK_LEFT, VK_NEXT, VK_OEM_1, VK_OEM_2, VK_OEM_3, VK_OEM_4, VK_OEM_5, VK_OEM_6, VK_OEM_7,
    VK_OEM_COMMA, VK_OEM_MINUS, VK_OEM_PERIOD, VK_OEM_PLUS, VK_PRIOR, VK_RETURN, VK_RIGHT,
    VK_SPACE, VK_TAB, VK_UP,
};

/// The virtual-key code `RegisterHotKey` takes for `key`.
///
/// Letters and digits are their own ASCII codes, and the function keys
/// are `VK_F1` onwards, which follow each other up to `VK_F24`. Windows
/// defines `VK_OEM_MINUS`, `VK_OEM_PLUS`, `VK_OEM_COMMA` and
/// `VK_OEM_PERIOD` as the keys that type those symbols on any layout; the
/// other punctuation codes stand for where a key is on a US keyboard, so
/// they type something else on other layouts.
pub(crate) fn virtual_key(key: Key) -> u32 {
    let code: VIRTUAL_KEY = match key {
        Key::Letter(letter) => return u32::from(letter.get()),
        Key::Digit(digit) => return u32::from(b'0' + digit.get()),
        Key::Function(function) => {
            return u32::from(VK_F1.0) + u32::from(function.get() - 1);
        }
        Key::Enter => VK_RETURN,
        Key::Tab => VK_TAB,
        Key::Escape => VK_ESCAPE,
        Key::Space => VK_SPACE,
        Key::Backspace => VK_BACK,
        Key::Delete => VK_DELETE,
        Key::Insert => VK_INSERT,
        Key::Home => VK_HOME,
        Key::End => VK_END,
        Key::PageUp => VK_PRIOR,
        Key::PageDown => VK_NEXT,
        Key::Left => VK_LEFT,
        Key::Up => VK_UP,
        Key::Right => VK_RIGHT,
        Key::Down => VK_DOWN,
        Key::Minus => VK_OEM_MINUS,
        Key::Plus => VK_OEM_PLUS,
        Key::Comma => VK_OEM_COMMA,
        Key::Period => VK_OEM_PERIOD,
        Key::Slash => VK_OEM_2,
        Key::Semicolon => VK_OEM_1,
        Key::Backslash => VK_OEM_5,
        Key::LBracket => VK_OEM_4,
        Key::RBracket => VK_OEM_6,
        Key::Quote => VK_OEM_7,
        Key::Backtick => VK_OEM_3,
    };
    u32::from(code.0)
}

#[cfg(test)]
#[path = "keys_tests.rs"]
mod tests;
