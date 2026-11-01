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
    click(&mut h, Role::Switch, "Follow the system setting");
    click(&mut h, Role::Button, "Apply");

    // Assert
    assert_eq!(
        config(&dir),
        "[animations]\neasing = \"linear\"\nfollow_system = false\n"
    );
}

#[test]
fn picking_a_custom_curve_starts_from_a_valid_curve() {
    // Arrange
    let store = folder("animations-custom");
    let dir = store.config_path().parent().unwrap().to_path_buf();
    let mut h = window(store, DaemonState::Stopped, &SignIn::default());
    open(&mut h, "Animations");

    // Act
    click(&mut h, Role::ComboBox, "Easing");
    h.get_by_label("Custom").click();
    h.run();
    click(&mut h, Role::Button, "Apply");

    // Assert
    assert_eq!(
        config(&dir),
        "[animations]\neasing = \"cubic-bezier(0.2, 0, 0, 1)\"\n"
    );
}

#[test]
fn a_point_of_a_custom_curve_can_be_typed() {
    // Arrange
    let store = folder("animations-point");
    let dir = store.config_path().parent().unwrap().to_path_buf();
    std::fs::write(
        dir.join("config.toml"),
        "[animations]\neasing = \"cubic-bezier(0.2, 0, 0, 1)\"\n",
    )
    .unwrap();
    let mut h = window(store, DaemonState::Stopped, &SignIn::default());
    open(&mut h, "Animations");

    // Act
    click(&mut h, Role::SpinButton, "Curve x2");
    h.get_by_role_and_label(Role::SpinButton, "Curve x2")
        .type_text("0.6");
    h.key_press(eframe::egui::Key::Enter);
    h.run();
    click(&mut h, Role::Button, "Apply");

    // Assert
    assert_eq!(
        config(&dir),
        "[animations]\neasing = \"cubic-bezier(0.2, 0, 0.6, 1)\"\n"
    );
}

#[test]
fn play_runs_the_dot_along_the_curve_and_stops() {
    // Arrange
    let (done, finished) = std::sync::mpsc::channel();

    // Act
    std::thread::spawn(move || {
        let store = folder("animations-play");
        let mut h = window(store, DaemonState::Stopped, &SignIn::default());
        open(&mut h, "Animations");
        h.get_by_role_and_label(Role::Button, "Play").click();
        h.run_steps(90);
        h.run();
        done.send(()).unwrap();
    });

    // Assert
    assert!(
        finished
            .recv_timeout(std::time::Duration::from_secs(60))
            .is_ok(),
        "the window stopped responding after Play was clicked"
    );
}

/// How many frames, a quarter of a second apart, the window draws after
/// Play is clicked with `duration` typed into Duration and not applied:
/// as many as the dot runs along the curve. `u64::MAX` when it still
/// runs after a second.
fn frames_played(test: &str, duration: &str) -> u64 {
    let mut h = window(folder(test), DaemonState::Stopped, &SignIn::default());
    open(&mut h, "Animations");
    click(&mut h, Role::SpinButton, "Duration");
    h.get_by_role_and_label(Role::SpinButton, "Duration")
        .type_text(duration);
    h.key_press(eframe::egui::Key::Enter);
    h.run();
    h.get_by_role_and_label(Role::Button, "Play").click();
    h.try_run().unwrap_or(u64::MAX)
}

#[test]
fn play_runs_the_dot_for_the_duration_being_edited() {
    // Act
    let short = frames_played("animations-play-short", "100");
    let long = frames_played("animations-play-long", "1000");

    // Assert
    assert!(short < long, "100 ms played {short} frames, 1000 ms {long}");
}

#[test]
fn a_preset_shows_its_points_without_letting_them_be_edited() {
    // Arrange
    let store = folder("animations-preset-points");
    let dir = store.config_path().parent().unwrap().to_path_buf();
    let mut h = window(store, DaemonState::Stopped, &SignIn::default());
    open(&mut h, "Animations");

    // Act
    click(&mut h, Role::SpinButton, "Curve x2");
    h.get_by_role_and_label(Role::SpinButton, "Curve x2")
        .type_text("0.6");
    h.key_press(eframe::egui::Key::Enter);
    h.run();
    click(&mut h, Role::Button, "Apply");

    // Assert
    assert_eq!(config(&dir), "");
    assert_eq!(easing(&h), &SettingValue::Text("ease-out-cubic".into()));
}
