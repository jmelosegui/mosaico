use std::fs;
use std::panic::{AssertUnwindSafe, catch_unwind};

use super::*;
use crate::Error;
use crate::config::ConfigStore;
use crate::pid::PidFile;

#[path = "lifecycle_test_support.rs"]
mod support;

use support::{EXE, Harness, PID};

#[test]
fn a_daemon_holds_its_lock_and_pid_file_while_it_serves_and_gives_both_back() {
    // Arrange
    let h = Harness::new("serves");
    let serve = |instance: &Instance| {
        assert_eq!(instance, &h.instance, "the loop runs the daemon's instance");
        h.note(format!(
            "serving, PID file {:?}",
            h.pid_file().read().unwrap()
        ));
        Ok(())
    };

    // Act
    let result = h.run(&serve);

    // Assert -- the PID file is gone before the lock is released, so a
    // daemon that starts next never has its PID file removed by this one.
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        h.journal(),
        [
            format!("leftovers of {EXE}"),
            "locked".to_string(),
            format!("serving, PID file Some({PID})"),
            "released without the PID file".to_string(),
        ]
    );
    assert_eq!(h.pid_file().read().unwrap(), None);
}

#[test]
fn a_second_daemon_is_refused_and_leaves_the_running_daemons_pid_file_alone() {
    // Arrange -- another daemon holds the lock and recorded its process.
    let mut h = Harness::new("refused");
    h.lock.held_elsewhere = true;
    h.pid_file().write(77).unwrap();
    let serve = |_: &Instance| {
        h.note("serving");
        Ok(())
    };

    // Act
    let result = h.run(&serve);

    // Assert -- leftovers are still cleared, as before the lock.
    assert!(matches!(result, Err(Error::AlreadyRunning)), "{result:?}");
    assert_eq!(
        h.journal(),
        [format!("leftovers of {EXE}"), "refused".into()]
    );
    assert_eq!(h.pid_file().read().unwrap(), Some(77));
}

#[test]
fn a_loop_that_fails_still_removes_the_pid_file_and_releases_the_lock() {
    // Arrange
    let h = Harness::new("loop-fails");
    let serve = |_: &Instance| Err(Error::platform("start the event loop", "no desktop"));

    // Act
    let result = h.run(&serve);

    // Assert -- the loop's own error, for the daemon command to print.
    assert!(
        matches!(
            result,
            Err(Error::Platform {
                operation: "start the event loop",
                ..
            })
        ),
        "{result:?}"
    );
    assert_eq!(
        h.journal().last().map(String::as_str),
        Some("released without the PID file")
    );
    assert_eq!(h.pid_file().read().unwrap(), None);
}

#[test]
fn a_loop_that_panics_still_removes_the_pid_file_before_the_lock_goes() {
    // Arrange
    let h = Harness::new("loop-panics");
    let serve = |_: &Instance| -> Result<()> { panic!("the loop broke") };

    // Act -- tests unwind. A release build aborts on a panic instead and
    // leaves the PID file behind, as a crash does.
    let unwound = catch_unwind(AssertUnwindSafe(|| h.run(&serve)));

    // Assert -- the panic goes on to the caller, after the cleanup.
    assert!(unwound.is_err(), "the panic should reach the caller");
    assert_eq!(
        h.journal(),
        [
            format!("leftovers of {EXE}"),
            "locked".into(),
            "released without the PID file".into(),
        ]
    );
    assert_eq!(h.pid_file().read().unwrap(), None);
}

#[test]
fn a_pid_file_that_cannot_be_written_stops_the_daemon_before_its_loop() {
    // Arrange -- a file where the data directory should be.
    let h = Harness::new("pid-unwritable");
    let data_dir = h.instance.data_dir();
    fs::create_dir_all(data_dir.parent().unwrap()).unwrap();
    fs::write(data_dir, "not a directory").unwrap();
    let serve = |_: &Instance| {
        h.note("serving");
        Ok(())
    };

    // Act
    let result = h.run(&serve);

    // Assert
    assert!(matches!(result, Err(Error::Io(_))), "{result:?}");
    assert_eq!(
        h.journal(),
        [
            format!("leftovers of {EXE}"),
            "locked".into(),
            "released without the PID file".into(),
        ]
    );
}

#[test]
fn leftovers_that_cannot_be_deleted_do_not_stop_the_daemon() {
    // Arrange
    let mut h = Harness::new("leftovers-stay");
    h.installer.fails = true;
    let serve = |_: &Instance| {
        h.note("serving");
        Ok(())
    };

    // Act
    let result = h.run(&serve);

    // Assert
    assert!(result.is_ok(), "{result:?}");
    assert!(
        h.journal().contains(&"serving".to_string()),
        "{:?}",
        h.journal()
    );
}

#[test]
fn no_leftovers_are_looked_for_when_the_executable_is_unknown() {
    // Arrange
    let mut h = Harness::new("no-exe");
    h.exe = None;
    let serve = |_: &Instance| Ok(());

    // Act
    let result = h.run(&serve);

    // Assert
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        h.journal(),
        ["locked", "released without the PID file"].map(String::from)
    );
}

#[test]
fn a_daemon_without_config_files_writes_the_defaults_before_its_loop_reads_them() {
    // Arrange -- a fresh install, where the daemon starts before
    // `mosaico init` ever ran, and the config folder does not exist.
    let h = Harness::new("writes-defaults");
    let store = ConfigStore::for_instance(&h.instance);
    assert!(!store.dir().exists());
    let serve = |_: &Instance| {
        let files: Vec<_> = [
            store.config_path(),
            store.keybindings_path(),
            store.rules_path(),
            store.user_rules_path(),
            store.bar_path(),
        ]
        .iter()
        .filter(|path| path.is_file())
        .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
        h.note(format!("serving with {}", files.join(", ")));
        Ok(())
    };

    // Act
    let result = h.run(&serve);

    // Assert
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        h.journal()[2],
        "serving with config.toml, keybindings.toml, rules.toml, user-rules.toml, bar.toml"
    );
}
