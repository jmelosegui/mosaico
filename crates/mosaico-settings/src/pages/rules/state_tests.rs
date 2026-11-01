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

    draft.move_to(0, 1);
    draft.move_to(1, 0);

    assert_eq!(draft.changes(), 0);
}

#[test]
fn a_new_rule_with_nothing_to_match_names_its_place() {
    let mut draft = Draft::new(vec![rule("A")]);

    draft.add();

    assert!(draft.check().unwrap_err().starts_with("rule 2:"));
}

#[test]
fn moving_a_rule_shifts_the_ones_between() {
    let mut draft = Draft::new(vec![rule("A"), rule("B"), rule("C")]);

    draft.move_to(0, 2);

    let classes: Vec<_> = draft
        .entries
        .iter()
        .map(|e| e.rule.match_class.clone().unwrap())
        .collect();
    assert_eq!(classes, ["B", "C", "A"]);
}

#[test]
fn a_pushed_rule_is_new() {
    let mut draft = Draft::new(vec![rule("A")]);

    let at = draft.push(rule("B"));

    assert_eq!(at, 1);
    assert_eq!(draft.changes(), 1);
}
