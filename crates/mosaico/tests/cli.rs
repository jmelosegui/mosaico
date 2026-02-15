use std::process::Command;
use std::time::Duration;

#[test]
fn help_exits_successfully() {
    // Arrange
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_mosaico"));
    cmd.arg("--help");

    // Act
    let output = cmd.output().expect("failed to execute mosaico");

    // Assert
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("tiling window manager"));
}

#[test]
fn version_exits_successfully() {
    // Arrange
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_mosaico"));
    cmd.arg("--version");

    // Act
    let output = cmd.output().expect("failed to execute mosaico");

    // Assert
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("mosaico"));
}

#[test]
fn start_and_stop_lifecycle() {
    // Arrange — make sure no daemon is already running
    let _ = Command::new(env!("CARGO_BIN_EXE_mosaico"))
        .arg("stop")
        .output();

    // Act — start the daemon.
    // We use spawn() + wait() instead of output() because on Windows the
    // daemon grandchild process inherits the stdout pipe handle. output()
    // waits for all write ends to close, which hangs until the daemon exits.
    // spawn() + wait() only waits for the direct child to exit.
    let mut child = Command::new(env!("CARGO_BIN_EXE_mosaico"))
        .arg("start")
        .spawn()
        .expect("failed to spawn mosaico start");

    let status = child.wait().expect("failed to wait for mosaico start");

    // Assert — start should succeed
    assert!(status.success());

    // Give the daemon a moment to create its pipe
    std::thread::sleep(Duration::from_secs(1));

    // Act — check status (output() is fine here, no grandchild spawned)
    let status_output = Command::new(env!("CARGO_BIN_EXE_mosaico"))
        .arg("status")
        .output()
        .expect("failed to execute mosaico status");

    assert!(status_output.status.success());
    let status_stdout = String::from_utf8_lossy(&status_output.stdout);
    assert!(
        status_stdout.contains("running"),
        "Expected 'running', got: {status_stdout}"
    );

    // Act — stop the daemon
    let stop_output = Command::new(env!("CARGO_BIN_EXE_mosaico"))
        .arg("stop")
        .output()
        .expect("failed to execute mosaico stop");

    // Assert — stop should succeed
    assert!(stop_output.status.success());
    let stop_stdout = String::from_utf8_lossy(&stop_output.stdout);
    assert!(
        stop_stdout.contains("stopped"),
        "Expected 'stopped', got: {stop_stdout}"
    );
}

#[test]
fn debug_list_subcommand_runs() {
    // Arrange
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_mosaico"));
    cmd.args(["debug", "list"]);

    // Act
    let output = cmd.output().expect("failed to execute mosaico");

    // Assert
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("windows found"));
}
