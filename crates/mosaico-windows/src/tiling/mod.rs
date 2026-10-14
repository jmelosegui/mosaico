//! The Windows desktop the core tiling manager runs on.
//!
//! [`mosaico_core::tiler::TilingManager`] drives the tiler and applies
//! the effects it returns. This module hands it the Win32
//! implementation of each contract it needs, in [`Win32Platform`], and
//! builds the `debug state` snapshot from it, in `TilingViews`.

mod desktop;
mod inspect;
mod platform;
mod views;

pub use platform::Win32Platform;
pub(crate) use views::TilingViews;

/// The tiling manager on the Win32 desktop.
pub type TilingManager = mosaico_core::tiler::TilingManager<Win32Platform>;
