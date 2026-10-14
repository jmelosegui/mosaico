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
mod views;

pub use platform::Win32Platform;
pub(crate) use views::TilingViews;

/// The tiling manager on the Win32 desktop.
pub type TilingManager = manager::TilingManager<Win32Platform>;
