//! The settings window: a title bar, the navigation, the page and the
//! bar that applies or reverts the edits.

mod chrome;
mod close;
mod footer;
mod sidebar;

use eframe::egui::{CentralPanel, Context, Frame, Margin, Panel, ScrollArea, Ui};
use mosaico_core::config::{ConfigStore, Theme};

use crate::model::{self, Draft, Loaded, Settings};
use crate::pages::{self, FilePages, Page, PageContext};
use crate::theme::{self, Colors};

/// How often the daemon is asked whether it runs, in seconds.
const POLL_SECS: f64 = 2.0;

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
    log_file: String,
    daemon: Box<dyn FnMut() -> DaemonState>,
    daemon_state: DaemonState,
    polled_at: Option<f64>,
    styled: Option<Theme>,
    close: close::CloseGuard,
}

impl SettingsApp {
    /// A window showing the settings `services` reads, with no edits.
    pub fn new(services: Services) -> Self {
        let sign_in = services.start_at_sign_in;
        let loaded = match &services.store {
            Ok(store) => Loaded::read(store, sign_in),
            Err(e) => Loaded::blocked(e.clone(), sign_in),
        };
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
            page: Page::General,
            query: String::new(),
            log_file: services.log_file,
            daemon: services.daemon,
            daemon_state: DaemonState::Stopped,
            polled_at: None,
            styled: None,
            close: close::CloseGuard::default(),
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

        let close_clicked = Panel::top("title bar")
            .exact_size(36.0)
            .resizable(false)
            .frame(Frame::new().fill(c.title_bar))
            .show(ui, |ui| chrome::title_bar(ui, &c))
            .inner;
        self.close.handle(ui.ctx(), close_clicked, self.changes());
        Panel::left("navigation")
            .exact_size(224.0)
            .resizable(false)
            .frame(Frame::new().fill(c.sidebar).inner_margin(Margin {
                left: 8,
                right: 8,
                top: 16,
                bottom: 12,
            }))
            .show(ui, |ui| {
                sidebar::show(ui, &c, &mut self.page, &mut self.query, &self.daemon_state);
            });
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

    fn page(&mut self, ui: &mut Ui, c: &Colors) {
        ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            Frame::new()
                .inner_margin(Margin::symmetric(32, 24))
                .show(ui, |ui| {
                    let file = self.page.file();
                    pages::header(ui, c, self.page, file);
                    if let Some(page) = self.files.get(self.page) {
                        if let Some(problem) = page.problem() {
                            pages::problem(ui, c, file, problem);
                        }
                        let enabled = page.problem().is_none();
                        ui.add_enabled_ui(enabled, |ui| page.show(ui, c, &self.query));
                        return;
                    }
                    if let Some(problem) = &self.problem {
                        pages::problem(ui, c, file, problem);
                    }
                    let saved = self.draft.saved().clone();
                    let cx = PageContext {
                        c,
                        saved: &saved,
                        defaults: &self.defaults,
                        log_file: &self.log_file,
                    };
                    ui.add_enabled_ui(self.problem.is_none(), |ui| {
                        pages::show(ui, &cx, self.page, &self.query, &mut self.draft.edit);
                    });
                });
        });
    }

    /// Styles the window for the theme being edited, which previews it.
    fn style(&mut self, ctx: &Context) -> Colors {
        let theme = self.draft.edit.theme();
        let colors = Colors::of(theme);
        if self.styled != Some(theme) {
            if self.styled.is_none() {
                theme::install_fonts(ctx);
            }
            theme::apply(ctx, &colors);
            self.styled = Some(theme);
        }
        colors
    }

    fn poll(&mut self, ctx: &Context) {
        let now = ctx.input(|i| i.time);
        if self.polled_at.is_none_or(|at| now - at >= POLL_SECS) {
            self.daemon_state = (self.daemon)();
            self.polled_at = Some(now);
            // Without edits, the window follows the file, so what an
            // editor writes, or a fix to it, shows up.
            if self.draft.changes() == 0 {
                self.reload(self.draft.saved().start_at_sign_in);
            }
            self.files.reload_unedited();
        }
        ctx.request_repaint_after(std::time::Duration::from_secs_f64(POLL_SECS));
    }
}

impl eframe::App for SettingsApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }
}
