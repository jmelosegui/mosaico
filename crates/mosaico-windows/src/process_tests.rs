use std::io::Read;
use std::time::{Duration, Instant};

use super::*;

/// Set in the copy of this test binary that runs [`launcher`].
const LAUNCHER: &str = "MOSAICO_PROCESS_TESTS_LAUNCHER";

/// Starts a program that runs for about six seconds, detached, as the
/// CLI starts the daemon. Only does so in the copy of this test binary
/// that the test below runs.
#[test]
#[ignore = "run by a_detached_program_does_not_hold_its_launchers_output_open"]
fn launcher() {
    if std::env::var_os(LAUNCHER).is_none() {
        return;
    }
    let mut sleeper = Command::new("ping");
    sleeper.args(["-n", "7", "127.0.0.1"]);
    spawn_detached(sleeper, "start the sleeper").unwrap();
}

#[test]
fn a_detached_program_does_not_hold_its_launchers_output_open() {
    // Arrange -- the launcher's output is a pipe that is read until it
    // closes, as an installer reads the output of a hook.
    let mut launcher = Command::new(std::env::current_exe().unwrap())
        .args(["process::tests::launcher", "--exact", "--ignored"])
        .env(LAUNCHER, "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let started = Instant::now();

    // Act
    let mut out = String::new();
    let mut pipe = launcher.stdout.take().unwrap();
    pipe.read_to_string(&mut out).unwrap();
    let closed_after = started.elapsed();
    launcher.wait().unwrap();

    // Assert -- the pipe closes when the launcher exits, not when the
    // program it started does.
    assert!(out.contains("1 passed"), "{out}");
    assert!(
        closed_after < Duration::from_secs(4),
        "the output stayed open for {closed_after:?}"
    );
}
