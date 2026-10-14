use mosaico_core::hotkey::Key;

use super::virtual_key;

/// The code a key name in `keybindings.toml` registers.
fn code_of(name: &str) -> Option<u32> {
    name.parse::<Key>().ok().map(virtual_key)
}

#[test]
fn letters_in_either_case_and_digits_register_their_ascii_codes() {
    // Assert
    assert_eq!(code_of("j"), Some(0x4A));
    assert_eq!(code_of("J"), Some(0x4A));
    assert_eq!(code_of("a"), Some(0x41));
    assert_eq!(code_of("Z"), Some(0x5A));
    assert_eq!(code_of("0"), Some(0x30));
    assert_eq!(code_of("9"), Some(0x39));
}

#[test]
fn function_keys_register_vk_f1_to_vk_f24() {
    // Assert
    assert_eq!(code_of("F1"), Some(0x70));
    assert_eq!(code_of("f12"), Some(0x7B));
    assert_eq!(code_of("F13"), Some(0x7C));
    assert_eq!(code_of("F24"), Some(0x87));
}

#[test]
fn every_other_key_name_registers_the_code_it_always_has() {
    // Arrange
    let codes = [
        ("Enter", 0x0D),
        ("Return", 0x0D),
        ("Tab", 0x09),
        ("Escape", 0x1B),
        ("Esc", 0x1B),
        ("Space", 0x20),
        ("Backspace", 0x08),
        ("Delete", 0x2E),
        ("Del", 0x2E),
        ("Insert", 0x2D),
        ("Ins", 0x2D),
        ("Home", 0x24),
        ("End", 0x23),
        ("PageUp", 0x21),
        ("PgUp", 0x21),
        ("PageDown", 0x22),
        ("PgDn", 0x22),
        ("Left", 0x25),
        ("Up", 0x26),
        ("Right", 0x27),
        ("Down", 0x28),
        ("Minus", 0xBD),
        ("Plus", 0xBB),
        ("Equals", 0xBB),
        ("Comma", 0xBC),
        ("Period", 0xBE),
        ("Dot", 0xBE),
        ("Slash", 0xBF),
        ("Semicolon", 0xBA),
        ("Backslash", 0xDC),
        ("LBracket", 0xDB),
        ("RBracket", 0xDD),
        ("Quote", 0xDE),
        ("Backtick", 0xC0),
        ("Grave", 0xC0),
    ];

    for (name, code) in codes {
        // Act
        let registered = code_of(name);

        // Assert
        assert_eq!(registered, Some(code), "{name}");
    }
}
