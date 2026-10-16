//! Carries out the effects that change windows.
//!
//! The platform applies every effect the tiler returns, but most of
//! them only change a window or move the pointer, and the decisions
//! they still need at that point (leaving a window that is already in
//! place alone, finding where to put the pointer) do not depend on the
//! platform. [`apply_window_effect`] makes those decisions against the
//! desktop as it is right then and leaves the changes to
//! [`WindowControl`].

use crate::{Rect, WindowControl, WindowId};

use super::{Desktop, Effect};

/// Carries out `effect` through `windows` if it changes a window or
/// moves the pointer, reading what it needs from `desktop`.
///
/// Failures are logged rather than returned: an effect that cannot be
/// applied, for example because its window closed in the meantime, must
/// not hold up the ones after it. Borders and frame decorations are not
/// window changes, so their effects are left to the caller.
pub fn apply_window_effect(effect: &Effect, desktop: &dyn Desktop, windows: &dyn WindowControl) {
    match *effect {
        Effect::Place { window, rect, .. } => place(window, rect, desktop, windows),
        Effect::PrePlace { window, rect } => {
            if let Err(e) = windows.pre_place(window, rect) {
                crate::log_debug!("~pre {} failed: {}", window, e);
            }
        }
        Effect::Hide { window, how } => report("hide", window, windows.hide(window, how)),
        Effect::Show { window, how } => report("show", window, windows.show(window, how)),
        Effect::Reveal { window } => report("reveal", window, windows.reveal(window)),
        Effect::Focus {
            window,
            move_cursor,
        } => {
            report("focus", window, windows.focus(window));
            if move_cursor {
                center_cursor(window, desktop, windows);
            }
        }
        Effect::CenterCursor { window } => center_cursor(window, desktop, windows),
        Effect::Close { window } => report("close", window, windows.close(window)),
        Effect::Minimize { window } => report("minimize", window, windows.minimize(window)),
        Effect::Repaint { window } => report("repaint", window, windows.repaint(window)),
        Effect::Decorate { .. }
        | Effect::Undecorate { .. }
        | Effect::Border { .. }
        | Effect::DropBorder { .. } => {}
    }
}

/// Moves a window into its slot and repaints it, unless it is already
/// there.
///
/// Maximized windows are always placed: placing one clears its
/// maximized state, which leaving it alone would not. The repaint
/// follows even when the move fails, as part of it may have landed.
fn place(window: WindowId, rect: Rect, desktop: &dyn Desktop, windows: &dyn WindowControl) {
    if !desktop.is_maximized(window) && desktop.rect(window) == Some(rect) {
        return;
    }
    if let Err(e) = windows.place(window, rect) {
        crate::log_warn!("Failed to place {}: {}", window, e);
    }
    report("repaint", window, windows.repaint(window));
}

/// Puts the pointer at the center of a window, unless the window is
/// gone.
fn center_cursor(window: WindowId, desktop: &dyn Desktop, windows: &dyn WindowControl) {
    let Some(rect) = desktop.rect(window) else {
        return;
    };
    report(
        "center the pointer on",
        window,
        windows.move_cursor(rect.center_x(), rect.center_y()),
    );
}

/// Logs a change the platform could not make. Most are expected now
/// and then, such as a window that closed before its effect was
/// applied, so they are only logged for debugging.
fn report(what: &str, window: WindowId, result: crate::Result<()>) {
    if let Err(e) = result {
        crate::log_debug!("{} {} failed: {}", what, window, e);
    }
}
