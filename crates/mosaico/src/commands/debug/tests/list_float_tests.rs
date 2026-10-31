use mosaico_core::WindowId;

use super::*;
use crate::commands::fake_desktop::{app_window, monitor};
use crate::commands::test_support::Harness;

#[test]
fn a_window_a_rule_floats_shows_as_floating() {
    // Arrange
    let mut h = Harness::new("list-float");
    h.desktop.monitors = vec![monitor(1, 0)];
    h.desktop.windows = vec![(WindowId::from_raw(1), app_window(1))];
    let float_app = WindowRule {
        match_class: Some("App".into()),
        match_title: None,
        match_path: None,
        action: RuleAction::Float,
    };

    // Act
    h.run(|cx| list(cx, &[float_app]));

    // Assert
    let out = h.out();
    assert!(
        out.contains("Managed: floating  Monitor: 1  State: normal"),
        "{out}"
    );
}
