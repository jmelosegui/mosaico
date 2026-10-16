use crate::config::Modifier;
use crate::hotkey::{KeyCombo, Modifiers, function, letter};

#[test]
fn a_modifier_list_becomes_the_set_of_its_modifiers() {
    // Act
    let set: Modifiers = [Modifier::Alt, Modifier::Shift].into_iter().collect();

    // Assert
    assert_eq!(set, Modifiers::ALT | Modifiers::SHIFT);
    assert!(set.contains(Modifiers::ALT));
    assert!(set.contains(Modifiers::SHIFT));
    assert!(!set.contains(Modifiers::CTRL));
    assert!(!set.contains(Modifiers::WIN));
}

#[test]
fn a_modifier_listed_twice_counts_once() {
    // Act
    let set: Modifiers = [Modifier::Alt, Modifier::Alt].into_iter().collect();

    // Assert
    assert_eq!(set, Modifiers::ALT);
}

#[test]
fn an_empty_modifier_list_is_no_modifier() {
    // Act
    let set: Modifiers = std::iter::empty::<Modifier>().collect();

    // Assert
    assert_eq!(set, Modifiers::NONE);
    assert!(set.is_empty());
}

#[test]
fn each_modifier_has_a_flag_of_its_own() {
    // Act
    let flags = [
        Modifier::Ctrl,
        Modifier::Alt,
        Modifier::Shift,
        Modifier::Win,
    ]
    .map(Modifiers::from);

    // Assert
    assert_eq!(
        flags,
        [
            Modifiers::CTRL,
            Modifiers::ALT,
            Modifiers::SHIFT,
            Modifiers::WIN
        ]
    );
    for (i, flag) in flags.iter().enumerate() {
        for (j, other) in flags.iter().enumerate() {
            assert_eq!(flag.contains(*other), i == j);
        }
    }
}

#[test]
fn a_combo_is_shown_with_its_modifiers_in_the_usual_order() {
    // Arrange
    let modifiers = [
        Modifier::Win,
        Modifier::Shift,
        Modifier::Ctrl,
        Modifier::Alt,
    ];
    let combo = KeyCombo {
        modifiers: modifiers.into_iter().collect(),
        key: letter('J'),
    };

    // Act
    let shown = combo.to_string();

    // Assert
    assert_eq!(shown, "Ctrl+Alt+Shift+Win+J");
}

#[test]
fn a_combo_without_modifiers_is_shown_as_its_key() {
    // Arrange
    let combo = KeyCombo {
        modifiers: Modifiers::NONE,
        key: function(5),
    };

    // Act
    let shown = combo.to_string();

    // Assert
    assert_eq!(shown, "F5");
}
