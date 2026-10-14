#![deny(missing_docs)]

//! Platform-agnostic core types and traits for Mosaico.

/// User-triggerable actions (focus, move, workspace switch).
pub mod action;
/// Configuration loading and types.
pub mod config;
/// The contract for drawing borders and corners on windows.
pub mod decoration;
/// The error type shared by core and the platform contracts.
pub mod error;
/// Window events, and the sources that report them.
pub mod event;
/// The ways a platform can hide windows, and what each one reports.
pub mod hiding;
/// Hotkeys: key names, and the contract for claiming them.
pub mod hotkey;
/// Fetching files from web servers.
pub mod http;
/// Opaque window and monitor ids.
pub mod id;
/// Which daemon instance a process belongs to.
pub mod instance;
/// Inter-process communication protocol types.
pub mod ipc;
/// Tiling layout trait and implementations.
pub mod layout;
/// Logging configuration and helpers.
pub mod log;
/// Platform-agnostic monitor identity and matching.
pub mod monitor;
/// PID file management for the daemon.
pub mod pid;
/// Axis-aligned rectangle type.
pub mod rect;
/// Spatial direction helpers.
pub mod spatial;
/// OS services the CLI uses: processes and start at login.
pub mod system;
/// The tiling state machine.
pub mod tiler;
/// Build-time version information.
pub mod version;
/// The contract for changing windows through the platform.
pub mod window;
/// Workspace state and operations.
pub mod workspace;

pub use action::{Action, Direction};
pub use config::{BarConfig, Config};
pub use decoration::Decorations;
pub use error::{Error, Result};
pub use event::WindowEvent;
pub use hiding::{HidingEvents, HidingSupport};
pub use id::{MonitorId, WindowId};
pub use instance::Instance;
pub use ipc::{Command, Response};
pub use layout::{BspLayout, Layout, LayoutKind, ThreeColumnLayout, VerticalStackLayout};
pub use monitor::{EdidIdentity, Monitor, MonitorKey, match_monitors, pnp_manufacturer};
pub use rect::Rect;
pub use window::WindowControl;
pub use workspace::Workspace;
