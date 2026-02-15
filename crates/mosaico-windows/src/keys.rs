/// Converts a key name string to a Windows virtual key code.
///
/// Supports letters (A–Z), digits (0–9), function keys (F1–F12),
/// and common named keys (Enter, Space, Tab, etc.).
/// Matching is case-insensitive.
pub fn vk_from_name(name: &str) -> Option<u32> {
    let upper = name.to_ascii_uppercase();

    // Single letter A–Z
    if upper.len() == 1 {
        let ch = upper.as_bytes()[0];
        if ch.is_ascii_uppercase() {
            return Some(u32::from(ch));
        }
        if ch.is_ascii_digit() {
            return Some(u32::from(ch));
        }
    }

    // Function keys F1–F12
    if let Some(rest) = upper.strip_prefix('F')
        && let Ok(n) = rest.parse::<u32>()
        && (1..=12).contains(&n)
    {
        return Some(0x70 + n - 1); // VK_F1 = 0x70
    }

    match upper.as_str() {
        // Digits spelled out are not needed — "0"-"9" handled above

        // Navigation
        "ENTER" | "RETURN" => Some(0x0D),
        "TAB" => Some(0x09),
        "ESCAPE" | "ESC" => Some(0x1B),
        "SPACE" => Some(0x20),
        "BACKSPACE" => Some(0x08),
        "DELETE" | "DEL" => Some(0x2E),
        "INSERT" | "INS" => Some(0x2D),
        "HOME" => Some(0x24),
        "END" => Some(0x23),
        "PAGEUP" | "PGUP" => Some(0x21),
        "PAGEDOWN" | "PGDN" => Some(0x22),

        // Arrow keys
        "LEFT" => Some(0x25),
        "UP" => Some(0x26),
        "RIGHT" => Some(0x27),
        "DOWN" => Some(0x28),

        // Punctuation / OEM keys
        "MINUS" => Some(0xBD),
        "PLUS" | "EQUALS" => Some(0xBB),
        "COMMA" => Some(0xBC),
        "PERIOD" | "DOT" => Some(0xBE),
        "SLASH" => Some(0xBF),
        "SEMICOLON" => Some(0xBA),
        "BACKSLASH" => Some(0xDC),
        "LBRACKET" => Some(0xDB),
        "RBRACKET" => Some(0xDD),
        "QUOTE" => Some(0xDE),
        "BACKTICK" | "GRAVE" => Some(0xC0),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letters_case_insensitive() {
        // Assert
        assert_eq!(vk_from_name("j"), Some(0x4A));
        assert_eq!(vk_from_name("J"), Some(0x4A));
        assert_eq!(vk_from_name("a"), Some(0x41));
        assert_eq!(vk_from_name("Z"), Some(0x5A));
    }

    #[test]
    fn digits_return_vk_codes() {
        // Assert
        assert_eq!(vk_from_name("0"), Some(0x30));
        assert_eq!(vk_from_name("9"), Some(0x39));
    }

    #[test]
    fn named_keys() {
        // Assert
        assert_eq!(vk_from_name("Enter"), Some(0x0D));
        assert_eq!(vk_from_name("SPACE"), Some(0x20));
        assert_eq!(vk_from_name("esc"), Some(0x1B));
    }

    #[test]
    fn function_keys() {
        // Assert
        assert_eq!(vk_from_name("F1"), Some(0x70));
        assert_eq!(vk_from_name("f12"), Some(0x7B));
    }

    #[test]
    fn unknown_returns_none() {
        // Assert
        assert_eq!(vk_from_name("INVALID"), None);
        assert_eq!(vk_from_name(""), None);
    }
}
