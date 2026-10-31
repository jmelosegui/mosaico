//! Opens the settings window of mosaico.

#![windows_subsystem = "windows"]

#[cfg(target_os = "windows")]
mod platform;

use eframe::egui::{ViewportBuilder, vec2};
use mosaico_settings::SettingsApp;

#[cfg(target_os = "windows")]
fn main() -> eframe::Result {
    let services = platform::services();
    let viewport = ViewportBuilder::default()
        .with_title("mosaico settings")
        .with_app_id("mosaico-settings")
        .with_inner_size(vec2(980.0, 700.0))
        .with_min_inner_size(vec2(720.0, 480.0))
        .with_decorations(false)
        .with_icon(icon());
    let options = eframe::NativeOptions {
        viewport,
        centered: true,
        ..Default::default()
    };
    eframe::run_native(
        "mosaico settings",
        options,
        Box::new(|_| Ok(Box::new(SettingsApp::new(services)))),
    )
}

#[cfg(target_os = "windows")]
fn icon() -> eframe::egui::IconData {
    let png = include_bytes!("../../../assets/png/mosaico-64.png");
    eframe::icon_data::from_png_bytes(png).unwrap_or_default()
}

#[cfg(not(target_os = "windows"))]
fn main() {
    eprintln!("mosaico settings is only available on Windows for now.");
    std::process::exit(1);
}
