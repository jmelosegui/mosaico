use eframe::egui::{Event, Key, Modifiers};
use mosaico_core::config::Modifier;

use super::*;

fn press(key: Key, physical: Option<Key>, modifiers: Modifiers) -> Event {
    Event::Key {
        key,
        physical_key: physical,
        pressed: true,
        repeat: false,
        modifiers,
    }
}

#[test]
fn a_key_with_modifiers_is_a_combination() {
    let events = [press(Key::J, None, Modifiers::ALT | Modifiers::SHIFT)];

    assert_eq!(
        pressed(&events),
        Some(Pressed::Combo {
            key: "J".into(),
            modifiers: vec![Modifier::Alt, Modifier::Shift],
        })
    );
}

#[test]
fn the_physical_key_wins_so_shift_and_a_digit_stay_that_digit() {
    let events = [press(Key::Exclamationmark, Some(Key::Num1), Modifiers::ALT)];

    let Some(Pressed::Combo { key, .. }) = pressed(&events) else {
        panic!("not a combination");
    };

    assert_eq!(key, "1");
}

#[test]
fn escape_alone_cancels_and_a_bare_letter_is_refused() {
    assert_eq!(
        pressed(&[press(Key::Escape, None, Modifiers::NONE)]),
        Some(Pressed::Cancel)
    );
    assert!(matches!(
        pressed(&[press(Key::A, None, Modifiers::NONE)]),
        Some(Pressed::Refused(_))
    ));
}

#[test]
fn a_function_key_needs_no_modifier_and_brackets_take_the_file_names() {
    assert!(matches!(
        pressed(&[press(Key::F5, None, Modifiers::NONE)]),
        Some(Pressed::Combo { .. })
    ));
    let Some(Pressed::Combo { key, .. }) =
        pressed(&[press(Key::OpenBracket, None, Modifiers::CTRL)])
    else {
        panic!("not a combination");
    };
    assert_eq!(key, "LBracket");
}

#[test]
fn releases_and_repeats_are_ignored() {
    let release = Event::Key {
        key: Key::J,
        physical_key: None,
        pressed: false,
        repeat: false,
        modifiers: Modifiers::ALT,
    };

    assert_eq!(pressed(&[release]), None);
}
