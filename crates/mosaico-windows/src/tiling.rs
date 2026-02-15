use mosaico_core::window::Window as WindowTrait;
use mosaico_core::{Action, BspLayout, Rect, WindowEvent, WindowResult, Workspace};

use crate::enumerate;
use crate::monitor;
use crate::window::Window;

/// Manages tiled windows on the primary monitor.
///
/// Holds the workspace (window list), layout algorithm, and cached
/// work area. Processes window events and actions.
pub struct TilingManager {
    workspace: Workspace,
    layout: BspLayout,
    work_area: Rect,
    /// The currently focused window handle, if any.
    focused: Option<usize>,
}

impl TilingManager {
    /// Creates a new tiling manager and populates it with existing windows.
    ///
    /// Uses the provided layout for window positioning. Call with
    /// `BspLayout::default()` if no configuration is loaded.
    pub fn new(layout: BspLayout) -> WindowResult<Self> {
        let work_area = monitor::primary_work_area()?;
        let mut manager = Self {
            workspace: Workspace::new(),
            layout,
            work_area,
            focused: None,
        };

        // Add all currently visible windows.
        for win in enumerate::enumerate_windows()? {
            manager.workspace.add(win.hwnd().0 as usize);
        }

        manager.apply_layout();
        Ok(manager)
    }

    /// Handles a window event and re-tiles if needed.
    pub fn handle_event(&mut self, event: &WindowEvent) {
        let changed = match event {
            WindowEvent::Created { hwnd } => {
                let window = Window::from_raw(*hwnd);
                if is_tileable(&window) {
                    self.workspace.add(*hwnd)
                } else {
                    false
                }
            }
            WindowEvent::Destroyed { hwnd } => self.workspace.remove(*hwnd),
            WindowEvent::Minimized { hwnd } => self.workspace.remove(*hwnd),
            WindowEvent::Restored { hwnd } => {
                let window = Window::from_raw(*hwnd);
                if is_tileable(&window) {
                    self.workspace.add(*hwnd)
                } else {
                    false
                }
            }
            WindowEvent::Focused { hwnd } => {
                self.focused = Some(*hwnd);
                false
            }
            _ => false,
        };

        if changed {
            self.apply_layout();
        }
    }

    /// Executes a user-triggered action.
    pub fn handle_action(&mut self, action: &Action) {
        match action {
            Action::FocusNext => self.focus_adjacent(1),
            Action::FocusPrev => self.focus_adjacent(-1),
            Action::SwapNext => self.swap_adjacent(1),
            Action::SwapPrev => self.swap_adjacent(-1),
            Action::Retile => self.apply_layout(),
        }
    }

    /// Moves focus to an adjacent window in the workspace.
    fn focus_adjacent(&mut self, direction: i32) {
        let Some(idx) = self.focused_index() else {
            return;
        };

        let len = self.workspace.len() as i32;
        let next = ((idx as i32 + direction).rem_euclid(len)) as usize;

        if let Some(&hwnd) = self.workspace.handles().get(next) {
            self.focused = Some(hwnd);
            set_foreground(hwnd);
        }
    }

    /// Swaps the focused window with an adjacent one and re-tiles.
    fn swap_adjacent(&mut self, direction: i32) {
        let Some(idx) = self.focused_index() else {
            return;
        };

        let len = self.workspace.len() as i32;
        let other = ((idx as i32 + direction).rem_euclid(len)) as usize;

        self.workspace.swap(idx, other);
        self.apply_layout();
    }

    /// Returns the workspace index of the currently focused window.
    fn focused_index(&self) -> Option<usize> {
        self.focused.and_then(|hwnd| self.workspace.index_of(hwnd))
    }

    /// Applies the current layout to all managed windows.
    fn apply_layout(&self) {
        let positions = self.workspace.compute_layout(&self.layout, &self.work_area);

        for (hwnd, rect) in &positions {
            let window = Window::from_raw(*hwnd);
            if let Err(e) = window.set_rect(rect) {
                eprintln!("Failed to position window 0x{hwnd:X}: {e}");
            }
        }
    }

    /// Returns the number of managed windows.
    pub fn window_count(&self) -> usize {
        self.workspace.len()
    }
}

/// Determines whether a window should be tiled.
fn is_tileable(window: &Window) -> bool {
    window.is_visible()
}

/// Sets the given window as the foreground (focused) window.
fn set_foreground(hwnd: usize) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow;

    let hwnd = HWND(hwnd as *mut _);
    // SAFETY: SetForegroundWindow is safe to call with a valid HWND.
    unsafe {
        let _ = SetForegroundWindow(hwnd);
    }
}
