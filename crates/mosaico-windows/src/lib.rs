/// Daemon main loop.
pub mod daemon;

/// Win32 window enumeration.
pub mod enumerate;

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
