//! Whether a window is an application window.

use super::Window;

impl Window {
    /// Returns whether this looks like a real application window.
    ///
    /// Checks for a caption bar (`WS_CAPTION`), rejects tool windows
    /// (`WS_EX_TOOLWINDOW`), rejects owned windows, and rejects modal
    /// dialog frames (`WS_EX_DLGMODALFRAME`).
    ///
    /// The modal-frame check is important because when a dialog is
    /// created on a different thread from its owner, the owner
    /// relationship may not be queryable at the time
    /// `EVENT_OBJECT_CREATE` fires. `WS_EX_DLGMODALFRAME` is set
    /// during `CreateWindowEx` itself, so it's always available.
    pub fn is_app_window(&self) -> bool {
        self.app_window_caption() == Some(true)
    }

    /// Like [`Self::is_app_window`], but without requiring `WS_CAPTION`.
    ///
    /// Returns `None` for tool windows, owned windows, and modal
    /// dialogs. Otherwise returns `Some(has_caption)`. Apps that draw
    /// their own title bar (e.g. winit or frameless windows) have no
    /// `WS_CAPTION` and come back as `Some(false)`; those are only
    /// tiled when a window rule explicitly opts them in.
    pub fn app_window_caption(&self) -> Option<bool> {
        use windows::Win32::UI::WindowsAndMessaging::{
            GW_OWNER, GWL_EXSTYLE, GWL_STYLE, GetWindow, GetWindowLongPtrW, WS_CAPTION,
            WS_EX_DLGMODALFRAME, WS_EX_TOOLWINDOW,
        };

        // SAFETY: GetWindowLongPtrW and GetWindow are read-only queries on a valid HWND.
        unsafe {
            let style = GetWindowLongPtrW(self.hwnd, GWL_STYLE) as u32;
            let ex_style = GetWindowLongPtrW(self.hwnd, GWL_EXSTYLE) as u32;

            let has_caption = (style & WS_CAPTION.0) == WS_CAPTION.0;
            let is_tool = (ex_style & WS_EX_TOOLWINDOW.0) == WS_EX_TOOLWINDOW.0;
            let is_dialog = (ex_style & WS_EX_DLGMODALFRAME.0) != 0;
            let has_owner = GetWindow(self.hwnd, GW_OWNER).is_ok_and(|owner| !owner.is_invalid());

            (!is_tool && !has_owner && !is_dialog).then_some(has_caption)
        }
    }
}
