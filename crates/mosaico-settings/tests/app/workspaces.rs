//! The Workspaces page.

use crate::support::*;

fn pick(h: &mut Harness<'static, SettingsApp>, workspace: &str, layout: &str) {
    click(h, Role::ComboBox, workspace);
    h.get_by_label(layout).click();
    h.run();
}

fn written(dir: &std::path::Path) -> String {
    std::fs::read_to_string(dir.join("config.toml")).unwrap()
}

#[test]
fn a_layout_picked_for_a_workspace_is_saved_in_its_table_and_shown_when_reopened() {
    // Arrange
    let store = folder("workspaces-layout");
    let dir = store.config_path().parent().unwrap().to_path_buf();
    let mut h = window(store, DaemonState::Stopped, &SignIn::default());
    open(&mut h, "Workspaces");

    // Act
    pick(&mut h, "Workspace 1", "Vertical stack");
    pick(&mut h, "Workspace 2", "BSP");
    click(&mut h, Role::Button, "Apply");

    // Assert
    assert_eq!(
        written(&dir),
        "[workspaces.layouts]\n1 = \"vertical-stack\"\n2 = \"bsp\"\n"
    );
    let mut again = window(
        ConfigStore::new(dir),
        DaemonState::Stopped,
        &SignIn::default(),
    );
    open(&mut again, "Workspaces");
    let edit = again.state().edit();
    assert_eq!(
        edit.get("workspaces.layouts.1"),
        &SettingValue::Text("vertical-stack".into())
    );
    assert_eq!(
        edit.get("workspaces.layouts.2"),
        &SettingValue::Text("bsp".into())
    );
    assert_eq!(
        edit.get("workspaces.layouts.3"),
        &SettingValue::Text(String::new())
    );
    assert!(again.query_by_label("2 unsaved changes").is_none());
}

#[test]
fn setting_a_workspace_back_to_the_default_layout_removes_its_entry() {
    // Arrange
    let store = folder("workspaces-default");
    let dir = store.config_path().parent().unwrap().to_path_buf();
    std::fs::write(
        dir.join("config.toml"),
        "# mine\n[workspaces.layouts]\n1 = \"bsp\"\n2 = \"three-column\"\n",
    )
    .unwrap();
    let mut h = window(store, DaemonState::Stopped, &SignIn::default());
    open(&mut h, "Workspaces");

    // Act
    pick(&mut h, "Workspace 1", "Default (BSP)");
    click(&mut h, Role::Button, "Apply");

    // Assert
    assert_eq!(
        written(&dir),
        "# mine\n[workspaces.layouts]\n2 = \"three-column\"\n"
    );
}

#[test]
fn the_switching_mode_and_wrapping_are_saved() {
    // Arrange
    let store = folder("workspaces-switching");
    let dir = store.config_path().parent().unwrap().to_path_buf();
    let mut h = window(store, DaemonState::Stopped, &SignIn::default());
    open(&mut h, "Workspaces");

    // Act
    click(&mut h, Role::RadioButton, "Global");
    click(&mut h, Role::Switch, "Wrap around");
    click(&mut h, Role::Button, "Apply");

    // Assert
    assert_eq!(
        written(&dir),
        "[workspaces]\nmode = \"global\"\nwrap = true\n"
    );
}

#[test]
fn a_workspace_without_a_layout_of_its_own_names_the_default_layout() {
    // Arrange
    let store = folder("workspaces-default-name");
    let dir = store.config_path().parent().unwrap().to_path_buf();
    std::fs::write(
        dir.join("config.toml"),
        "[layout]\ndefault = \"three-column\"\n",
    )
    .unwrap();
    let mut h = window(store, DaemonState::Stopped, &SignIn::default());

    // Act
    open(&mut h, "Workspaces");

    // Assert
    assert_eq!(h.get_all_by_value("Default (Three column)").count(), 8);
}
