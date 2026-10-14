use std::cell::Cell;

use mosaico_core::system::{InstanceLock, SingleInstance};
use mosaico_core::{Error, Instance, Result};

use super::*;
use crate::commands::test_support::FakeInstaller;

/// A lock that another daemon may hold already.
#[derive(Default)]
struct Lock {
    held_elsewhere: Cell<bool>,
}

impl SingleInstance for Lock {
    fn acquire(&self, _instance: &Instance) -> Result<InstanceLock> {
        if self.held_elsewhere.get() {
            return Err(Error::AlreadyRunning);
        }
        Ok(InstanceLock::new(|| {}))
    }
}

/// Runs the daemon command with `lock` and `serve` as its loop, on an
/// instance in a fresh temporary directory named after `test`. Returns
/// how it ended and what it wrote to its error stream.
fn run_daemon(test: &str, lock: &Lock, serve: &dyn Fn(&Instance) -> Result<()>) -> (Exit, String) {
    let dir = std::env::temp_dir()
        .join("mosaico-command-tests")
        .join(format!("{test}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let instance = Instance::new(None, dir.join("config"), dir.join("data")).unwrap();
    let installer = FakeInstaller::default();
    let mut err = Vec::new();
    let startup = Startup {
        instance: &instance,
        lock,
        installer: &installer,
        exe: None,
        pid: 4321,
        serve,
    };

    let exit = execute(startup, &mut err);
    (exit, String::from_utf8_lossy(&err).into_owned())
}

#[test]
fn a_second_daemon_says_one_is_already_running() {
    // Arrange
    let lock = Lock::default();
    lock.held_elsewhere.set(true);

    // Act
    let (exit, err) = run_daemon("daemon-second", &lock, &|_: &Instance| Ok(()));

    // Assert -- what the second_daemon_is_rejected integration test looks
    // for, and exit status 1.
    assert_eq!(exit, Exit::Failure);
    assert_eq!(err, "Daemon error: Mosaico daemon is already running.\n");
}

#[test]
fn a_loop_that_fails_is_reported_with_its_error() {
    // Arrange
    let lock = Lock::default();
    let serve = |_: &Instance| Err(Error::platform("start the event loop", "no desktop"));

    // Act
    let (exit, err) = run_daemon("daemon-loop-fails", &lock, &serve);

    // Assert
    assert_eq!(exit, Exit::Failure);
    assert_eq!(
        err,
        "Daemon error: could not start the event loop: no desktop\n"
    );
}

#[test]
fn a_daemon_that_is_stopped_exits_successfully() {
    // Arrange
    let lock = Lock::default();

    // Act
    let (exit, err) = run_daemon("daemon-stopped", &lock, &|_: &Instance| Ok(()));

    // Assert
    assert_eq!(exit, Exit::Success);
    assert_eq!(err, "");
}
