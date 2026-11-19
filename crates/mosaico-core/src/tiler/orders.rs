//! Orders: what a client asks of a window, monitor or workspace it names,
//! instead of the focused one an [`Action`](crate::Action) works on.

use serde::{Deserialize, Serialize};

use crate::config::WorkspaceMode;
use crate::layout::LayoutKind;
use crate::{MonitorId, WindowId};

use super::{Effect, Plan, Tiler};

/// Something to do to a window, monitor or workspace named by its id.
/// Workspaces are counted from 0, as in the desktop snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "order", rename_all = "snake_case")]
pub enum Order {
    /// Focus the window and bring it into view.
    Focus {
        /// The window.
        window: WindowId,
    },
    /// Send a tiled window to another workspace of its monitor and follow
    /// it there, as `send-to-workspace` does for the focused window.
    Send {
        /// The window.
        window: WindowId,
        /// The workspace to send it to.
        workspace: usize,
    },
    /// Float or tile the window again. It is focused first.
    Float {
        /// The window.
        window: WindowId,
        /// Whether it should float.
        floating: bool,
    },
    /// Ask the window to close.
    Close {
        /// The window.
        window: WindowId,
    },
    /// Show a workspace on a monitor. In global mode every monitor
    /// switches with it, as with `goto-workspace`.
    Show {
        /// The monitor.
        monitor: MonitorId,
        /// The workspace to show.
        workspace: usize,
    },
    /// Tile a workspace with a layout, on screen or not.
    Layout {
        /// The monitor the workspace belongs to.
        monitor: MonitorId,
        /// The workspace.
        workspace: usize,
        /// The layout.
        layout: LayoutKind,
    },
    /// Turn monocle on or off on the workspace a monitor shows. The
    /// monitor gets focus, as monocle fills it with its focused window.
    Monocle {
        /// The monitor.
        monitor: MonitorId,
        /// Whether monocle should be on.
        on: bool,
    },
}

impl Tiler {
    /// Whether everything `order` names exists: a managed window, a
    /// connected monitor, a workspace it has.
    pub fn knows(&self, order: &Order) -> bool {
        match order {
            Order::Focus { window } | Order::Float { window, .. } | Order::Close { window } => {
                self.find_managed(*window).is_some()
            }
            Order::Send { window, workspace } => self
                .find_managed(*window)
                .is_some_and(|(mi, _)| *workspace < self.monitors[mi].workspaces.len()),
            Order::Show { monitor, workspace }
            | Order::Layout {
                monitor, workspace, ..
            } => self
                .monitor_index(*monitor)
                .is_some_and(|mi| *workspace < self.monitors[mi].workspaces.len()),
            Order::Monocle { monitor, .. } => self.monitor_index(*monitor).is_some(),
        }
    }

    fn monitor_index(&self, id: MonitorId) -> Option<usize> {
        self.monitors.iter().position(|m| m.id == id)
    }

    /// Carries out `order`. Does nothing for what it names that does not
    /// exist; [`Tiler::knows`] tells that first.
    pub(super) fn carry_out(&mut self, plan: &mut Plan, order: &Order) {
        match *order {
            Order::Focus { window } => self.raise(plan, window),
            Order::Send { window, workspace } => {
                self.raise(plan, window);
                if let Ok(n) = u8::try_from(workspace + 1) {
                    self.send_to_workspace(plan, n);
                }
            }
            Order::Float { window, floating } => {
                self.raise(plan, window);
                let floats = self.find_floating(window).is_some();
                if floats != floating {
                    self.toggle_float(plan);
                }
            }
            Order::Close { window } => {
                if self.find_managed(window).is_some() {
                    plan.emit(Effect::Close { window });
                }
            }
            Order::Show { monitor, workspace } => self.show(plan, monitor, workspace),
            Order::Layout {
                monitor,
                workspace,
                layout,
            } => self.set_layout(plan, monitor, workspace, layout),
            Order::Monocle { monitor, on } => {
                let Some(mi) = self.monitor_index(monitor) else {
                    return;
                };
                if self.monitors[mi].active_ws().monocle() == on {
                    return;
                }
                if mi != self.focused_monitor {
                    self.enter_monitor(plan, mi);
                }
                self.toggle_monocle(plan);
            }
        }
    }

    fn show(&mut self, plan: &mut Plan, monitor: MonitorId, workspace: usize) {
        let Some(mi) = self.monitor_index(monitor) else {
            return;
        };
        match self.workspace_mode {
            WorkspaceMode::PerMonitor => {
                let refocus = mi == self.focused_monitor;
                self.goto_workspace_on(plan, mi, workspace, refocus);
            }
            WorkspaceMode::Global => {
                let focused = self.focused_monitor;
                for each in 0..self.monitors.len() {
                    self.goto_workspace_on(plan, each, workspace, each == focused);
                }
            }
        }
    }

    fn set_layout(&mut self, plan: &mut Plan, monitor: MonitorId, ws: usize, layout: LayoutKind) {
        let Some(mi) = self.monitor_index(monitor) else {
            return;
        };
        let Some(target) = self.monitors[mi].workspaces.get_mut(ws) else {
            return;
        };
        if target.layout_kind() == layout {
            return;
        }
        target.set_layout_kind(layout);
        if self.monitors[mi].active_workspace == ws {
            self.apply_layout_on(plan, mi);
        }
    }
}
