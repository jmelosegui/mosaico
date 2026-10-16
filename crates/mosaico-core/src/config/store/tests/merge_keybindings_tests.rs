use crate::Action;
use crate::action::Direction;
use crate::config::Modifier;
use crate::config::store::test_support::{contents, file_names, temp_store};

use super::*;

/// What a binding does and how it is pressed, for comparing lists.
fn summary(bindings: &[Keybinding]) -> Vec<(Action, String, Vec<Modifier>)> {
    bindings
        .iter()
        .map(|kb| (kb.action, kb.key.clone(), kb.modifiers.clone()))
        .collect()
}

#[test]
fn without_a_file_the_defaults_are_used_and_nothing_is_created() {
    // Arrange
    let store = temp_store("merge-keys-missing");

    // Act
    let bindings = store.merge_missing_keybindings();

    // Assert
    assert_eq!(summary(&bindings), summary(&keybinding::defaults()));
    assert!(file_names(&store).is_empty(), "{:?}", file_names(&store));
}

#[test]
fn default_actions_the_file_lacks_are_appended_and_returned() {
    // Arrange -- the user moved focus-down to the arrow key.
    let store = temp_store("merge-keys-old");
    let original =
        "[[keybinding]]\naction = \"focus-down\"\nkey = \"Down\"\nmodifiers = [\"alt\"]\n";
    std::fs::write(store.keybindings_path(), original).unwrap();

    // Act
    let bindings = store.merge_missing_keybindings();

    // Assert -- the user's binding stays first and is not added twice.
    let focus_down = Action::Focus(Direction::Down);
    assert_eq!(bindings.len(), keybinding::defaults().len());
    assert_eq!(bindings[0].key, "Down");
    assert_eq!(
        bindings.iter().filter(|kb| kb.action == focus_down).count(),
        1
    );
    let on_disk = contents(&store.keybindings_path());
    assert!(on_disk.starts_with(original), "{on_disk}");
    assert!(on_disk.contains("# Added automatically"), "{on_disk}");
    let reloaded = store.try_load_keybindings().unwrap();
    assert_eq!(summary(&reloaded), summary(&bindings));
}

#[test]
fn merging_keybindings_twice_appends_nothing_the_second_time() {
    // Arrange
    let store = temp_store("merge-keys-twice");
    let original = "[[keybinding]]\naction = \"retile\"\nkey = \"F5\"\nmodifiers = [\"ctrl\"]\n";
    std::fs::write(store.keybindings_path(), original).unwrap();
    let first = store.merge_missing_keybindings();
    let merged = contents(&store.keybindings_path());

    // Act
    let second = store.merge_missing_keybindings();

    // Assert
    assert_eq!(summary(&second), summary(&first));
    assert_eq!(contents(&store.keybindings_path()), merged);
}

#[test]
fn an_invalid_keybindings_file_is_left_alone_and_the_defaults_used() {
    // Arrange
    let store = temp_store("merge-keys-invalid");
    std::fs::write(store.keybindings_path(), "not = [valid").unwrap();

    // Act
    let bindings = store.merge_missing_keybindings();

    // Assert
    assert_eq!(summary(&bindings), summary(&keybinding::defaults()));
    assert_eq!(contents(&store.keybindings_path()), "not = [valid");
}
