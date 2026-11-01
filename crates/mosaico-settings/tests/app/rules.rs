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

fn editor_is_open(h: &Harness<'static, SettingsApp>, n: usize) -> bool {
    let label = format!("Class is of rule {n}");
    h.query_by_role_and_label(Role::TextInput, &label).is_some()
}

#[test]
fn the_header_names_both_files() {
    let (mut h, _) = with_file("rules-header", "");

    open(&mut h, "Rules");

    h.get_by_label("user-rules.toml and rules.toml");
    h.get_by_label("Your rules");
}

#[test]
fn adding_a_rule_opens_its_editor_and_applying_writes_user_rules_toml() {
    // Arrange
    let (mut h, path) = with_file("rules-add", "");
    open(&mut h, "Rules");

    // Act
    click(&mut h, Role::Button, "Add rule");
    assert!(editor_is_open(&h, 1));
    type_into(&mut h, "Class is of rule 1", "Popup");
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
fn a_saved_rule_shows_its_chips_and_what_it_does() {
    let (mut h, _) = with_file("rules-chips", FILE);

    open(&mut h, "Rules");

    h.get_by_label("class is");
    h.get_by_label("CalcFrame");
    h.get_by_label("Float");
    assert!(!editor_is_open(&h, 1));
}

#[test]
fn editing_a_rule_keeps_the_comments_of_the_file() {
    // Arrange
    let (mut h, path) = with_file("rules-edit", FILE);
    open(&mut h, "Rules");

    // Act
    click(&mut h, Role::Button, "Edit rule 1");
    assert!(editor_is_open(&h, 1));
    click(&mut h, Role::RadioButton, "Tile");
    type_into(&mut h, "Title contains of rule 1", "Calc");
    click(&mut h, Role::Button, "Done");
    click(&mut h, Role::Button, "Apply");

    // Assert
    assert!(!editor_is_open(&h, 1));
    let written = std::fs::read_to_string(path).unwrap();
    let expected = FILE.replace(
        "\"CalcFrame\"\naction = \"float\"",
        "\"CalcFrame\"\naction = \"tile\"\nmatch_title = \"Calc\"",
    );
    assert_eq!(written, expected);
}

#[test]
fn a_rule_can_float_the_windows_it_matches() {
    // Arrange
    let (mut h, path) = with_file("rules-float", "");
    open(&mut h, "Rules");
    click(&mut h, Role::Button, "Add rule");
    type_into(&mut h, "Class is of rule 1", "Popup");

    // Act
    click(&mut h, Role::RadioButton, "Float");
    click(&mut h, Role::Button, "Apply");

    // Assert
    let written = std::fs::read_to_string(path).unwrap();
    assert_eq!(
        written,
        "[[rule]]\nmatch_class = \"Popup\"\naction = \"float\"\n"
    );
}

#[test]
fn not_tiling_a_floating_rule_leaves_its_windows_alone() {
    // Arrange
    let (mut h, path) = with_file("rules-ignore", FILE);
    open(&mut h, "Rules");

    // Act
    click(&mut h, Role::Button, "Edit rule 1");
    click(&mut h, Role::RadioButton, "Don't tile");
    click(&mut h, Role::Button, "Apply");

    // Assert
    let written = std::fs::read_to_string(path).unwrap();
    assert_eq!(written, FILE.replace("\"float\"", "\"ignore\""));
}

#[test]
fn deleting_a_rule_removes_it_from_the_file() {
    // Arrange
    let (mut h, path) = with_file("rules-delete", FILE);
    open(&mut h, "Rules");
    click(&mut h, Role::Button, "Add rule");
    type_into(&mut h, "Title contains of rule 2", "Popup");
    click(&mut h, Role::Button, "Apply");

    // Act
    click(&mut h, Role::Button, "Delete rule 2");
    click(&mut h, Role::Button, "Apply");

    // Assert
    assert!(!editor_is_open(&h, 2));
    assert_eq!(std::fs::read_to_string(path).unwrap(), FILE);
}

#[test]
fn community_rules_are_read_only_and_can_be_overridden() {
    // Arrange
    let store = folder("rules-community");
    std::fs::write(
        store.rules_path(),
        "[[rule]]\nmatch_class = \"Shared\"\naction = \"ignore\"\n",
    )
    .unwrap();
    let user = store.user_rules_path();
    let mut h = window(store, DaemonState::Stopped, &SignIn::default());
    open(&mut h, "Rules");
    h.get_by_label("Community rules");
    h.get_by_label("Shared");
    assert!(!editor_is_open(&h, 1));
    h.get_by_label("No unsaved changes");

    // Act
    click(&mut h, Role::Button, "Override community rule 1");

    // Assert
    h.get_by_label("1 unsaved change");
    assert!(editor_is_open(&h, 1));
    click(&mut h, Role::Button, "Apply");
    let written = std::fs::read_to_string(user).unwrap();
    assert_eq!(
        written,
        "[[rule]]\nmatch_class = \"Shared\"\naction = \"ignore\"\n"
    );
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
    h.get_by_role_and_label(Role::Button, "Edit rule 1");
    assert_eq!(std::fs::read_to_string(path).unwrap(), FILE);
}

#[test]
fn reopening_shows_the_saved_rules() {
    // Arrange
    let (mut h, path) = with_file("rules-reopen", "");
    open(&mut h, "Rules");
    click(&mut h, Role::Button, "Add rule");
    type_into(&mut h, "Executable path ends with of rule 1", "tool.exe");
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
    again.get_by_label("tool.exe");
    again.get_by_label("No unsaved changes");
}
