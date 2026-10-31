use super::*;

fn rule(class: &str) -> WindowRule {
    WindowRule {
        match_class: Some(class.into()),
        match_title: None,
        match_path: None,
        action: RuleAction::Float,
    }
}

#[test]
fn an_untouched_list_has_no_changes() {
    let draft = Draft::new(vec![rule("A"), rule("B")]);

    assert_eq!(draft.changes(), 0);
}

#[test]
fn adds_edits_moves_and_removals_each_count() {
    // Arrange
    let mut draft = Draft::new(vec![rule("A"), rule("B"), rule("C")]);

    // Act
    draft.entries[2].rule.action = RuleAction::Ignore;
    let after_edit = draft.changes();
    draft.remove(0);
    let after_removal = draft.changes();
    draft.add();

    // Assert
    assert_eq!(after_edit, 1);
    // B and C moved up a place, C is edited, A is gone.
    assert_eq!(after_removal, 3);
    assert_eq!(draft.changes(), 4);
}

#[test]
fn putting_a_rule_back_where_it_was_is_not_a_change() {
    let mut draft = Draft::new(vec![rule("A"), rule("B")]);

    draft.move_down(0);
    draft.move_up(1);

    assert_eq!(draft.changes(), 0);
}

#[test]
fn a_new_rule_with_nothing_to_match_names_its_place() {
    let mut draft = Draft::new(vec![rule("A")]);

    draft.add();

    assert!(draft.check().unwrap_err().starts_with("rule 2:"));
}
