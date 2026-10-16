//! Telling the user about their config while it loads.
//!
//! The CLI and the daemon load the config the same way. These lines have
//! always gone to standard error, which is where the CLI shows them. The
//! daemon has no console, so they go to the log as well.

use std::fmt;

use crate::log::{self, Level};

/// A problem mosaico works around, such as a file it falls back from.
pub(super) fn warning(args: fmt::Arguments<'_>) {
    log::write(Level::Warn, args);
    eprintln!("Warning: {args}");
}

/// A change mosaico made to the user's files.
pub(super) fn info(args: fmt::Arguments<'_>) {
    log::write(Level::Info, args);
    eprintln!("Info: {args}");
}
