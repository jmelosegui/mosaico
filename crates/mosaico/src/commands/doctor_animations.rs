//! What `mosaico doctor` says about window transitions: the
//! `[animations]` settings, whether the system lets windows be animated
//! now, and whether this desktop can move windows frame by frame.

use mosaico_core::config::{AnimationConfig, ConfigStore};
use mosaico_core::{Error, Result};

use super::context::{Context, say};

const OK: &str = "\x1b[32m[ok]\x1b[0m";
const WARN: &str = "\x1b[33m[warn]\x1b[0m";
const FAIL: &str = "\x1b[31m[fail]\x1b[0m";

/// The `[animations]` settings doctor reports, and where they come from.
#[derive(Debug, Clone, PartialEq)]
struct Settings {
    animations: AnimationConfig,
    /// Whether they are the defaults because `config.toml` is invalid.
    defaults_for_invalid_file: bool,
}

/// Reports the animation settings, and what the system and the desktop
/// say about them now. Asking for frames with none moves nothing.
pub fn check_animations(cx: &mut Context) {
    let settings = settings_in_use(&cx.config);
    let allowed = (cx.tiler_desktop)().animations_allowed();
    let frames = cx.windows.place_frames(&[], &mut Vec::new());
    for line in animation_lines(&settings, allowed, &frames) {
        say!(cx, "{line}");
    }
}

/// The `[animations]` settings the daemon starts with: those in
/// `config.toml`, or the defaults when the file is missing or invalid.
/// The config check reports an invalid file, so it is only named here.
fn settings_in_use(store: &ConfigStore) -> Settings {
    match store.try_load_config() {
        Ok(config) => Settings {
            animations: config.animations,
            defaults_for_invalid_file: false,
        },
        Err(_) => Settings {
            animations: AnimationConfig::default(),
            defaults_for_invalid_file: store.config_path().exists(),
        },
    }
}

/// What doctor says about window transitions with `settings`, while the
/// system allows animations when `allowed`, on a desktop that answered
/// `frames` to a call with no frames: whether windows move, the settings,
/// what the system says, and whether the desktop has frames.
fn animation_lines(settings: &Settings, allowed: bool, frames: &Result<()>) -> Vec<String> {
    let AnimationConfig {
        enabled,
        duration,
        easing,
        follow_system,
    } = &settings.animations;
    let values = format!(
        "enabled = {enabled}, duration = {duration} ms, easing = {easing}, \
         follow_system = {follow_system}"
    );
    let values = if settings.defaults_for_invalid_file {
        format!("The defaults, as config.toml is invalid: {values}")
    } else {
        values
    };
    let system = if allowed {
        "The system allows animations"
    } else {
        "The system turns animations off (its setting is off, or this is a remote session)"
    };
    let frames_line = match frames {
        Ok(()) => "       This desktop moves windows frame by frame".to_string(),
        Err(Error::Unsupported(_)) => {
            "       This desktop cannot move windows frame by frame".to_string()
        }
        Err(e) => format!("  {FAIL} Could not ask whether windows can move frame by frame: {e}"),
    };
    vec![
        status_line(settings, allowed, frames),
        format!("       {values}"),
        format!("       {system}"),
        frames_line,
    ]
}

/// Whether windows move to their new places, and when transitions are
/// enabled but windows snap anyway, why. A desktop that cannot be asked
/// about frames gives no reason: whether windows snap is then unknown.
fn status_line(settings: &Settings, allowed: bool, frames: &Result<()>) -> String {
    let animations = &settings.animations;
    if !animations.enabled {
        return format!("  {OK} Window transitions are off ([animations] enabled = false)");
    }
    let why = if !animations.is_on() {
        "they take no time (duration = 0)"
    } else if animations.follow_system && !allowed {
        "the system turns animations off (follow_system = true)"
    } else if let Err(Error::Unsupported(_)) = frames {
        "this desktop cannot move windows frame by frame"
    } else if frames.is_err() {
        return format!(
            "  {WARN} Window transitions are on, but whether windows can move frame by frame \
             is unknown"
        );
    } else {
        return format!("  {OK} Window transitions are on");
    };
    format!("  {WARN} Window transitions are on, but windows snap into place: {why}")
}

#[cfg(test)]
#[path = "tests/doctor_animations_tests.rs"]
mod tests;
