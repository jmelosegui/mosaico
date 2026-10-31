use super::*;
use crate::config::validate_rules;

fn rules(toml: &str) -> Vec<WindowRule> {
    validate_rules(toml).unwrap()
}

#[test]
fn a_rule_says_what_it_does_with_action() {
    // Arrange
    let toml = r#"
[[rule]]
match_class = "A"
action = "tile"

[[rule]]
match_class = "B"
action = "float"

[[rule]]
match_class = "C"
action = "ignore"
"#;

    // Act
    let actions: Vec<_> = rules(toml).iter().map(|r| r.action).collect();

    // Assert
    assert_eq!(
        actions,
        [RuleAction::Tile, RuleAction::Float, RuleAction::Ignore]
    );
}

#[test]
fn manage_still_means_tile_or_ignore() {
    // Arrange
    let toml = r#"
[[rule]]
match_class = "A"
manage = true

[[rule]]
match_class = "B"
manage = false
"#;

    // Act
    let actions: Vec<_> = rules(toml).iter().map(|r| r.action).collect();

    // Assert
    assert_eq!(actions, [RuleAction::Tile, RuleAction::Ignore]);
}

#[test]
fn a_rule_with_both_action_and_manage_is_refused() {
    // Arrange
    let toml = "[[rule]]\nmatch_class = \"A\"\naction = \"float\"\nmanage = true\n";

    // Act
    let error = validate_rules(toml).unwrap_err();

    // Assert
    assert!(error.to_string().contains("not both"), "{error}");
}

#[test]
fn a_rule_that_does_nothing_is_refused() {
    // Arrange
    let toml = "[[rule]]\nmatch_class = \"A\"\n";

    // Act
    let error = validate_rules(toml).unwrap_err();

    // Assert
    assert!(error.to_string().contains("needs an `action`"), "{error}");
}

#[test]
fn an_unknown_action_is_refused() {
    // Arrange
    let toml = "[[rule]]\nmatch_class = \"A\"\naction = \"hover\"\n";

    // Act / Assert
    assert!(validate_rules(toml).is_err());
}

#[test]
fn a_window_without_a_title_bar_floats_only_when_a_rule_says_so() {
    // Arrange
    let float = rules("[[rule]]\nmatch_class = \"Popup\"\naction = \"float\"\n");

    // Act / Assert
    assert_eq!(decide("Popup", "", "", false, &float), RuleAction::Float);
    assert_eq!(decide("Other", "", "", false, &float), RuleAction::Ignore);
    assert_eq!(decide("Other", "", "", true, &float), RuleAction::Tile);
}
