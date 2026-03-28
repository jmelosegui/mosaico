#![deny(missing_docs)]

//! Platform-agnostic core types and traits for Mosaico.

/// User-triggerable actions (focus, move, workspace switch).
pub mod action;
/// Configuration loading and types.
pub mod config;
/// Window lifecycle events from the platform.
pub mod event;
/// Inter-process communication protocol types.
pub mod ipc;
/// Tiling layout trait and implementations.
pub mod layout;
/// Logging configuration and helpers.
pub mod log;
/// PID file management for the daemon.
pub mod pid;
/// Axis-aligned rectangle type.
pub mod rect;
/// Spatial direction helpers.
pub mod spatial;
/// Build-time version information.
pub mod version;
/// Platform-agnostic window abstraction.
pub mod window;
/// Workspace state and operations.
pub mod workspace;

pub use action::{Action, Direction};
pub use config::{BarConfig, Config};
pub use event::WindowEvent;
pub use ipc::{Command, Response};
pub use layout::{BspLayout, Layout};
pub use rect::Rect;
pub use window::{Window, WindowResult};
pub use workspace::Workspace;
