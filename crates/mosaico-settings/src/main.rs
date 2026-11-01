//! Opens the settings window of mosaico.

#![windows_subsystem = "windows"]

#[cfg(target_os = "windows")]
mod platform;

#[cfg(target_os = "windows")]
use eframe::egui::{ViewportBuilder, vec2};
#[cfg(target_os = "windows")]
use mosaico_core::config::SETTINGS_WINDOW_MIN;
#[cfg(target_os = "windows")]
use mosaico_settings::{Forward, SettingsApp, remembered};

#[cfg(target_os = "windows")]
fn main() -> eframe::Result {
    // A second launch brings the open window forward and leaves. When the
    // lock cannot be taken at all, the window opens without it.
    let forward = Forward::default();
    let summoned = forward.clone();
    let _lock = match platform::lock_window(Box::new(move || summoned.bring())) {
        Ok(lock) => Some(lock),
        Err(mosaico_core::Error::AlreadyRunning) => return Ok(()),
        Err(_) => None,
    };
    let services = platform::services();
    // The place is not remembered: the window opens centered, on a
    // monitor that is surely connected.
    let prefs = remembered(&services.store);
    let (width, height) = prefs.size();
    let viewport = ViewportBuilder::default()
        .with_title("mosaico settings")
        .with_app_id("mosaico-settings")
        .with_inner_size(vec2(width, height))
        .with_min_inner_size(vec2(SETTINGS_WINDOW_MIN.0, SETTINGS_WINDOW_MIN.1))
        .with_maximized(prefs.maximized)
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
        Box::new(move |cc| {
            forward.attach(&cc.egui_ctx);
            Ok(Box::new(SettingsApp::new(services)))
        }),
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
