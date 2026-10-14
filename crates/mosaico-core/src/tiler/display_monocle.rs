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
    /// behind first, so it never shows in front at full size.
    pub(super) fn place_waiting_behind(
        &mut self,
        plan: &mut Plan,
        monitor_idx: usize,
        front: WindowId,
        rect: Rect,
        timing: Timing,
    ) {
        let waiting = self.monitors[monitor_idx]
            .active_ws_mut()
            .take_waiting(front);
        for behind in waiting {
            plan.emit(Effect::PutBehind {
                window: behind,
                other: front,
            });
            plan.emit(Effect::Place {
                window: behind,
                rect,
                timing,
            });
        }
    }
}

/// Settles monocle on the workspaces a display change took the monocle
/// window from.
///
/// - One of the `returning` monitors whose monocle window did not come
///   back leaves monocle, as it always did.
/// - One whose monocle window `went_home` stays in monocle when windows
///   wait behind it, with the one in front of them as its monocle window.
///   Otherwise it keeps monocle with no monocle window if it is empty, as
///   it was before the window came, and leaves monocle if other windows
///   remain.
///
/// A stale monocle anywhere else is left for its window to come back to.
pub(super) fn clear_stale_monocle(
    monitors: &mut [MonitorState],
    returning: &[usize],
    went_home: &[WindowId],
) {
    for (mi, mon) in monitors.iter_mut().enumerate() {
        for ws in &mut mon.workspaces {
            let gone = ws.monocle_window().filter(|&w| !ws.contains(w));
            if !ws.monocle() {
                continue;
            }
            if returning.contains(&mi) {
                if !ws.monocle_window().is_some_and(|w| ws.contains(w)) {
                    ws.set_monocle(false);
                    ws.set_monocle_window(None);
                }
                continue;
            }
            if !gone.is_some_and(|w| went_home.contains(&w)) || ws.pass_monocle_on() {
                continue;
            }
            if !ws.is_empty() {
                ws.set_monocle(false);
            }
            ws.set_monocle_window(None);
        }
    }
}

/// Settles monocle on the workspaces that the windows in `joined`, which
/// came from another monitor, moved onto.
///
/// - One that is alone on a monocle workspace with no monocle window
///   becomes its monocle window, as on adoption.
/// - Any other waits behind the monocle window, with focus or not.
/// - A workspace's own monocle window coming home changes nothing.
pub(super) fn join_monocle(monitors: &mut [MonitorState], joined: &[WindowId]) {
    for &window in joined {
        let Some(ws) = monitors
            .iter_mut()
            .flat_map(|m| m.workspaces.iter_mut())
            .find(|ws| ws.contains(window))
        else {
            continue;
        };
        if !ws.monocle() || ws.monocle_window() == Some(window) {
            continue;
        }
        if ws.monocle_window().is_none() && ws.len() == 1 {
            ws.set_monocle_window(Some(window));
        } else {
            ws.wait_behind(window);
        }
    }
}
