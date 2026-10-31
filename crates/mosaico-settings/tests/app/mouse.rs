//! The Mouse page.

use crate::support::*;

#[test]
fn the_mouse_page_saves_its_switches_and_shows_them_when_reopened() {
    // Arrange
    let store = folder("mouse-page");
    let dir = store.config_path().parent().unwrap().to_path_buf();
    let mut h = window(store, DaemonState::Stopped, &SignIn::default());
    open(&mut h, "Mouse");

    // Act
    click(&mut h, Role::Switch, "Focus follows pointer");
    click(&mut h, Role::Switch, "Drag to swap");
    click(&mut h, Role::Button, "Apply");

    // Assert
    let written = std::fs::read_to_string(dir.join("config.toml")).unwrap();
    assert_eq!(
        written,
        "[mouse]\ndrag_to_swap = false\nfocus_follows_mouse = true\n"
    );
    let mut again = window(
        ConfigStore::new(dir),
        DaemonState::Stopped,
        &SignIn::default(),
    );
    open(&mut again, "Mouse");
    let edit = again.state().edit();
    assert_eq!(
        edit.get("mouse.focus_follows_mouse"),
        &SettingValue::Bool(true)
    );
    assert_eq!(edit.get("mouse.drag_to_swap"), &SettingValue::Bool(false));
}
