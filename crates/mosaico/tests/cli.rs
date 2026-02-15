use std::process::Command;

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
fn start_subcommand_runs() {
    // Arrange
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_mosaico"));
    cmd.arg("start");

    // Act
    let output = cmd.output().expect("failed to execute mosaico");

    // Assert
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Starting Mosaico"));
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
