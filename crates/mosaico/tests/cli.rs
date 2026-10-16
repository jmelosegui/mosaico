mod common;

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use mosaico_core::system::ProcessSetup;
use mosaico_core::tiler::Enumerator;
use mosaico_windows::enumerate::WindowsEnumerator;
use mosaico_windows::setup::WindowsSetup;

use common::{TestInstance, desktop_guard};

/// Every file and directory under `dir`, with its size and when it was
/// last written, to tell whether a command changed anything there.
fn snapshot(dir: &Path) -> Vec<(PathBuf, u64, SystemTime)> {
    let mut entries = Vec::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir).expect("failed to read a test directory") {
            let entry = entry.expect("failed to read a directory entry");
            let meta = entry.metadata().expect("failed to read file metadata");
            if meta.is_dir() {
                pending.push(entry.path());
            }
            let modified = meta.modified().expect("no modification time");
            entries.push((entry.path(), meta.len(), modified));
        }
    }
    entries.sort();
    entries
}

/// Runs the CLI of `instance` to the end and returns its standard output.
///
/// Unlike [`TestInstance::stdout`], this reads the output while the
/// command runs, which `doctor` needs: its animated logo alone is more
/// than a pipe holds, so it would wait for a reader until the timeout.
fn whole_stdout(instance: &TestInstance, args: &[&str]) -> String {
    let output = instance
        .command(args)
        .output()
        .unwrap_or_else(|e| panic!("failed to run mosaico {args:?}: {e}"));
    assert!(output.status.success(), "mosaico {args:?} failed");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// The size and position of each monitor a command lists, from lines
/// such as `Monitor 0: 3840x2064 at (0, 0)`, in the order it lists them.
fn monitor_rects(output: &str) -> Vec<String> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("Monitor "))
        .filter_map(|line| line.split_once(": "))
        .map(|(_, rect)| rect.to_string())
        .collect()
}

#[test]
fn help_exits_successfully() {
    // Arrange
    let instance = TestInstance::new("help");

    // Act
    let output = instance.run(&["--help"]);

    // Assert
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("tiling window manager"));
}

#[test]
fn version_exits_successfully() {
    // Arrange
    let instance = TestInstance::new("version");

    // Act
    let output = instance.run(&["--version"]);

    // Assert
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("mosaico"));
}

#[test]
fn an_invalid_instance_name_is_refused() {
    // Arrange -- falling back to the default instance would reach the
    // user's own daemon.
    let instance = TestInstance::new("invalid");

    // Act
    let output = instance
        .command(&["status"])
        .env(mosaico_core::instance::INSTANCE_VAR, "not a valid name")
        .output()
        .expect("failed to run mosaico");

    // Assert
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("MOSAICO_INSTANCE"), "{stderr}");
}

#[test]
fn a_test_instance_does_not_see_other_daemons() {
    // Arrange -- nothing was started for this instance, whatever else
    // runs on the machine.
    let instance = TestInstance::new("isolated");

    // Act
    let stdout = instance.stdout(&["status"]);

    // Assert
    assert!(stdout.contains("not running"), "{stdout}");
}

#[test]
fn start_and_stop_lifecycle() {
    let _guard = desktop_guard();
    let instance = TestInstance::new("lifecycle");
    instance.write_config("config.toml", "[logging]\nenabled = true\n");

    // Act -- start the daemon and wait for it to answer, rather than
    // guessing at a fixed delay
    instance.start_daemon();
    let status_output = instance.run(&["status"]);

    // Assert
    assert!(status_output.status.success());
    let status_stdout = String::from_utf8_lossy(&status_output.stdout);
    assert!(
        status_stdout.contains("running"),
        "Expected 'running', got: {status_stdout}"
    );
    assert!(
        instance.root().join("data").join("mosaico.pid").exists(),
        "the PID file belongs in the instance's data directory"
    );
    assert!(
        instance.log_path().exists(),
        "the log belongs in the instance's config directory"
    );

    // Act -- stop the daemon
    let stop_output = instance.run(&["stop"]);

    // Assert
    assert!(stop_output.status.success());
    let stop_stdout = String::from_utf8_lossy(&stop_output.stdout);
    assert!(
        stop_stdout.contains("stopped"),
        "Expected 'stopped', got: {stop_stdout}"
    );
}

#[test]
fn second_daemon_is_rejected() {
    let _guard = desktop_guard();
    let instance = TestInstance::new("second");

    // Arrange -- exactly one daemon of this instance holds its lock.
    instance.start_daemon();

    // Act -- try to start a second daemon of the same instance directly.
    // A rejected daemon prints to stderr and exits. If the guard ever
    // stops rejecting it, this one keeps running and the timeout kills
    // it, so the test fails with a clear message rather than hanging.
    let output = instance.run(&["daemon"]);

    // Cleanup
    instance.stop_daemon();

    // Assert
    assert!(
        !output.status.success(),
        "Second daemon should exit with error"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("already running"),
        "Expected 'already running' in stderr, got: {stderr}"
    );
}

#[test]
fn debug_list_subcommand_runs() {
    // Arrange
    let instance = TestInstance::new("list");

    // Act
    let output = instance.run(&["debug", "list"]);

    // Assert
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("windows found"));
}

#[test]
fn doctor_and_debug_list_show_monitors_as_the_daemon_sees_them() {
    // Arrange -- a fresh instance leaves doctor nothing to fix or migrate:
    // its config directory exists, and it has no config and no PID file.
    let instance = TestInstance::new("doctor-monitors");
    let before = snapshot(instance.root());

    // The monitors as a process set up like the daemon sees them. On a
    // scaled display that is physical pixels, which a command that skips
    // the setup would not show.
    WindowsSetup.init();
    let daemon_sees: Vec<String> = WindowsEnumerator
        .monitors()
        .expect("failed to list the monitors")
        .iter()
        .map(|m| {
            let r = m.work_area;
            format!("{}x{} at ({}, {})", r.width, r.height, r.x, r.y)
        })
        .collect();

    // Act
    let doctor = whole_stdout(&instance, &["doctor"]);
    let after_doctor = snapshot(instance.root());
    let list = whole_stdout(&instance, &["debug", "list"]);

    // Assert -- doctor only read.
    assert_eq!(after_doctor, before, "doctor changed the instance's files");
    assert_eq!(monitor_rects(&doctor), daemon_sees, "doctor");
    assert_eq!(monitor_rects(&list), daemon_sees, "debug list");
}
