//! The settings window of mosaico.
//!
//! [`SettingsApp`] draws the window into any [`eframe::egui::Ui`], reading
//! the system only through the [`Services`] it is given, so tests can run
//! it headless with made-up settings and daemon.

mod app;
mod icons;
mod model;
mod pages;
mod theme;
mod widgets;

pub use app::{DaemonState, Services, SettingsApp};
pub use model::General;
