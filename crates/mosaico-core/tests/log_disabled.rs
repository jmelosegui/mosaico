//! Logging that is turned off.
//!
//! The logger is global and can only be set up once, so this file holds
//! a single test and gets a process of its own, where it starts empty.

use std::path::PathBuf;

use mosaico_core::log::{self, LogConfig};
use mosaico_core::log_warn;

/// A fresh directory for this test's log, which must never appear.
fn log_dir() -> PathBuf {
    let dir = std::env::temp_dir()
        .join("mosaico-log-tests")
        .join(format!("disabled-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

#[test]
fn nothing_is_written_when_logging_is_off() {
    // Arrange
    let dir = log_dir();
    log_warn!("early warn");
    let off = LogConfig {
        enabled: false,
        ..LogConfig::default()
    };
    let on = LogConfig {
        enabled: true,
        ..LogConfig::default()
    };

    // Act -- the lines logged so far are dropped, and the logger is set
    // up for good, so turning it on afterwards changes nothing.
    log::init_in(&off, &dir);
    log_warn!("late warn");
    log::init_in(&on, &dir);
    log_warn!("later warn");

    // Assert -- not even the directory was created.
    assert!(!dir.exists(), "{} should not exist", dir.display());
}
