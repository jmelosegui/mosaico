/// Daemon main loop.
pub mod daemon;

/// Win32 window enumeration.
pub mod enumerate;

/// IPC via Named Pipes.
pub mod ipc;

/// Process utilities (alive check).
pub mod process;

/// Window type wrapping a Win32 `HWND`.
pub mod window;

pub use enumerate::enumerate_windows;
pub use window::Window;
