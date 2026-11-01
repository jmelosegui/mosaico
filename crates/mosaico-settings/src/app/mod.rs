//! The settings window: a title bar, the navigation, the page and the
//! bar that applies or reverts the edits.

mod chrome;
mod close;
mod footer;
mod page;
mod prefs;
mod refresh;
mod search;
mod sidebar;

use eframe::egui::{CentralPanel, Frame, Margin, Panel, Ui};
use mosaico_core::config::{ConfigStore, SettingsAppConfig, Theme};

use crate::model::{self, Draft, Loaded, Settings};
use crate::pages::{self, FilePages, Page};

pub use prefs::remembered;

/// Whether the daemon answers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DaemonState {
    /// The daemon answered, reporting its version.
    Running { version: String },
    /// Nothing answered.
    Stopped,
}

/// Turns starting at sign-in on or off, saying why it could not.
pub type SignIn = Box<dyn FnMut(bool) -> Result<(), String>>;

/// What the window reads from and writes to the system.
pub struct Services {
    /// The folder of the config files, or why it cannot be found.
    pub store: Result<ConfigStore, String>,
    /// Whether mosaico starts at sign-in now.
    pub start_at_sign_in: bool,
    /// Turns starting at sign-in on or off.
    pub sign_in: SignIn,
    /// Where the log file is written, as shown to the user.
    pub log_file: String,
    /// How many monitors are connected, left to right.
    pub monitors: usize,
    /// The installed font families, for the pickers of a font.
    pub fonts: Vec<String>,
    /// Asks the daemon whether it runs. Called every few seconds, so it
    /// must return quickly.
    pub daemon: Box<dyn FnMut() -> DaemonState>,
}

/// What became of the last Apply.
#[derive(Debug, Clone, PartialEq)]
pub enum Applied {
    /// Every edit was saved.
    Saved,
    /// Nothing, or not everything, was saved, for this reason.
    Failed(String),
}

/// The state of the settings window.
pub struct SettingsApp {
    draft: Draft<Settings>,
    files: FilePages,
    defaults: Settings,
    store: Result<ConfigStore, String>,
    /// Why the settings cannot be edited, when they cannot.
    problem: Option<String>,
    applied: Option<Applied>,
    sign_in: SignIn,
    page: Page,
    query: String,
    /// The page whose results a search scrolls to next.
    scroll_to: Option<Page>,
    /// Whether the next frame shows its content from the top.
    scroll_top: bool,
    log_file: String,
    daemon: Box<dyn FnMut() -> DaemonState>,
    daemon_state: DaemonState,
    polled_at: Option<f64>,
    styled: Option<Theme>,
    close: close::CloseGuard,
    remembered: SettingsAppConfig,
}

impl SettingsApp {
    /// A window showing the settings `services` reads, with no edits.
    pub fn new(services: Services) -> Self {
        let sign_in = services.start_at_sign_in;
        let loaded = match &services.store {
            Ok(store) => Loaded::read(store, sign_in),
            Err(e) => Loaded::blocked(e.clone(), sign_in),
        };
        let remembered = prefs::remembered(&services.store);
        Self {
            draft: Draft::new(loaded.settings),
            files: services
                .store
                .as_ref()
                .map(|store| pages::file_pages(store, services.monitors, &services.fonts))
                .unwrap_or_default(),
            defaults: Settings::defaults(),
            store: services.store,
            problem: loaded.problem,
            applied: None,
            sign_in: services.sign_in,
            page: prefs::page_named(&remembered.page),
            query: String::new(),
            scroll_to: None,
            scroll_top: false,
            log_file: services.log_file,
            daemon: services.daemon,
            daemon_state: DaemonState::Stopped,
            polled_at: None,
            styled: None,
            close: close::CloseGuard::default(),
            remembered,
        }
    }

    /// The settings with the edits made so far.
    pub fn edit(&self) -> &Settings {
        &self.draft.edit
    }

    /// The settings as they were loaded or last applied.
    pub fn saved(&self) -> &Settings {
        self.draft.saved()
    }

    /// What became of the last Apply, until the edits are reverted.
    pub fn applied(&self) -> Option<&Applied> {
        self.applied.as_ref()
    }

    /// How many edits are not saved yet, on every page.
    pub fn changes(&self) -> usize {
        self.draft.changes() + self.files.changes()
    }

    /// Saves the edits: each file's to the file, which the daemon
    /// reloads, and starting at sign-in through the system. The edits
    /// that could not be saved are kept.
    pub fn apply(&mut self) {
        let result = self.apply_config().and_then(|()| self.files.apply());
        self.applied = Some(match result {
            Ok(()) => Applied::Saved,
            Err(e) => Applied::Failed(e),
        });
    }

    /// Saves the edits of `config.toml` and starting at sign-in.
    fn apply_config(&mut self) -> Result<(), String> {
        if self.draft.changes() == 0 {
            return Ok(());
        }
        let store = self.store.as_ref().map_err(Clone::clone)?;
        if let Some(problem) = &self.problem {
            return Err(problem.clone());
        }
        let (saved, edit) = (self.draft.saved().clone(), self.draft.edit.clone());
        let written = model::save(store, &saved, &edit);
        let result = written.clone().and_then(|()| {
            if edit.start_at_sign_in == saved.start_at_sign_in {
                Ok(())
            } else {
                (self.sign_in)(edit.start_at_sign_in)
            }
        });
        if written.is_ok() {
            // When only sign-in failed, the file's edits are saved and
            // the sign-in edit stays.
            let sign_in = if result.is_ok() {
                edit.start_at_sign_in
            } else {
                saved.start_at_sign_in
            };
            self.reload(sign_in);
            self.draft.edit.start_at_sign_in = edit.start_at_sign_in;
        }
        result
    }

    /// Reads the settings again, dropping any edits.
    fn reload(&mut self, sign_in: bool) {
        let Ok(store) = &self.store else {
            return;
        };
        let loaded = Loaded::read(store, sign_in);
        self.problem = loaded.problem;
        if *self.draft.saved() != loaded.settings {
            self.draft = Draft::new(loaded.settings);
        }
    }

    /// Draws the window into `ui`, which should cover the whole viewport.
    pub fn show(&mut self, ui: &mut Ui) {
        let c = self.style(ui.ctx());
        self.poll(ui.ctx());
        self.track(ui.ctx());

        let close_clicked = Panel::top("title bar")
            .exact_size(36.0)
            .resizable(false)
            .frame(Frame::new().fill(c.title_bar))
            .show(ui, |ui| chrome::title_bar(ui, &c))
            .inner;
        self.close.handle(ui.ctx(), close_clicked, self.changes());
        let query = self.query.clone();
        let picked = Panel::left("navigation")
            .exact_size(224.0)
            .resizable(false)
            .frame(Frame::new().fill(c.sidebar).inner_margin(Margin {
                left: 8,
                right: 8,
                top: 16,
                bottom: 12,
            }))
            .show(ui, |ui| {
                let counts = self.searching().then(|| self.counts());
                let nav = sidebar::Nav {
                    page: self.page,
                    counts: counts.as_ref(),
                    daemon: &self.daemon_state,
                };
                sidebar::show(ui, &c, &nav, &mut self.query)
            })
            .inner;
        if let Some(page) = picked {
            self.page = page;
            self.scroll_to = self.searching().then_some(page);
        }
        self.scroll_top |= self.query != query || (picked.is_some() && self.scroll_to.is_none());
        let footer = Panel::bottom("footer")
            .exact_size(52.0)
            .resizable(false)
            .frame(
                Frame::new()
                    .fill(c.sidebar)
                    .inner_margin(Margin::symmetric(24, 0)),
            )
            .show(ui, |ui| {
                let state = footer::State {
                    changes: self.changes(),
                    applied: self.applied.as_ref(),
                };
                footer::show(ui, &c, &state)
            })
            .inner;
        match footer {
            Some(footer::Action::Apply) => self.apply(),
            Some(footer::Action::Revert) => {
                self.draft.revert();
                self.files.revert();
                self.applied = None;
            }
            None => {}
        }
        CentralPanel::default()
            .frame(Frame::new().fill(c.page))
            .show(ui, |ui| self.page(ui, &c));

        self.close.show(ui.ctx(), &c, self.changes());
        chrome::resize_edges(ui.ctx());
    }
}

impl eframe::App for SettingsApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.remember();
    }
}
