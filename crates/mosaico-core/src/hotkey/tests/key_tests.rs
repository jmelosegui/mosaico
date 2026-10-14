use crate::Error;
use crate::hotkey::{Key, digit, function, letter};

/// Every name of a key other than a letter, digit or function key that
/// `keybindings.toml` has always accepted, with the key it stands for.
const NAMES: [(&str, Key); 35] = [
    ("Enter", Key::Enter),
    ("Return", Key::Enter),
    ("Tab", Key::Tab),
    ("Escape", Key::Escape),
    ("Esc", Key::Escape),
    ("Space", Key::Space),
    ("Backspace", Key::Backspace),
    ("Delete", Key::Delete),
    ("Del", Key::Delete),
    ("Insert", Key::Insert),
    ("Ins", Key::Insert),
    ("Home", Key::Home),
    ("End", Key::End),
    ("PageUp", Key::PageUp),
    ("PgUp", Key::PageUp),
    ("PageDown", Key::PageDown),
    ("PgDn", Key::PageDown),
    ("Left", Key::Left),
    ("Up", Key::Up),
    ("Right", Key::Right),
    ("Down", Key::Down),
    ("Minus", Key::Minus),
    ("Plus", Key::Plus),
    ("Equals", Key::Plus),
    ("Comma", Key::Comma),
    ("Period", Key::Period),
    ("Dot", Key::Period),
    ("Slash", Key::Slash),
    ("Semicolon", Key::Semicolon),
    ("Backslash", Key::Backslash),
    ("LBracket", Key::LBracket),
    ("RBracket", Key::RBracket),
    ("Quote", Key::Quote),
    ("Backtick", Key::Backtick),
    ("Grave", Key::Backtick),
];

fn parse(name: &str) -> Option<Key> {
    name.parse().ok()
}

#[test]
fn every_letter_parses_in_either_case() {
    for upper_case in 'A'..='Z' {
        // Act
        let upper = parse(&upper_case.to_string());
        let lower = parse(&upper_case.to_ascii_lowercase().to_string());

        // Assert
        assert_eq!(upper, Some(letter(upper_case)));
        assert_eq!(lower, Some(letter(upper_case)));
    }
}

#[test]
fn every_digit_parses() {
    for number in 0..=9u8 {
        // Act
        let key = parse(&number.to_string());

        // Assert
        assert_eq!(key, Some(digit(number)));
    }
}

#[test]
fn function_keys_go_from_f1_to_f12() {
    for number in 1..=12u8 {
        // Act
        let key = parse(&format!("F{number}"));

        // Assert
        assert_eq!(key, Some(function(number)));
    }
}

#[test]
fn there_is_no_f0_or_f13() {
    // Assert
    assert_eq!(parse("F0"), None);
    assert_eq!(parse("F13"), None);
}

#[test]
fn a_function_key_parses_in_lowercase_and_with_leading_zeros() {
    // Assert
    assert_eq!(parse("f12"), Some(function(12)));
    assert_eq!(parse("F01"), Some(function(1)));
}

#[test]
fn every_other_key_parses_by_each_of_its_names_in_any_case() {
    for (name, key) in NAMES {
        // Act
        let as_written = parse(name);
        let upper = parse(&name.to_ascii_uppercase());
        let lower = parse(&name.to_ascii_lowercase());

        // Assert
        assert_eq!(as_written, Some(key), "{name}");
        assert_eq!(upper, Some(key), "{name}");
        assert_eq!(lower, Some(key), "{name}");
    }
}

#[test]
fn a_name_with_spaces_or_no_name_at_all_is_not_a_key() {
    for name in ["", " ", " J", "J ", "Page Up", "F 1"] {
        // Assert
        assert_eq!(parse(name), None, "{name:?}");
    }
}

#[test]
fn an_unknown_key_name_is_a_config_error_that_names_it() {
    // Act
    let error = "Foo".parse::<Key>().unwrap_err();

    // Assert
    assert!(matches!(error, Error::Config { path: None, .. }));
    assert_eq!(
        error.to_string(),
        "unknown key name \"Foo\" (expected A-Z, 0-9, F1-F12 or a name such as \
         Enter, Space or PageUp)"
    );
}

#[test]
fn keys_are_shown_the_way_bindings_usually_write_them() {
    // Assert
    assert_eq!(letter('j').to_string(), "J");
    assert_eq!(digit(1).to_string(), "1");
    assert_eq!(function(5).to_string(), "F5");
    assert_eq!(Key::Escape.to_string(), "Escape");
    assert_eq!(Key::LBracket.to_string(), "LBracket");
}

#[test]
fn every_key_is_shown_with_a_name_that_parses_back_to_it() {
    // Arrange
    let letters = ('A'..='Z').map(letter);
    let digits = (0..=9).map(digit);
    let function_keys = (1..=12).map(function);
    let others = NAMES.map(|(_, key)| key);
    let keys = letters.chain(digits).chain(function_keys).chain(others);

    for key in keys {
        // Act
        let shown = key.to_string();

        // Assert
        assert_eq!(parse(&shown), Some(key), "{shown}");
    }
}
