//! Runs the settings window headless, with made-up settings and daemon.

use eframe::egui::{Role, ViewportCommand, ViewportEvent, ViewportId};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use mosaico_core::config::{CatppuccinFlavor, Theme};
use mosaico_settings::{DaemonState, Forward, General, Services, SettingsApp};

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

/// The commands the last frame sent to the window.
fn commands(h: &Harness<'static, SettingsApp>) -> Vec<ViewportCommand> {
    h.output()
        .viewport_output
        .get(&ViewportId::ROOT)
        .map(|output| output.commands.clone())
        .unwrap_or_default()
}

/// A window with one unsaved edit whose close button was just clicked.
fn closing_with_an_edit() -> Harness<'static, SettingsApp> {
    let mut h = stopped();
    h.get_by_role_and_label(Role::Switch, "Write a log file")
        .click();
    h.run();
    h.get_by_role_and_label(Role::Button, "Close").click();
    h.step();
    h
}

#[test]
fn closing_with_no_edits_closes_at_once() {
    let mut h = stopped();

    h.get_by_role_and_label(Role::Button, "Close").click();
    h.step();

    assert!(commands(&h).contains(&ViewportCommand::Close));
    assert!(h.query_by_label("Discard unsaved changes?").is_none());
}

#[test]
fn closing_with_edits_asks_first() {
    let mut h = closing_with_an_edit();

    assert!(!commands(&h).contains(&ViewportCommand::Close));
    h.run();
    h.get_by_label("Discard unsaved changes?");
}

#[test]
fn keep_editing_keeps_the_window_and_the_edits() {
    let mut h = closing_with_an_edit();
    h.run();

    h.get_by_role_and_label(Role::Button, "Keep editing")
        .click();
    h.run();

    assert!(h.query_by_label("Discard unsaved changes?").is_none());
    assert!(h.state().edit().log_to_file);
    h.get_by_label("1 unsaved change");
}

#[test]
fn discarding_closes_the_window() {
    let mut h = closing_with_an_edit();
    h.run();

    h.get_by_role_and_label(Role::Button, "Discard and close")
        .click();
    h.step();

    assert!(commands(&h).contains(&ViewportCommand::Close));
}

#[test]
fn a_close_from_the_system_with_edits_is_held_back_to_ask() {
    let mut h = stopped();
    h.get_by_role_and_label(Role::Switch, "Write a log file")
        .click();
    h.run();

    let root = h.input_mut().viewports.entry(ViewportId::ROOT).or_default();
    root.events.push(ViewportEvent::Close);
    h.step();

    assert!(commands(&h).contains(&ViewportCommand::CancelClose));
    h.run();
    h.get_by_label("Discard unsaved changes?");
}
#[test]
fn a_summons_restores_and_focuses_the_window() {
    let mut h = stopped();
    let forward = Forward::default();
    forward.attach(&h.ctx);

    forward.bring();
    h.step();

    let sent = commands(&h);
    assert!(
        sent.contains(&ViewportCommand::Minimized(false)),
        "{sent:?}"
    );
    assert!(sent.contains(&ViewportCommand::Focus), "{sent:?}");
}
