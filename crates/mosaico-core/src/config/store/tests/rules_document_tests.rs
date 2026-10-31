use crate::config::store::test_support::{contents, temp_store};
use crate::config::{RuleAction, WindowRule};

const FILE: &str = "\
# My rules\n\
\n\
# Calculator floats\n\
[[rule]]\n\
match_class = \"CalcFrame\" # the class\n\
action = \"float\"\n\
\n\
# Never tile this one\n\
[[rule]]\n\
match_path = \"tool.exe\"\n\
manage = false\n\
note = \"kept\"\n";

fn title(text: &str, action: RuleAction) -> WindowRule {
    WindowRule {
        match_class: None,
        match_title: Some(text.into()),
        match_path: None,
        action,
    }
}

#[test]
fn rules_are_listed_in_the_order_of_the_file() {
    // Arrange
    let store = temp_store("rules-list");
    std::fs::write(store.user_rules_path(), FILE).unwrap();

    // Act
    let rules = store.open_user_rules().unwrap().rules();

    // Assert
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].match_class.as_deref(), Some("CalcFrame"));
    assert_eq!(rules[1].action, RuleAction::Ignore);
}

#[test]
fn a_missing_file_opens_empty_and_is_created_by_the_first_save() {
    // Arrange
    let store = temp_store("rules-missing");
    let mut doc = store.open_user_rules().unwrap();
    assert!(doc.rules().is_empty());

    // Act
    doc.add(title("Popup", RuleAction::Float)).unwrap();
    doc.save().unwrap();

    // Assert
    let text = contents(&store.user_rules_path());
    assert_eq!(
        text,
        "[[rule]]\nmatch_title = \"Popup\"\naction = \"float\"\n"
    );
    assert_eq!(store.try_load_user_rules().unwrap().len(), 1);
}

#[test]
fn adding_a_rule_keeps_the_comments_and_the_other_rules() {
    // Arrange
    let store = temp_store("rules-add");
    std::fs::write(store.user_rules_path(), FILE).unwrap();
    let mut doc = store.open_user_rules().unwrap();

    // Act
    doc.add(title("Popup", RuleAction::Float)).unwrap();
    doc.save().unwrap();

    // Assert
    let text = contents(&store.user_rules_path());
    assert!(text.starts_with(FILE), "{text}");
    assert!(text.ends_with("match_title = \"Popup\"\naction = \"float\"\n"));
    assert_eq!(store.try_load_user_rules().unwrap().len(), 3);
}

#[test]
fn editing_a_rule_changes_its_keys_and_keeps_its_comments() {
    // Arrange
    let store = temp_store("rules-edit");
    std::fs::write(store.user_rules_path(), FILE).unwrap();
    let mut doc = store.open_user_rules().unwrap();
    let mut rule = doc.rules().remove(0);
    rule.match_class = Some("Calc".into());
    rule.action = RuleAction::Ignore;

    // Act
    doc.update(0, rule).unwrap();
    doc.save().unwrap();

    // Assert
    let expected = FILE
        .replace("\"CalcFrame\" # the class", "\"Calc\" # the class")
        .replace("action = \"float\"", "action = \"ignore\"");
    assert_eq!(contents(&store.user_rules_path()), expected);
}

#[test]
fn editing_the_action_of_an_older_rule_replaces_manage() {
    // Arrange
    let store = temp_store("rules-manage");
    std::fs::write(store.user_rules_path(), FILE).unwrap();
    let mut doc = store.open_user_rules().unwrap();
    let mut rule = doc.rules().remove(1);
    rule.action = RuleAction::Float;

    // Act
    doc.update(1, rule).unwrap();
    doc.save().unwrap();

    // Assert
    let text = contents(&store.user_rules_path());
    assert!(!text.contains("manage"), "{text}");
    assert!(text.contains("note = \"kept\""), "{text}");
    assert_eq!(
        store.try_load_user_rules().unwrap()[1].action,
        RuleAction::Float
    );
}

#[test]
fn removing_a_rule_takes_its_comment_and_leaves_the_rest() {
    // Arrange
    let store = temp_store("rules-remove");
    std::fs::write(store.user_rules_path(), FILE).unwrap();
    let mut doc = store.open_user_rules().unwrap();

    // Act
    doc.remove(0).unwrap();
    doc.save().unwrap();

    // Assert
    let text = contents(&store.user_rules_path());
    assert!(text.starts_with("# My rules\n"), "{text}");
    assert!(!text.contains("Calculator"), "{text}");
    assert!(text.contains("# Never tile this one"), "{text}");
    assert_eq!(store.try_load_user_rules().unwrap().len(), 1);
}

#[test]
fn moving_a_rule_changes_the_order_and_takes_its_comment_along() {
    // Arrange
    let store = temp_store("rules-move");
    std::fs::write(store.user_rules_path(), FILE).unwrap();
    let mut doc = store.open_user_rules().unwrap();

    // Act
    doc.move_up(1).unwrap();
    doc.move_up(0).unwrap();
    doc.save().unwrap();

    // Assert
    let text = contents(&store.user_rules_path());
    let tool = text.find("# Never tile this one").unwrap();
    let calc = text.find("# Calculator floats").unwrap();
    assert!(tool < calc, "{text}");
    assert!(text[calc..].contains("CalcFrame"), "{text}");
    let rules = store.try_load_user_rules().unwrap();
    assert_eq!(rules[0].match_path.as_deref(), Some("tool.exe"));

    doc.move_down(0).unwrap();
    assert_eq!(doc.rules()[0].match_class.as_deref(), Some("CalcFrame"));
}

#[test]
fn rules_the_loader_would_refuse_are_not_put_in() {
    // Arrange
    let store = temp_store("rules-refused");
    let mut doc = store.open_user_rules().unwrap();
    let nothing = WindowRule {
        match_class: None,
        match_title: None,
        match_path: None,
        action: RuleAction::Tile,
    };
    let blank = WindowRule {
        match_class: Some(" ".into()),
        ..title("x", RuleAction::Tile)
    };

    // Act
    let results = [doc.add(nothing), doc.add(blank), doc.remove(3)];

    // Assert
    assert!(results.iter().all(Result::is_err));
    assert!(!doc.is_dirty());
    assert!(doc.rules().is_empty());
}

#[test]
fn a_file_the_loader_refuses_does_not_open() {
    let store = temp_store("rules-broken");
    std::fs::write(store.user_rules_path(), "[[rule]]\nmatch_class = \"A\"\n").unwrap();

    let error = store.open_user_rules().unwrap_err().to_string();

    assert!(error.contains("action"), "{error}");
}

#[test]
fn crlf_line_endings_survive_a_save() {
    // Arrange
    let store = temp_store("rules-crlf");
    std::fs::write(store.user_rules_path(), FILE.replace('\n', "\r\n")).unwrap();
    let mut doc = store.open_user_rules().unwrap();

    // Act
    doc.add(title("Popup", RuleAction::Float)).unwrap();
    doc.save().unwrap();

    // Assert
    let text = contents(&store.user_rules_path());
    assert!(text.starts_with(&FILE.replace('\n', "\r\n")), "{text}");
    assert_eq!(text.matches('\n').count(), text.matches("\r\n").count());
}

#[test]
fn an_edit_that_changes_nothing_is_not_dirty() {
    let store = temp_store("rules-clean");
    std::fs::write(store.user_rules_path(), FILE).unwrap();
    let mut doc = store.open_user_rules().unwrap();

    let same = doc.rules().remove(0);
    doc.update(0, same).unwrap();
    doc.move_up(0).unwrap();

    assert!(!doc.is_dirty());
    assert!(!doc.has_changed_on_disk());
}
