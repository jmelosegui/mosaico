//! Drives the core tiler against the Windows desktop.
//!
//! [`mosaico_core::tiler::Tiler`] makes every decision. This module
//! feeds it actions and events, applies the effects it returns through
//! [`applier::Applier`], and drops the events those effects cause.

mod applier;
mod desktop;
mod inspect;
mod manager;
mod platform;

pub use manager::TilingManager;
pub use platform::Win32Platform;
