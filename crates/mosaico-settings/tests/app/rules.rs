//! The Rules page.

use crate::support::*;

const FILE: &str =
    "# my rules\n\n# calculator\n[[rule]]\nmatch_class = \"CalcFrame\"\naction = \"float\"\n";

fn with_file(test: &str, text: &str) -> (Harness<'static, SettingsApp>, PathBuf) {
    let store = folder(test);
    let path = store.user_rules_path();
    if !text.is_empty() {
        std::fs::write(&path, text).unwrap();
    }
    let h = window(store, DaemonState::Stopped, &SignIn::default());
    (h, path)
}

fn type_into(h: &mut Harness<'static, SettingsApp>, label: &str, text: &str) {
    h.get_by_role_and_label(Role::TextInput, label).click();
    h.run();
    h.get_by_role_and_label(Role::TextInput, label)
        .type_text(text);
    h.run();
}

#[test]
fn adding_a_rule_and_applying_writes_user_rules_toml() {
    // Arrange
    let (mut h, path) = with_file("rules-add", "");
    open(&mut h, "Rules");

    // Act
    click(&mut h, Role::Button, "Add rule");
    type_into(&mut h, "Class of rule 1", "Popup");
    click(&mut h, Role::Button, "Apply");

    // Assert
    let written = std::fs::read_to_string(path).unwrap();
    assert_eq!(
        written,
        "[[rule]]\nmatch_class = \"Popup\"\naction = \"tile\"\n"
    );
    h.get_by_label("Changes saved");
}

#[test]
fn a_rule_with_nothing_to_match_is_not_saved() {
    let (mut h, path) = with_file("rules-blank", "");
    open(&mut h, "Rules");

    click(&mut h, Role::Button, "Add rule");
    click(&mut h, Role::Button, "Apply");

    h.get_by_label("Could not save: rule 1: a rule needs a class, a title or a path to match");
    assert!(!path.exists());
}

#[test]
fn editing_and_removing_keep_the_comments_of_the_file() {
    // Arrange
    let (mut h, path) = with_file("rules-edit", FILE);
    open(&mut h, "Rules");
    click(&mut h, Role::Button, "Add rule");
    type_into(&mut h, "Title contains of rule 2", "Popup");
    click(&mut h, Role::Button, "Apply");

    // Act
    click(&mut h, Role::ComboBox, "Action of rule 1");
    h.get_by_label("Ignore").click();
    h.run();
    click(&mut h, Role::Button, "Move rule 1 down");
    click(&mut h, Role::Button, "Apply");
    click(&mut h, Role::Button, "Delete rule 1");
    click(&mut h, Role::Button, "Apply");

    // Assert
    let written = std::fs::read_to_string(path).unwrap();
    let expected = FILE.replace("\"float\"", "\"ignore\"");
    assert_eq!(written, expected);
}

#[test]
fn community_rules_are_shown_and_not_editable() {
    // Arrange
    let store = folder("rules-community");
    std::fs::write(
        store.rules_path(),
        "[[rule]]\nmatch_class = \"Shared\"\naction = \"ignore\"\n",
    )
    .unwrap();
    let mut h = window(store, DaemonState::Stopped, &SignIn::default());
    open(&mut h, "Rules");

    // Act
    h.get_by_label("Community rules (1)").click();
    h.run();

    // Assert
    h.get_by_label("ignore: class \u{201c}Shared\u{201d}");
    let field = h.query_by_role_and_label(Role::TextInput, "Class of rule 1");
    assert!(field.is_none());
    h.get_by_label("No unsaved changes");
}

#[test]
fn revert_drops_the_edits() {
    // Arrange
    let (mut h, path) = with_file("rules-revert", FILE);
    open(&mut h, "Rules");
    click(&mut h, Role::Button, "Delete rule 1");
    h.get_by_label("1 unsaved change");

    // Act
    click(&mut h, Role::Button, "Revert");

    // Assert
    h.get_by_label("No unsaved changes");
    h.get_by_role_and_label(Role::TextInput, "Class of rule 1");
    assert_eq!(std::fs::read_to_string(path).unwrap(), FILE);
}

#[test]
fn reopening_shows_the_saved_rules() {
    // Arrange
    let (mut h, path) = with_file("rules-reopen", "");
    open(&mut h, "Rules");
    click(&mut h, Role::Button, "Add rule");
    type_into(&mut h, "Program path of rule 1", "tool.exe");
    click(&mut h, Role::Button, "Apply");

    // Act
    let dir = path.parent().unwrap().to_path_buf();
    let mut again = window(
        ConfigStore::new(dir),
        DaemonState::Stopped,
        &SignIn::default(),
    );
    open(&mut again, "Rules");

    // Assert
    again.get_by_role_and_label(Role::TextInput, "Program path of rule 1");
    again.get_by_label("No unsaved changes");
}
