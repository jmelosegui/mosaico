//! Window events.

use crate::WindowEvent;

use crate::WindowId;

use super::{Plan, Tiler};

impl Tiler {
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
                self.filling.forget(window);
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
            WindowEvent::Minimized { window } => {
                // A minimize reported once the window was restored again
                // leaves it where it is.
                if !plan.is_minimized(window) {
                    crate::log_debug!("min-ignored {} (not minimized)", window);
                    return;
                }
                self.remove_from_tiling(plan, window, "min", true);
            }
            // Nothing in the layout changes until the drag ends, with
            // `Moved`.
            WindowEvent::MoveStarted { .. } => {}
            WindowEvent::Moved { window, .. } => {
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

    fn on_location_changed(&mut self, plan: &mut Plan, window: WindowId) {
        self.note_filling(plan, window);
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
}
