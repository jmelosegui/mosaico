//! What a daemon logs as it starts and stops.
//!
//! The logger is global and can only be set up once, so this file holds
//! a single test and gets a process of its own, where it starts empty.

use std::path::{Path, PathBuf};

use mosaico_core::lifecycle::{self, Startup};
use mosaico_core::log::{self, LogConfig};
use mosaico_core::system::{InstanceLock, SingleInstance};
use mosaico_core::update::Installer;
use mosaico_core::{Error, Instance, Result};

/// A lock no other daemon holds.
struct FreeLock;

impl SingleInstance for FreeLock {
    fn acquire(&self, _instance: &Instance) -> Result<InstanceLock> {
        Ok(InstanceLock::new(|| {}))
    }
}

/// A platform whose updates leave nothing behind.
struct NothingLeftOver;

impl Installer for NothingLeftOver {
    fn asset_name(&self) -> Result<&str> {
        Ok("mosaico.zip")
    }

    fn binary_name(&self) -> &str {
        "mosaico"
    }

    fn replace(&self, _exe: &Path, _binary: &[u8]) -> Result<()> {
        Ok(())
    }

    fn remove_leftovers(&self, _exe: &Path) -> Result<()> {
        Ok(())
    }
}

/// A fresh directory for this test's instance.
fn test_dir() -> PathBuf {
    let dir = std::env::temp_dir()
        .join("mosaico-log-tests")
        .join(format!("lifecycle-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

#[test]
fn a_daemon_logs_that_it_started_and_the_error_that_stopped_it() {
    // Arrange -- a loop that sets up the logger, as the platform's does,
    // and then fails.
    let dir = test_dir();
    let instance = Instance::new(None, dir.join("config"), dir.join("data")).unwrap();
    let log_dir = instance.log_dir();
    let config = LogConfig {
        enabled: true,
        level: "info".into(),
        max_file_mb: 10,
    };
    let serve = |_: &Instance| {
        log::init_in(&config, &log_dir);
        Err(Error::platform("start the event loop", "no desktop"))
    };

    // Act
    let result = lifecycle::run(Startup {
        instance: &instance,
        lock: &FreeLock,
        installer: &NothingLeftOver,
        exe: None,
        pid: 4242,
        serve: &serve,
    });

    // Assert -- the start was logged before the logger existed, and is
    // written once it does, ahead of the error that ended the loop.
    assert!(result.is_err());
    let text = std::fs::read_to_string(log_dir.join("mosaico.log")).expect("no log was written");
    let started = text
        .find("[INFO] Daemon started (PID: 4242)")
        .unwrap_or_else(|| panic!("the start is not in the log:\n{text}"));
    let stopped = text
        .find("[ERROR] Daemon stopped: could not start the event loop: no desktop")
        .unwrap_or_else(|| panic!("the error is not in the log:\n{text}"));
    assert!(started < stopped, "{text}");
}
