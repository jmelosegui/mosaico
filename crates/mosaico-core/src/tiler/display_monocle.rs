//! Monocle workspaces across display changes (#132).
//!
//! A window a display change brings onto a monocle workspace from another
//! monitor, parked on the host or sent back home, would keep its old
//! place. It waits behind the monocle window instead, whether it held
//! focus or not: the layout puts it behind and in the monocle rect the
//! next time the workspace is laid out in monocle, and one that held
//! focus hands it to the monocle window (`Tiler::settle_focus`). The
//! workspace keeps what waits, so a window that leaves it leaves nothing
//! waiting. A visit leaves the monocle window, monocle and the stacking
//! as they were.
//!
//! A window that is always on top cannot wait behind a monocle window
//! that is not: `put_behind` leaves it in front, so it shows at the
//! monocle rect over the monocle window, which keeps focus.

use crate::{Rect, WindowId};

use super::{Effect, MonitorState, Plan, Tiler, Timing};

impl Tiler {
    /// Puts the windows waiting on the monitor's monocle workspace behind
    /// its monocle window `front`, then at its `rect`, once. Each is put
    /// behind first, so it never shows in front at full size. The caller
    /// places `front` first, so a waiting window never shows there alone.
    pub(super) fn place_waiting_behind(
        &mut self,
        plan: &mut Plan,
        monitor_idx: usize,
        front: WindowId,
        rect: Rect,
        timing: Timing,
    ) {
        let state = &mut self.monitors[monitor_idx];
        let waiting = state.active_ws_mut().take_waiting(front);
        for behind in waiting {
            plan.emit(Effect::PutBehind {
                window: behind,
                other: front,
            });
            plan.place(behind, rect, timing, state.id);
        }
    }

    /// Puts the windows in `waiting` that were just `shown` behind their
    /// monocle window again. They were put behind it before they were
    /// shown, while still hidden, and showing a window that was hidden may
    /// bring it to the front.
    pub(super) fn put_shown_behind(
        &self,
        plan: &mut Plan,
        waiting: &[WindowId],
        shown: &[WindowId],
    ) {
        for &window in waiting.iter().filter(|w| shown.contains(w)) {
            let Some((mi, wi)) = self.find_window(window) else {
                continue;
            };
            let ws = &self.monitors[mi].workspaces[wi];
            if let Some(other) = ws
                .monocle_window()
                .filter(|&front| front != window && ws.contains(front))
            {
                plan.emit(Effect::PutBehind { window, other });
            }
        }
    }
}

/// Turns monocle off on the workspaces of a returning monitor whose
/// monocle window did not come back, as it always did.
pub(super) fn clear_stale_monocle(monitor: &mut MonitorState) {
    for ws in &mut monitor.workspaces {
        if ws.monocle() && !ws.monocle_window().is_some_and(|w| ws.contains(w)) {
            ws.set_monocle(false);
            ws.set_monocle_window(None);
        }
    }
}

/// Settles monocle on the workspaces that the windows in `joined`, which
/// came from another monitor, moved onto, and on those whose monocle
/// window `went_home`, once all of them have joined. Returns the windows
/// that joined and wait behind a monocle window.
///
/// - A workspace whose monocle window is set but not on it, because the
///   user sent or dragged it away, is tiled until it comes back, so
///   nothing waits there.
/// - Anywhere else in monocle each window waits behind the monocle
///   window, with focus or not. A workspace's own monocle window coming
///   home changes nothing.
/// - A workspace whose monocle window went home, or an empty one with no
///   monocle window, takes one of the windows that wait as its monocle
///   window: the `focused` one, else the one put behind last, which is in
///   front of the others, else the first. Without one, a workspace whose
///   monocle window went home keeps monocle with no monocle window if it
///   is empty, as it was before the window came, and leaves monocle if
///   other windows remain.
pub(super) fn join_monocle(
    monitors: &mut [MonitorState],
    joined: &[WindowId],
    went_home: &[WindowId],
    focused: Option<WindowId>,
) -> Vec<WindowId> {
    let mut waiting = Vec::new();
    for ws in monitors.iter_mut().flat_map(|m| m.workspaces.iter_mut()) {
        let monocle_window = ws.monocle_window();
        let away = monocle_window.filter(|&w| !ws.contains(w));
        let gone_home = away.is_some_and(|w| went_home.contains(&w));
        if !ws.monocle() || (away.is_some() && !gone_home) {
            continue;
        }
        let newcomers: Vec<WindowId> = ws
            .windows()
            .iter()
            .copied()
            .filter(|&w| joined.contains(&w) && Some(w) != monocle_window)
            .collect();
        let was_empty = monocle_window.is_none() && newcomers.len() == ws.len();
        for &window in &newcomers {
            ws.wait_behind(window);
        }
        if gone_home || (was_empty && !newcomers.is_empty()) {
            if let Some(window) = focused.filter(|&w| ws.waits_behind(w)) {
                ws.hand_monocle_to(window);
            } else if !ws.pass_monocle_on() && gone_home {
                if !ws.is_empty() {
                    ws.set_monocle(false);
                }
                ws.set_monocle_window(None);
            }
        }
        waiting.extend(newcomers.into_iter().filter(|&w| ws.waits_behind(w)));
    }
    waiting
}
