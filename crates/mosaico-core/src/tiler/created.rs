//! Windows that were just created: moved into place before they are
//! shown, or taken in at once when they already are.

use crate::WindowId;

use super::{Effect, Plan, Tiler};

/// Whether a window that exists but has not been shown yet should be
/// moved into the slot it is going to occupy.
///
/// Monocle workspaces are included: `pre_place` asks `monocle_rect` for
/// them, so the destination is right either way.
pub(super) fn should_pre_place(tracked: bool, passes_rules: bool) -> bool {
    !tracked && passes_rules
}

impl Tiler {
    pub(super) fn on_created(&mut self, plan: &mut Plan, window: WindowId) {
        if !plan.is_visible(window) {
            // Created but not shown yet. Moving it now means its first
            // frame is painted in the right place instead of wherever
            // the OS put it. It is not adopted: applications create many
            // hidden helper windows that are never shown, and those must
            // not reflow the layout. Adoption happens on the show event.
            if should_pre_place(
                self.find_window(window).is_some(),
                self.passes_tiling_rules(plan, window),
            ) {
                self.pre_place(plan, window);
            }
            return;
        }
        // Already visible when created, so there is no moment to place
        // it before it shows; adopt it straight away. Hiding it across
        // the move does not help: the system can hide a window only
        // once it has registered it, which happens after the window is
        // already on screen and no earlier than the move lands. See #28.
        if !self.is_tileable(plan, window) {
            return;
        }
        // Applications built on some UI frameworks report a show for
        // child elements under the parent window. Without this the
        // window would be added to every workspace active at the time.
        if self.find_window(window).is_some() {
            return;
        }
        self.add_and_focus(plan, window);
    }

    /// Moves a window that has been created but not shown yet into the
    /// slot it would get, so its first frame is already in place.
    ///
    /// Nothing is adopted and the workspace is left exactly as it was:
    /// the destination comes from asking the layout where the window
    /// would go, and only that window is moved. It is taken over for
    /// real when it is shown.
    pub(super) fn pre_place(&mut self, plan: &mut Plan, window: WindowId) {
        let idx = self.focused_monitor;
        let Some(state) = self.monitors.get(idx) else {
            return;
        };

        let rect = if state.active_ws().monocle() {
            // A window adopted onto a monocle workspace becomes the
            // monocle target, which always fills the work area.
            self.monocle_rect(idx)
        } else {
            if !self.monitors[idx].active_ws_mut().add(window) {
                return;
            }
            let positions = self.compute_positions(idx);
            self.monitors[idx].active_ws_mut().remove(window);
            let Some((_, rect)) = positions.into_iter().find(|(w, _)| *w == window) else {
                return;
            };
            rect
        };
        crate::log_debug!(
            "~pre {} to mon {} ({},{} {}x{}) before it is shown",
            window,
            idx,
            rect.x,
            rect.y,
            rect.width,
            rect.height
        );
        plan.emit(Effect::PrePlace { window, rect });
    }
}
