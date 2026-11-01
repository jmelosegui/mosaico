use crate::Action;
use crate::action::Direction;
use crate::config::Modifier::{Alt, Ctrl, Shift};
use crate::config::keybinding;
use crate::config::store::test_support::{TempStore, contents, temp_store};

const FILE: &str = "# my keys\n\n[[keybinding]]\naction = \"focus-down\" # down\nkey = \"J\" # the key\nmodifiers = [\"alt\"] # held\n\n# floating\n[[keybinding]]\naction = \"toggle-float\"\nkey = \"F\"\nmodifiers = [\"alt\"]\n";

fn store_with(test: &str, text: &str) -> TempStore {
    let store = temp_store(test);
    std::fs::write(store.keybindings_path(), text).unwrap();
    store
}

#[test]
fn changing_a_key_keeps_the_comments_and_the_order() {
    // Arrange
    let store = store_with("kb-doc-change", FILE);
    let mut doc = store.open_keybindings().unwrap();

    // Act
    doc.set_key(0, "down", &[Shift, Ctrl]).unwrap();
    assert!(doc.is_dirty());
    doc.save().unwrap();

    // Assert
    let expected = FILE
        .replace("key = \"J\" # the key", "key = \"Down\" # the key")
        .replace("[\"alt\"] # held", "[\"shift\", \"ctrl\"] # held");
    assert_eq!(contents(&store.keybindings_path()), expected);
    let loaded = store.try_load_keybindings().unwrap();
    assert_eq!(loaded[0].key, "Down");
    assert_eq!(loaded[1].action, Action::ToggleFloat);
}

#[test]
fn crlf_files_stay_crlf() {
    let store = store_with("kb-doc-crlf", &FILE.replace('\n', "\r\n"));
    let mut doc = store.open_keybindings().unwrap();

    doc.set_key(1, "G", &[Alt]).unwrap();
    doc.save().unwrap();

    let written = contents(&store.keybindings_path());
    assert!(written.contains("key = \"G\"\r\n"), "{written:?}");
    assert_eq!(written.replace("\r\n", "").matches('\n').count(), 0);
}

#[test]
fn adding_and_removing_bindings_leave_the_rest_alone() {
    // Arrange
    let store = store_with("kb-doc-add-remove", FILE);
    let mut doc = store.open_keybindings().unwrap();

    // Act
    doc.add(Action::Retile, "r", &[Alt, Shift]).unwrap();
    doc.remove(1).unwrap();
    doc.save().unwrap();

    // Assert
    let written = contents(&store.keybindings_path());
    assert!(written.starts_with("# my keys\n"), "{written}");
    assert!(written.contains("focus-down\" # down"), "{written}");
    assert!(!written.contains("toggle-float"), "{written}");
    let loaded = store.try_load_keybindings().unwrap();
    let actions: Vec<_> = loaded.iter().map(|b| b.action).collect();
    assert_eq!(actions, [Action::Focus(Direction::Down), Action::Retile]);
    assert_eq!(
        (loaded[1].key.as_str(), loaded[1].modifiers.len()),
        ("R", 2)
    );
}

#[test]
fn an_unknown_key_is_refused_and_changes_nothing() {
    let store = store_with("kb-doc-bad-key", FILE);
    let mut doc = store.open_keybindings().unwrap();

    assert!(doc.set_key(0, "Hyper", &[Alt]).is_err());
    assert!(doc.add(Action::Retile, "", &[Alt]).is_err());
    assert!(doc.set_key(9, "K", &[Alt]).is_err());
    assert!(doc.remove(9).is_err());

    assert!(!doc.is_dirty());
    doc.save().unwrap();
    assert_eq!(contents(&store.keybindings_path()), FILE);
}

#[test]
fn a_file_the_loader_refuses_does_not_open() {
    let store = store_with("kb-doc-broken", "[[keybinding]]\naction = \"dance\"\n");

    let error = store.open_keybindings().unwrap_err();

    assert!(error.to_string().contains("keybindings.toml"), "{error}");
}

#[test]
fn conflicts_are_reported_whatever_the_case_and_order() {
    let store = store_with("kb-doc-conflicts", FILE);
    let mut doc = store.open_keybindings().unwrap();
    assert!(doc.conflicts().is_empty());

    doc.set_key(1, "j", &[Alt]).unwrap();

    let found = doc.conflicts();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].bindings, [0, 1]);
}

#[test]
fn a_missing_file_opens_with_the_defaults_and_is_created_by_a_save() {
    // Arrange
    let store = temp_store("kb-doc-missing");
    let mut doc = store.open_keybindings().unwrap();
    assert_eq!(doc.bindings().len(), keybinding::defaults().len());
    assert!(!store.keybindings_path().exists());

    // Act
    doc.set_key(0, "Down", &[Alt]).unwrap();
    doc.save().unwrap();

    // Assert
    let loaded = store.try_load_keybindings().unwrap();
    assert_eq!(loaded.len(), keybinding::defaults().len());
    assert_eq!(loaded[0].action, Action::Focus(Direction::Down));
    assert_eq!(loaded[0].key, "Down");
}
