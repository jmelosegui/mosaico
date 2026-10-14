//! Actions and window events.

use crate::{Action, WindowEvent};

use crate::WindowId;

use super::{Effect, Plan, Tiler};

impl Tiler {
    /// Executes a user-triggered action.
    pub(super) fn handle_action(&mut self, plan: &mut Plan, action: &Action) {
        match action {
            Action::Focus(dir) => self.focus_direction(plan, *dir),
            Action::Move(dir) => self.move_direction(plan, *dir),
            Action::Retile => self.retile_all(plan),
            Action::ToggleMonocle => self.toggle_monocle(plan),
            Action::CloseFocused => {
                if let Some(window) = self.focused_window {
                    plan.emit(Effect::Close { window });
                }
            }
            Action::MinimizeFocused => {
                // The minimize is reported back as a `Minimized` event,
                // which is what takes the window out of the layout.
                if let Some(window) = self.focused_window {
                    plan.emit(Effect::Minimize { window });
                }
            }
            Action::CycleLayout => self.cycle_layout(plan),
            Action::GoToWorkspace(n) => self.goto_workspace(plan, *n),
            Action::SendToWorkspace(n) => self.send_to_workspace(plan, *n),
            // Pausing is handled by the daemon before reaching here.
            Action::TogglePause => {}
        }
    }

    /// Handles a window event that is not an echo of our own effects.
    pub(super) fn handle_event(&mut self, plan: &mut Plan, event: &WindowEvent) {
        match *event {
            WindowEvent::Created { window } => self.on_created(plan, window),
            WindowEvent::Restored { window } => {
                if self.adopt_rejected.contains(&window) {
                    return;
                }
                if !self.is_tileable(plan, window) {
                    self.adopt_rejected.insert(window);
                    return;
                }
                // The window may already be back, for example when a
                // focus event beat this one during Alt+Tab.
                if self.find_window(window).is_some() {
                    self.focus_window(plan, window, false);
                    return;
                }
                self.add_and_focus(plan, window);
            }
            WindowEvent::Destroyed { window } => {
                self.adopt_rejected.remove(&window);
                self.reconnect.forget(window);
                self.hidden_how.forget(window);
                self.undecorate(plan, window);
                self.remove_from_tiling(plan, window, "del", false);
            }
            WindowEvent::Hidden { window } => {
                // Some applications report a hide while repainting a
                // surface. A real hide, such as closing to the tray,
                // leaves the window invisible.
                if plan.is_visible(window) {
                    crate::log_debug!("hide-ignored {} (still visible)", window);
                    return;
                }
                self.undecorate(plan, window);
                self.remove_from_tiling(plan, window, "hide", false);
            }
            WindowEvent::Minimized { window } => self.remove_from_tiling(plan, window, "min", true),
            WindowEvent::Moved { window } => {
                // A window the user just maximized stays maximized;
                // retiling it would snap it back into the layout.
                let maximized = plan.is_maximized(window);
                if self.focused_window == Some(window) {
                    self.focused_maximized = maximized;
                }
                if maximized {
                    self.update_border(plan);
                } else {
                    self.reassign_monitor(plan, window);
                }
            }
            WindowEvent::Focused { window } => self.on_focused(plan, window),
            WindowEvent::TitleChanged { window } => {
                // Some applications only become recognizable once their
                // title is set, and never report being created or shown.
                if self.find_window(window).is_none() {
                    self.try_adopt(plan, window);
                }
            }
            WindowEvent::MouseHover { window } => {
                if self.is_managed_on_active_workspace(window)
                    && self.focused_window != Some(window)
                {
                    self.focus_window(plan, window, true);
                }
            }
            WindowEvent::LocationChanged { window } => self.on_location_changed(plan, window),
            WindowEvent::DisplayChanged | WindowEvent::WorkAreaChanged => {}
        }
    }

    fn on_created(&mut self, plan: &mut Plan, window: WindowId) {
        if !plan.is_visible(window) {
            // Created but not shown yet. Moving it now means its first
            // frame is painted in the right place instead of wherever
            // the OS put it. It is not adopted: applications create many
            // hidden helper windows that are never shown, and those must
            // not reflow the layout. Adoption happens on the show event.
            if super::adopt::should_pre_place(
                self.find_window(window).is_some(),
                self.passes_tiling_rules(plan, window),
            ) {
                self.pre_place(plan, window);
            }
            return;
        }
        // Already visible when created, so there is no moment to place
        // it before it shows; adopt it straight away. Hiding it across
        // the move was measured at roughly 160ms invisible, a worse
        // artifact than the brief glimpse it removes. See #28.
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

    fn on_location_changed(&mut self, plan: &mut Plan, window: WindowId) {
        // Fires on every animation frame, so only react when the focused
        // window enters or leaves maximized state. That catches the
        // title bar buttons, which do not report a move.
        if self.focused_window != Some(window) {
            return;
        }
        let maximized = plan.is_maximized(window);
        if maximized == self.focused_maximized {
            return;
        }
        self.focused_maximized = maximized;
        if !maximized {
            // Back from maximized: retile so the window snaps into its
            // slot before the border reappears around it.
            self.reassign_monitor(plan, window);
        }
        self.update_border(plan);
    }

    /// Exits monocle mode on the active workspace of `mon_idx` and lays
    /// the workspace out again.
    pub(super) fn exit_monocle(&mut self, plan: &mut Plan, mon_idx: usize) {
        let ws = self.monitors[mon_idx].active_ws_mut();
        ws.set_monocle(false);
        ws.set_monocle_window(None);
        self.apply_layout_on(plan, mon_idx);
    }

    /// Removes a window from the layout, hands focus to a sibling if it
    /// had it, and retiles.
    ///
    /// With `active_only`, a window on an inactive workspace is left
    /// alone.
    pub(super) fn remove_from_tiling(
        &mut self,
        plan: &mut Plan,
        window: WindowId,
        reason: &str,
        active_only: bool,
    ) {
        let Some((mon_idx, ws_idx)) = self.find_window(window) else {
            return;
        };
        let on_active = ws_idx == self.monitors[mon_idx].active_workspace;
        if active_only && !on_active {
            return;
        }

        // Out of the layout, it is no longer the tiler's to show again,
        // and its id may go to a new window.
        self.hidden_how.forget(window);
        let ws = &mut self.monitors[mon_idx].workspaces[ws_idx];
        ws.remove(window);
        crate::log_info!(
            "-{} {} from mon {} ws {} (now {})",
            reason,
            window,
            mon_idx,
            ws_idx + 1,
            ws.len()
        );
        if ws.monocle() && ws.monocle_window() == Some(window) {
            ws.set_monocle(false);
            ws.set_monocle_window(None);
        }

        if self.focused_window == Some(window) {
            self.focused_window = None;
            // Promote a sibling so the border stays visible.
            if on_active {
                let ws = &self.monitors[mon_idx].workspaces[ws_idx];
                let replacement = ws
                    .last_focused()
                    .filter(|&w| ws.contains(w))
                    .or_else(|| ws.windows().first().copied());
                if let Some(next) = replacement {
                    self.focus_window(plan, next, false);
                }
            }
        }

        if on_active {
            self.apply_layout_on(plan, mon_idx);
        }
    }
}
