//! What the tests share: a window on a config folder of its own, with
//! a made-up daemon and sign-in.

pub use std::cell::RefCell;
pub use std::path::PathBuf;
pub use std::rc::Rc;

pub use eframe::egui::{Role, ViewportCommand, ViewportEvent, ViewportId};
pub use egui_kittest::Harness;
pub use egui_kittest::kittest::{NodeT, Queryable};
pub use mosaico_core::config::schema::SettingValue;
pub use mosaico_core::config::{CatppuccinFlavor, ConfigStore, Theme};
pub use mosaico_settings::{Applied, DaemonState, Forward, Services, Settings, SettingsApp};

pub fn saved() -> Settings {
    Settings::defaults()
}

/// A config folder of its own for `test`, emptied first.
pub fn folder(test: &str) -> ConfigStore {
    let dir: PathBuf = std::env::temp_dir()
        .join("mosaico-settings-app-tests")
        .join(format!("{test}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    ConfigStore::new(dir)
}

/// What the window asked sign-in to be turned to, and whether it fails.
#[derive(Clone, Default)]
pub struct SignIn {
    pub asked: Rc<RefCell<Vec<bool>>>,
    pub fails: bool,
}

pub fn window(
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
        monitors: 2,
        fonts: vec!["Cascadia Code".into(), "Segoe UI".into()],
        daemon: Box::new(move || daemon.clone()),
    });
    let mut harness = Harness::builder()
        .with_size([980.0, 700.0])
        .build_ui_state(|ui, app: &mut SettingsApp| app.show(ui), app);
    harness.run();
    harness
}

pub fn harness(daemon: DaemonState) -> Harness<'static, SettingsApp> {
    let test = std::thread::current()
        .name()
        .unwrap_or("test")
        .replace("::", "-");
    window(folder(&test), daemon, &SignIn::default())
}

pub fn search(h: &mut Harness<'static, SettingsApp>, text: &str) {
    h.get_by_role_and_label(Role::TextInput, "Search settings")
        .click();
    h.run();
    h.get_by_role_and_label(Role::TextInput, "Search settings")
        .type_text(text);
}

pub fn stopped() -> Harness<'static, SettingsApp> {
    harness(DaemonState::Stopped)
}

pub fn logging<'a>(h: &'a Harness<'static, SettingsApp>) -> &'a SettingValue {
    h.state().edit().get("logging.enabled")
}
#[test]
fn starts_with_no_unsaved_changes() {
    let h = stopped();

    h.get_by_label("No unsaved changes");
    assert_eq!(h.state().edit(), &saved());
}

pub fn click(h: &mut Harness<'static, SettingsApp>, role: Role, label: &str) {
    h.get_by_role_and_label(role, label).click();
    h.run();
}

pub fn open(h: &mut Harness<'static, SettingsApp>, page: &str) {
    click(h, Role::Tab, page);
}
