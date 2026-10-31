//! How floating windows stack against the rest of their workspace: a
//! window that starts floating comes to the front, and a focused tile and
//! its border stay behind the floating windows.

use crate::WindowId;
use crate::workspace::Workspace;

use super::super::{Effect, Plan, Tiler};

/// The window a tile of `ws` goes behind to stay behind its floating
/// windows: the backmost of them, unless monocle hides them.
fn backmost_float(ws: &Workspace) -> Option<WindowId> {
    ws.floating().first().copied().filter(|_| !ws.monocle())
}

impl Tiler {
    /// The window the border of `window` on monitor `mi` goes behind
    /// instead of its own: the backmost floating window on screen there
    /// when `window` is the focused tile, which `keep_floats_in_front`
    /// puts behind them. Its border stays behind them too, before and
    /// after the tile lands there.
    pub(in crate::tiler) fn border_behind(&self, window: WindowId, mi: usize) -> Option<WindowId> {
        let ws = self.monitors.get(mi)?.active_ws();
        let tile = self.focused_window == Some(window) && ws.contains(window);
        backmost_float(ws).filter(|_| tile)
    }

    /// Keeps the floating windows of the workspace holding `window` above
    /// its tiles as `window` comes to the front: a floating window moves in
    /// front of the others, and a tile goes behind the backmost of them.
    pub(in crate::tiler) fn keep_floats_in_front(&mut self, plan: &mut Plan, window: WindowId) {
        let Some((mi, wi)) = self.find_managed(window) else {
            return;
        };
        let ws = &mut self.monitors[mi].workspaces[wi];
        if ws.is_floating(window) {
            ws.raise_floating(window);
        } else if let Some(other) = backmost_float(ws) {
            plan.emit(Effect::PutBehind { window, other });
        }
    }

    /// Brings floating `window` in front of the other floating windows of
    /// its workspace, and with `over_tiles` in front of its tiles too, by
    /// putting each of them behind the one before it, front first. Focus
    /// alone does not do it for a window that has focus already, which
    /// the platform leaves where it is.
    pub(in crate::tiler) fn float_to_front(
        &mut self,
        plan: &mut Plan,
        window: WindowId,
        over_tiles: bool,
    ) {
        let Some((mi, wi)) = self.find_floating(window) else {
            return;
        };
        let ws = &mut self.monitors[mi].workspaces[wi];
        ws.raise_floating(window);
        let others = ws.floating().iter().rev().skip(1);
        let tiles = ws.windows().iter().filter(|_| over_tiles);
        let mut front = window;
        for &other in others.chain(tiles) {
            plan.emit(Effect::PutBehind {
                window: other,
                other: front,
            });
            front = other;
        }
    }
}
