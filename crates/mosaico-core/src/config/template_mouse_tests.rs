use crate::Action;
use crate::config::WorkspacesConfig;
use crate::config::bar::BarMouseConfig;

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
fn keybindings_template_names_the_steps_through_the_workspaces() {
    // Arrange
    let text = generate_keybindings();

    // Act / Assert
    for name in ["goto-workspace-next", "goto-workspace-prev"] {
        assert!(text.contains(name), "{name} is not listed");
        assert!(name.parse::<Action>().is_ok(), "{name} is not an action");
    }
}
