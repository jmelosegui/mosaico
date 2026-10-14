//! Carries out the effects that draw borders and change window corners.
//!
//! The tiler decides how every border looks and which corners windows
//! get, so what is left when such an effect is applied is to ask the
//! platform, and to decide which failures are worth a warning.
//! [`apply_decoration_effect`] does both, the same way for every
//! platform, and leaves the drawing to [`Decorations`].

use crate::log::Level;
use crate::{Decorations, Error, WindowId};

use super::Effect;
use super::window_effects::report;

/// Carries out `effect` through `decorations` if it draws, hides or
/// drops a border, or changes the corners of a window.
///
/// Failures are logged rather than returned, as in
/// [`apply_window_effect`](super::apply_window_effect). A border that
/// could not be created, the only way drawing one fails, is a warning,
/// unless the platform does not draw borders at all. Other failures,
/// such as a window that closed in the meantime, are only logged for
/// debugging. Effects that change a window or move the pointer are left
/// to `apply_window_effect`.
pub fn apply_decoration_effect(effect: &Effect, decorations: &mut dyn Decorations) {
    match *effect {
        Effect::Border {
            window,
            style: Some(ref style),
        } => report_border(window, decorations.show_border(window, style)),
        Effect::Border {
            window,
            style: None,
        } => report(
            "hide the border of",
            window,
            decorations.hide_border(window),
        ),
        Effect::DropBorder { window } => report(
            "drop the border of",
            window,
            decorations.drop_border(window),
        ),
        Effect::Decorate { window, corners } => {
            report("decorate", window, decorations.set_corners(window, corners))
        }
        Effect::Undecorate { window } => {
            report("undecorate", window, decorations.reset_corners(window))
        }
        Effect::Place { .. }
        | Effect::PrePlace { .. }
        | Effect::Hide { .. }
        | Effect::Show { .. }
        | Effect::Reveal { .. }
        | Effect::Focus { .. }
        | Effect::CenterCursor { .. }
        | Effect::Close { .. }
        | Effect::Minimize { .. }
        | Effect::Repaint { .. } => {}
    }
}

/// Logs a border the platform could not draw.
fn report_border(window: WindowId, result: crate::Result<()>) {
    if let Err(e) = result {
        crate::log::write(
            border_failure_level(&e),
            format_args!("border {} failed: {}", window, e),
        );
    }
}

/// How loudly a border the platform could not draw is logged: a
/// warning, unless the platform does not draw borders at all, which it
/// says on every redraw.
pub(super) fn border_failure_level(error: &Error) -> Level {
    match error {
        Error::Unsupported(_) => Level::Debug,
        _ => Level::Warn,
    }
}
