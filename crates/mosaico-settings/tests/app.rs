//! Runs the settings window headless, with made-up settings and daemon.

use eframe::egui::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use mosaico_core::config::{CatppuccinFlavor, Theme};
use mosaico_settings::{DaemonState, General, Services, SettingsApp};

fn saved() -> General {
    General::defaults()
}

fn harness(daemon: DaemonState) -> Harness<'static, SettingsApp> {
    let app = SettingsApp::new(Services {
        general: saved(),
        log_file: "logs/mosaico.log".into(),
        daemon: Box::new(move || daemon.clone()),
    });
    let mut harness = Harness::builder()
        .with_size([980.0, 700.0])
        .build_ui_state(|ui, app: &mut SettingsApp| app.show(ui), app);
    harness.run();
    harness
}

fn search(h: &mut Harness<'static, SettingsApp>, text: &str) {
    h.get_by_role(Role::TextInput).click();
    h.run();
    h.get_by_role(Role::TextInput).type_text(text);
}

fn stopped() -> Harness<'static, SettingsApp> {
    harness(DaemonState::Stopped)
}

#[test]
fn starts_with_no_unsaved_changes() {
    let h = stopped();

    h.get_by_label("No unsaved changes");
    assert_eq!(h.state().edit(), &saved());
}

#[test]
fn toggling_a_setting_counts_one_unsaved_change() {
    let mut h = stopped();

    h.get_by_role_and_label(Role::Switch, "Write a log file")
        .click();
    h.run();

    assert!(h.state().edit().log_to_file);
    h.get_by_label("1 unsaved change");
}

#[test]
fn revert_drops_every_edit() {
    let mut h = stopped();
    h.get_by_role_and_label(Role::Switch, "Write a log file")
        .click();
    h.run();
    h.get_by_role_and_label(Role::Switch, "Start at sign-in")
        .click();
    h.run();
    h.get_by_label("2 unsaved changes");

    h.get_by_role_and_label(Role::Button, "Revert").click();
    h.run();

    assert_eq!(h.state().edit(), &saved());
    h.get_by_label("No unsaved changes");
}

#[test]
fn picking_a_flavor_previews_it() {
    let mut h = stopped();

    h.get_by_role_and_label(Role::RadioButton, "Latte").click();
    h.run();

    let latte = Theme::Catppuccin(CatppuccinFlavor::Latte);
    assert_eq!(h.state().edit().theme, latte);
    assert!(!h.ctx.global_style().visuals.dark_mode);
}

#[test]
fn search_shows_only_the_settings_that_match() {
    let mut h = stopped();

    search(&mut h, "rotated");
    h.run();

    assert!(h.query_by_label("Maximum file size").is_some());
    assert_eq!(h.query_all_by_label("Theme").count(), 0);
    assert_eq!(h.query_all_by_label("Startup").count(), 0);
}

#[test]
fn search_says_when_nothing_matches() {
    let mut h = stopped();

    search(&mut h, "zzz");
    h.run();

    h.get_by_label("No settings match \u{201c}zzz\u{201d}.");
}

#[test]
fn shows_whether_the_daemon_runs() {
    stopped().get_by_label("Daemon stopped");

    let running = harness(DaemonState::Running {
        version: "9.9.9".into(),
    });
    running.get_by_label("Daemon running");
    running.get_by_label("mosaico 9.9.9");
}
