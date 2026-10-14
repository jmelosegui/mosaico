mod common;

use common::{TestInstance, desktop_guard};

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
