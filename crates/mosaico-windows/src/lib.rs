/// Win32 window enumeration.
pub mod enumerate;

/// Window type wrapping a Win32 `HWND`.
pub mod window;

pub use enumerate::enumerate_windows;
pub use window::Window;
