//! The settings window: a title bar, the navigation, the page and the
//! bar that applies or reverts the edits.

mod chrome;
mod footer;
mod sidebar;

use eframe::egui::{CentralPanel, Context, Frame, Margin, Panel, ScrollArea, Ui};
use mosaico_core::config::Theme;

use crate::model::{Draft, General};
use crate::pages::{self, Page, PageContext, general};
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

/// What the window reads from the system.
pub struct Services {
    /// The General settings as they are now.
    pub general: General,
    /// Where the log file is written, as shown to the user.
    pub log_file: String,
    /// Asks the daemon whether it runs. Called every few seconds, so it
    /// must return quickly.
    pub daemon: Box<dyn FnMut() -> DaemonState>,
}

/// The state of the settings window.
pub struct SettingsApp {
    draft: Draft<General>,
    defaults: General,
    page: Page,
    query: String,
    log_file: String,
    daemon: Box<dyn FnMut() -> DaemonState>,
    daemon_state: DaemonState,
    polled_at: Option<f64>,
    styled: Option<Theme>,
}

impl SettingsApp {
    /// A window showing `services.general`, with no edits.
    pub fn new(services: Services) -> Self {
        Self {
            draft: Draft::new(services.general),
            defaults: General::defaults(),
            page: Page::General,
            query: String::new(),
            log_file: services.log_file,
            daemon: services.daemon,
            daemon_state: DaemonState::Stopped,
            polled_at: None,
            styled: None,
        }
    }

    /// The settings with the edits made so far.
    pub fn edit(&self) -> &General {
        &self.draft.edit
    }

    /// Draws the window into `ui`, which should cover the whole viewport.
    pub fn show(&mut self, ui: &mut Ui) {
        let c = self.style(ui.ctx());
        self.poll(ui.ctx());

        Panel::top("title bar")
            .exact_size(36.0)
            .resizable(false)
            .frame(Frame::new().fill(c.title_bar))
            .show(ui, |ui| chrome::title_bar(ui, &c));
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
        Panel::bottom("footer")
            .exact_size(52.0)
            .resizable(false)
            .frame(
                Frame::new()
                    .fill(c.sidebar)
                    .inner_margin(Margin::symmetric(24, 0)),
            )
            .show(ui, |ui| footer::show(ui, &c, &mut self.draft));
        CentralPanel::default()
            .frame(Frame::new().fill(c.page))
            .show(ui, |ui| self.page(ui, &c));

        chrome::resize_edges(ui.ctx());
    }

    fn page(&mut self, ui: &mut Ui, c: &Colors) {
        ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            Frame::new()
                .inner_margin(Margin::symmetric(32, 24))
                .show(ui, |ui| {
                    pages::header(ui, c, self.page, general::FILE);
                    let saved = self.draft.saved().clone();
                    let cx = PageContext {
                        c,
                        saved: &saved,
                        defaults: &self.defaults,
                        log_file: &self.log_file,
                    };
                    general::show(ui, &cx, &self.query, &mut self.draft.edit);
                });
        });
    }

    /// Styles the window for the theme being edited, which previews it.
    fn style(&mut self, ctx: &Context) -> Colors {
        let theme = self.draft.edit.theme;
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
        }
        ctx.request_repaint_after(std::time::Duration::from_secs_f64(POLL_SECS));
    }
}

impl eframe::App for SettingsApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }
}
