#![deny(unsafe_op_in_unsafe_fn)]
#![warn(clippy::undocumented_unsafe_blocks)]

/// Windows startup registration via the HKCU Run registry key.
pub mod autostart;

/// COM-based window cloaking via the ImmersiveShell.
mod com;

/// Status bar overlay windows.
pub mod bar;

/// Border overlay windows for visual focus indicators.
pub mod border;

/// Ctrl+C in the console, through SetConsoleCtrlHandler.
pub mod ctrl_c;

/// Daemon main loop.
pub mod daemon;

/// Focus borders and window corners through Win32.
mod decorations;

/// Change notifications on a directory, for config hot-reload.
pub mod directory_watcher;

/// DPI awareness, which [`setup`] turns on in every process.
mod dpi;

/// Window and monitor listing for the core `Enumerator` contract.
pub mod enumerate;

/// Wrapping native errors in the core error.
mod error;

/// Quick file checks that never wait on a network share.
pub mod files;

/// Window frame and border offset helpers.
mod frame;

/// The HTTP client, through WinHTTP.
pub mod http;

/// Global hotkeys registered with `RegisterHotKey`.
mod hotkey;

/// The thread that owns the global hotkeys.
mod hotkey_thread;

/// Virtual-key codes for the keys a hotkey can be bound to.
mod keys;

/// Win32 event translation.
mod event;

/// Installing a downloaded release over the running executable.
pub mod install;

/// The single instance lock, as a named kernel mutex.
pub mod instance_guard;

/// Window events from Win32 event hooks and broadcasts.
pub mod event_source;

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

/// Starting, checking and ending processes.
pub mod process;

/// What every mosaico process sets up first.
pub mod setup;

/// Machine readings for the bar's system widgets.
pub mod system_info;

/// The Win32 platform the core tiling manager runs on.
pub mod tiling;

/// The user's profile and local application data folders.
pub mod user_dirs;

/// Window type wrapping a Win32 `HWND`, which other crates reach only
/// through the core contracts.
mod window;

/// Moving, showing, hiding and focusing windows through Win32.
pub mod window_control;
