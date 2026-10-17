use crate::Action;
use crate::action::Scope;
use crate::config::bar::{BarMouseConfig, ClickBinding};
use crate::config::{MouseConfig, WorkspacesConfig};

use super::*;

#[test]
fn bar_template_writes_out_the_mouse_section_at_its_defaults() {
    // Arrange
    let toml_str = generate_bar();

    // Act
    let file: toml::Table = toml::from_str(&toml_str).unwrap();
    let config: crate::BarConfig = toml::from_str(&toml_str).unwrap();

    // Assert
    let mouse = file["mouse"].as_table().expect("a [mouse] table");
    assert!(mouse.contains_key("enabled") && mouse.contains_key("scroll"));
    assert_eq!(config.mouse, BarMouseConfig::default());
}

#[test]
fn config_template_writes_out_wrap_at_its_default() {
    // Arrange
    let toml_str = generate_config();

    // Act
    let file: toml::Table = toml::from_str(&toml_str).unwrap();
    let config: crate::Config = toml::from_str(&toml_str).unwrap();

    // Assert
    let workspaces = file["workspaces"].as_table().expect("a [workspaces] table");
    assert!(workspaces.contains_key("wrap"));
    assert_eq!(config.workspaces.wrap, WorkspacesConfig::default().wrap);
}

#[test]
fn config_template_writes_out_drag_to_swap_at_its_default() {
    // Arrange
    let toml_str = generate_config();

    // Act
    let file: toml::Table = toml::from_str(&toml_str).unwrap();
    let config: crate::Config = toml::from_str(&toml_str).unwrap();

    // Assert
    let mouse = file["mouse"].as_table().expect("a [mouse] table");
    assert!(mouse.contains_key("drag_to_swap"));
    assert_eq!(config.mouse, MouseConfig::default());
}

#[test]
fn keybindings_template_names_the_steps_through_the_workspaces() {
    // Arrange
    let text = generate_keybindings();

    // Act / Assert
    for name in ["goto-workspace-next", "goto-workspace-prev"] {
        assert!(text.contains(name), "{name} is not listed");
        assert!(name.parse::<Action>().is_ok(), "{name} is not an action");
    }
}

#[test]
fn bar_template_writes_out_the_default_clicks_and_names_actions_a_bar_can_run() {
    // Arrange
    let toml_str = generate_bar();

    // Act
    let file: toml::Table = toml::from_str(&toml_str).unwrap();
    let config: crate::BarConfig = toml::from_str(&toml_str).unwrap();

    // Assert -- as [[mouse.click]] entries, which the user can change.
    let entries = file["mouse"]["click"]
        .as_array()
        .expect("[[mouse.click]] entries");
    assert_eq!(entries.len(), ClickBinding::defaults().len());
    assert_eq!(config.mouse.click, ClickBinding::defaults());
    let named = [
        "cycle-layout",
        "toggle-monocle",
        "goto-workspace-next",
        "goto-workspace-prev",
        "toggle-pause",
        "retile",
    ];
    for name in named {
        assert!(toml_str.contains(name), "{name} is not listed");
        let action: Action = name.parse().unwrap();
        assert_ne!(action.scope(), Scope::FocusedWindow, "{name}");
    }
}

#[test]
fn bar_template_says_that_none_removes_a_binding_and_a_deleted_entry_keeps_it() {
    // Arrange
    let toml_str = generate_bar();

    // Act
    let says = |text: &str| toml_str.contains(text);

    // Assert
    assert!(says("action = \"none\" removes the binding"));
    assert!(says("deleting an entry keeps"));
    assert!(says("with double brackets"));
}
