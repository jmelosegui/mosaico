use mosaico_core::Command;

use super::context::{Context, Exit};
use super::request;

/// Pauses every hotkey except toggle-pause.
pub fn pause(cx: &mut Context) -> Exit {
    request::run(cx, &Command::PauseHotkeys)
}

/// Resumes the paused hotkeys.
pub fn unpause(cx: &mut Context) -> Exit {
    request::run(cx, &Command::UnpauseHotkeys)
}
