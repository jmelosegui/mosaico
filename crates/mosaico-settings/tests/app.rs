//! Runs the settings window headless, on a config folder of its own and
//! with a made-up daemon and sign-in.

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use eframe::egui::{Role, ViewportCommand, ViewportEvent, ViewportId};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use mosaico_core::config::schema::SettingValue;
use mosaico_core::config::{CatppuccinFlavor, ConfigStore, Theme};
use mosaico_settings::{Applied, DaemonState, Forward, Services, Settings, SettingsApp};

fn saved() -> Settings {
    Settings::defaults()
}

/// A config folder of its own for `test`, emptied first.
fn folder(test: &str) -> ConfigStore {
    let dir: PathBuf = std::env::temp_dir()
        .join("mosaico-settings-app-tests")
        .join(format!("{test}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    ConfigStore::new(dir)
}

/// What the window asked sign-in to be turned to, and whether it fails.
#[derive(Clone, Default)]
struct SignIn {
    asked: Rc<RefCell<Vec<bool>>>,
    fails: bool,
}

fn window(
    store: ConfigStore,
    daemon: DaemonState,
    sign_in: &SignIn,
) -> Harness<'static, SettingsApp> {
    let sign_in = sign_in.clone();
    let app = SettingsApp::new(Services {
        store: Ok(store),
        start_at_sign_in: false,
        sign_in: Box::new(move |on| {
            sign_in.asked.borrow_mut().push(on);
            if sign_in.fails {
                Err("the system said no".into())
            } else {
                Ok(())
            }
        }),
        log_file: "logs/mosaico.log".into(),
        daemon: Box::new(move || daemon.clone()),
    });
    let mut harness = Harness::builder()
        .with_size([980.0, 700.0])
        .build_ui_state(|ui, app: &mut SettingsApp| app.show(ui), app);
    harness.run();
    harness
}

fn harness(daemon: DaemonState) -> Harness<'static, SettingsApp> {
    let test = std::thread::current()
        .name()
        .unwrap_or("test")
        .replace("::", "-");
    window(folder(&test), daemon, &SignIn::default())
}

fn search(h: &mut Harness<'static, SettingsApp>, text: &str) {
    h.get_by_role(Role::TextInput).click();
    h.run();
    h.get_by_role(Role::TextInput).type_text(text);
}

fn stopped() -> Harness<'static, SettingsApp> {
    harness(DaemonState::Stopped)
}

fn logging<'a>(h: &'a Harness<'static, SettingsApp>) -> &'a SettingValue {
    h.state().edit().get("logging.enabled")
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

    assert_eq!(logging(&h), &SettingValue::Bool(true));
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
    assert_eq!(h.state().edit().theme(), latte);
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
    assert_eq!(logging(&h), &SettingValue::Bool(true));
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

fn click(h: &mut Harness<'static, SettingsApp>, role: Role, label: &str) {
    h.get_by_role_and_label(role, label).click();
    h.run();
}

#[test]
fn apply_writes_the_edits_to_config_toml_and_keeps_its_comments() {
    // Arrange
    let store = folder("apply-writes");
    let file = "# my setup\n[logging]\nlevel = \"warn\" # quieter\n";
    std::fs::write(store.config_path(), file).unwrap();
    let path = store.config_path();
    let mut h = window(store, DaemonState::Stopped, &SignIn::default());
    click(&mut h, Role::Switch, "Write a log file");

    // Act
    click(&mut h, Role::Button, "Apply");

    // Assert
    let written = std::fs::read_to_string(path).unwrap();
    assert_eq!(written, format!("{file}enabled = true\n"));
    assert_eq!(h.state().applied(), Some(&Applied::Saved));
    assert_eq!(h.state().saved(), h.state().edit());
    h.get_by_label("Changes saved");
}

#[test]
fn apply_turns_start_at_sign_in_on_through_the_system() {
    let sign_in = SignIn::default();
    let mut h = window(folder("apply-sign-in"), DaemonState::Stopped, &sign_in);
    click(&mut h, Role::Switch, "Start at sign-in");

    click(&mut h, Role::Button, "Apply");

    assert_eq!(*sign_in.asked.borrow(), [true]);
    assert!(h.state().saved().start_at_sign_in);
    h.get_by_label("Changes saved");
}

#[test]
fn a_failed_apply_says_why_and_keeps_the_edit_that_failed() {
    // Arrange
    let store = folder("apply-fails");
    let path = store.config_path();
    let sign_in = SignIn {
        fails: true,
        ..SignIn::default()
    };
    let mut h = window(store, DaemonState::Stopped, &sign_in);
    click(&mut h, Role::Switch, "Write a log file");
    click(&mut h, Role::Switch, "Start at sign-in");

    // Act
    click(&mut h, Role::Button, "Apply");

    // Assert
    h.get_by_label("Could not save: the system said no");
    let written = std::fs::read_to_string(path).unwrap();
    assert_eq!(written, "[logging]\nenabled = true\n");
    assert!(!h.state().saved().start_at_sign_in);
    assert!(h.state().edit().start_at_sign_in);
}

#[test]
fn a_file_the_daemon_would_not_load_is_shown_and_not_saved_over() {
    // Arrange
    let store = folder("apply-broken");
    let broken = "[animations]\neasing = \"wobble\"\n";
    std::fs::write(store.config_path(), broken).unwrap();
    let path = store.config_path();

    // Act
    let h = window(store, DaemonState::Stopped, &SignIn::default());

    // Assert
    h.get_by_label("config.toml has a problem, so it can't be edited here");
    assert!(
        h.get_by_role_and_label(Role::Button, "Apply")
            .accesskit_node()
            .is_disabled()
    );
    assert_eq!(h.state().edit(), &saved());
    assert_eq!(std::fs::read_to_string(path).unwrap(), broken);
}

#[test]
fn an_edit_made_in_an_editor_shows_up_while_nothing_is_unsaved() {
    // Arrange
    let store = folder("apply-outside-edit");
    let path = store.config_path();
    let mut h = window(store, DaemonState::Stopped, &SignIn::default());

    // Act
    std::fs::write(path, "[logging]\nenabled = true\n").unwrap();
    for _ in 0..20 {
        h.step();
    }

    // Assert
    assert_eq!(logging(&h), &SettingValue::Bool(true));
    assert_eq!(h.state().saved(), h.state().edit());
}
