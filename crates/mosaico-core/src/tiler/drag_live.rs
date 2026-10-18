//! The tiles that follow a drag while it goes on.
//!
//! While the tiler follows the drag of a tiled window and decides where it
//! lands, each report of where the window is lays out what dropping it
//! then would leave (`drag_live_layout.rs`): the workspace of its monitor
//! with the splits its held edges would move, for a resize, or that of
//! another monitor with the window put where it would join, for a move
//! over it. Each other window of that workspace is placed in its slot there
//! at once, never moved there over time, and only when the drag has not
//! put it there already, or, the first time, when it is not there. Nothing
//! is committed: the workspace keeps its windows and splits, and the
//! dragged window its slot, until the drop. A drop that lands as they were
//! laid out finds them in place, so only the window it drops settles into
//! its slot.
//!
//! The tiles the drag moved go back where their workspace has them, with
//! a transition if due, once it no longer lays their monitor out
//! (`drag_live_back.rs`): as it moves on to where a drop would leave them
//! alone, as when Escape puts the window back, and as the step ends that
//! interrupted the drag, ended it without laying that monitor out, or
//! started another. Any layout of the monitor meanwhile, as for a window
//! that opens there, puts them back too, until the next report. In
//! monocle, the window in front goes back to fill the work area, and the
//! others where they were before the drag moved them. The tiler stopping
//! leaves every window where it is.

use crate::{Rect, WindowId};

use super::super::{Effect, Plan, Tiler, Timing, Transition};

#[path = "drag_live_back.rs"]
mod back;
#[path = "drag_live_layout.rs"]
mod layout;

/// The windows of the workspace on screen of a monitor that a drag moved
/// live.
#[derive(Debug)]
pub(in crate::tiler) struct Live {
    /// The window dragged.
    window: WindowId,
    /// The monitor.
    monitor: usize,
    /// Each window moved, where it was before the drag first moved it, and
    /// where the drag put it last.
    moved: Vec<(WindowId, Rect, Rect)>,
}

impl Tiler {
    /// Lays out live what dropping the window the tiler follows the drag of
    /// would leave now, while the tiler decides where it lands: each other
    /// window of that layout is placed in its slot at once, unless it is
    /// there already. The windows it moved on a monitor it no longer lays
    /// out go back where their workspace has them.
    pub(in crate::tiler) fn follow_live(&mut self, plan: &mut Plan) {
        let Some(drag) = self.mouse.drag.clone().filter(|d| d.decides) else {
            return;
        };
        let laid = self.live_layout(plan, &drag);
        let elsewhere = |live: &Live| laid.as_ref().is_none_or(|(idx, _)| *idx != live.monitor);
        if let Some(left) = self.mouse.live.take_if(|live| elsewhere(live)) {
            self.put_back(plan, left);
        }
        let Some((idx, positions)) = laid else {
            return;
        };
        let mut live = self.mouse.live.take().unwrap_or(Live {
            window: drag.window,
            monitor: idx,
            moved: Vec::new(),
        });
        for (window, rect) in positions.into_iter().filter(|&(w, _)| w != drag.window) {
            let put = live.moved.iter().position(|&(w, ..)| w == window);
            let Some(at) = put.map(|i| live.moved[i].2).or_else(|| plan.rect(window)) else {
                continue;
            };
            if at == rect {
                continue;
            }
            match put {
                Some(i) => live.moved[i].2 = rect,
                None => live.moved.push((window, at, rect)),
            }
            plan.emit(Effect::Place {
                window,
                rect,
                timing: Timing::Now,
                transition: Transition::Snap,
            });
            self.show_border_for(plan, window, rect, idx);
        }
        self.mouse.live = (!live.moved.is_empty()).then_some(live);
    }

    /// Puts the windows a drag moved live back where their workspace has
    /// them, once the tiler no longer follows that drag deciding where its
    /// window lands. Runs as each step ends, so a monitor the step laid out
    /// already, as a drop onto it does, is not put back twice.
    pub(in crate::tiler) fn settle_live(&mut self, plan: &mut Plan) {
        let drag = self.mouse.drag.as_ref();
        let followed = |live: &Live| drag.is_some_and(|d| d.decides && d.window == live.window);
        if let Some(live) = self.mouse.live.take_if(|live| !followed(live)) {
            self.put_back(plan, live);
        }
    }

    /// Forgets what a drag laid out live without laying anything out, as the
    /// tiler stops and leaves every window where it is.
    pub(in crate::tiler) fn leave_live(&mut self) {
        self.mouse.live = None;
    }
}

#[cfg(test)]
#[path = "tests/drag_live_support.rs"]
mod support;

#[cfg(test)]
#[path = "tests/drag_live_resize_tests.rs"]
mod resize_tests;

#[cfg(test)]
#[path = "tests/drag_live_resize_end_tests.rs"]
mod resize_end_tests;

#[cfg(test)]
#[path = "tests/drag_live_resize_cases_tests.rs"]
mod resize_cases_tests;
