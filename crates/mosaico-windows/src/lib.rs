#![deny(unsafe_op_in_unsafe_fn)]
#![warn(clippy::undocumented_unsafe_blocks)]

/// Windows startup registration via the HKCU Run registry key.
pub mod autostart;

/// COM-based window cloaking via the ImmersiveShell.
mod com;

/// Status bar overlay windows.
pub mod bar;

/// Manages bar instances across all monitors.
pub mod bar_manager;

/// Border overlay windows for visual focus indicators.
pub mod border;

/// Downloads community rules from the mosaico-rules repository.
pub mod community_rules;

/// Config file watcher for hot-reloading.
pub mod config_watcher;

/// Ctrl+C signal handler using SetConsoleCtrlHandler.
pub mod ctrl_c;

/// Daemon main loop.
pub mod daemon;

/// Focus borders and window corners through Win32.
mod decorations;

/// DPI awareness setup.
pub mod dpi;

/// Window and monitor listing for the core `Enumerator` contract.
pub mod enumerate;

/// Wrapping native errors in the core error.
mod error;

/// Window frame and border offset helpers.
mod frame;

/// Minimal HTTPS GET client using WinHTTP.
pub mod http;

/// Global hotkey registration.
pub mod hotkey;

/// The thread that owns the global hotkeys.
mod hotkey_thread;

/// Key name to virtual key code mapping.
pub mod keys;

/// Win32 event translation.
mod event;

/// Single-instance daemon guard (named kernel mutex).
pub mod instance_guard;

/// Win32 event loop (SetWinEventHook + message pump).
pub mod event_loop;

/// IPC via Named Pipes.
pub mod ipc;

/// Threads that own Win32 state delivered through a message queue.
mod message_thread;

/// Monitor work area queries.
mod monitor;

/// The window under the mouse pointer, for focus-follows-mouse.
pub mod pointer;

/// Workarounds for Chromium-based windows.
mod chromium;

/// EDID and display-configuration string decoding.
mod monitor_edid;

/// Stable monitor identity from the display configuration.
mod monitor_identity;

/// Process utilities (alive check).
pub mod process;

/// Tiling manager that applies layouts to managed windows.
pub mod tiling;

/// GitHub release version check.
pub mod version_check;

/// Window type wrapping a Win32 `HWND`.
pub mod window;

/// Moving, showing, hiding and focusing windows through Win32.
pub mod window_control;

pub use window::Window;
