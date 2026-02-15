/// Daemon main loop.
pub mod daemon;

/// DPI awareness setup.
pub mod dpi;

/// Win32 window enumeration.
pub mod enumerate;

/// Window frame and border offset helpers.
pub mod frame;

/// Win32 event translation.
pub mod event;

/// Win32 event loop (SetWinEventHook + message pump).
pub mod event_loop;

/// IPC via Named Pipes.
pub mod ipc;

/// Process utilities (alive check).
pub mod process;

/// Window type wrapping a Win32 `HWND`.
pub mod window;

pub use enumerate::enumerate_windows;
pub use window::Window;
