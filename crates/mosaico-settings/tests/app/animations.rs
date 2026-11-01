//! The Animations page.

use crate::support::*;

fn config(dir: &std::path::Path) -> String {
    std::fs::read_to_string(dir.join("config.toml")).unwrap_or_default()
}

fn easing<'a>(h: &'a Harness<'static, SettingsApp>) -> &'a SettingValue {
    h.state().edit().get("animations.easing")
}

#[test]
fn a_preset_easing_picked_from_the_list_is_saved_by_name() {
    // Arrange
    let store = folder("animations-preset");
    let dir = store.config_path().parent().unwrap().to_path_buf();
    let mut h = window(store, DaemonState::Stopped, &SignIn::default());
    open(&mut h, "Animations");

    // Act
    click(&mut h, Role::ComboBox, "Easing");
    h.get_by_label("Linear").click();
    h.run();
    click(&mut h, Role::Switch, "Follow the system");
    click(&mut h, Role::Button, "Apply");

    // Assert
    assert_eq!(
        config(&dir),
        "[animations]\neasing = \"linear\"\nfollow_system = false\n"
    );
}

#[test]
fn a_custom_curve_shows_a_field_that_starts_from_a_valid_curve() {
    // Arrange
    let store = folder("animations-custom");
    let dir = store.config_path().parent().unwrap().to_path_buf();
    let mut h = window(store, DaemonState::Stopped, &SignIn::default());
    open(&mut h, "Animations");
    assert!(
        h.query_by_role_and_label(Role::TextInput, "Custom curve")
            .is_none()
    );

    // Act
    click(&mut h, Role::ComboBox, "Easing");
    h.get_by_label("Custom curve").click();
    h.run();
    click(&mut h, Role::Button, "Apply");

    // Assert
    h.get_by_role_and_label(Role::TextInput, "Custom curve");
    assert_eq!(
        config(&dir),
        "[animations]\neasing = \"cubic-bezier(0.2, 0, 0, 1)\"\n"
    );
}

#[test]
fn a_curve_that_does_not_parse_says_why_and_is_not_saved() {
    // Arrange
    let store = folder("animations-invalid");
    let dir = store.config_path().parent().unwrap().to_path_buf();
    std::fs::write(
        dir.join("config.toml"),
        "[animations]\neasing = \"cubic-bezier(0.2, 0, 0, 1)\"\n",
    )
    .unwrap();
    let mut h = window(store, DaemonState::Stopped, &SignIn::default());
    open(&mut h, "Animations");

    // Act
    click(&mut h, Role::TextInput, "Custom curve");
    h.get_by_role_and_label(Role::TextInput, "Custom curve")
        .type_text("x");
    h.run();
    click(&mut h, Role::Button, "Apply");

    // Assert
    let says_why = h.query_all_by_label_contains("invalid easing").any(|n| {
        let node = n.accesskit_node();
        let text = node.label().or(node.value()).unwrap_or_default();
        text.starts_with("invalid easing")
    });
    assert!(says_why, "the row's help says why");
    assert_ne!(easing(&h), &SettingValue::Text(CUSTOM.into()));
    assert_eq!(
        config(&dir),
        "[animations]\neasing = \"cubic-bezier(0.2, 0, 0, 1)\"\n"
    );
}

const CUSTOM: &str = "cubic-bezier(0.2, 0, 0, 1)";
