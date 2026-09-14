use std::process::{Command, Output, Stdio};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

/// How long any single CLI invocation is allowed to take.
const CLI_TIMEOUT: Duration = Duration::from_secs(30);

/// Serializes the tests that drive a real daemon.
///
/// Only one daemon can hold the single instance mutex, and every one of
/// these tests begins by stopping whatever is running. Left to run in
/// parallel they clobber each other: one test's `stop` kills the daemon
/// another test just started, so the first fails and the second is left
/// waiting on a `mosaico daemon` that was never rejected and therefore
/// never exits. Taking this guard makes them run one at a time.
fn test_guard() -> std::sync::MutexGuard<'static, ()> {
    static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    TEST_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|e| {
            eprintln!("test mutex poisoned, recovering");
            e.into_inner()
        })
}

/// Runs the CLI and gives up after [`CLI_TIMEOUT`] instead of blocking.
///
/// `Command::output` waits for the child to exit and for every write end
/// of its pipes to close, so a daemon that outlives the command, or a
/// grandchild holding an inherited handle, hangs the caller for good.
/// A hung test that fails after half a minute is debuggable; one that
/// blocks a CI runner until the job limit is not.
///
/// Panics with the argument list when the timeout is reached, after
/// killing the child so the pipes close and the next test starts clean.
fn run_cli(args: &[&str]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_mosaico"))
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("failed to spawn mosaico {args:?}: {e}"));

    let deadline = Instant::now() + CLI_TIMEOUT;
    loop {
        match child.try_wait().expect("failed to poll mosaico") {
            Some(_) => break,
            None if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                panic!("mosaico {args:?} did not exit within {CLI_TIMEOUT:?}");
            }
            None => std::thread::sleep(Duration::from_millis(50)),
        }
    }

    child
        .wait_with_output()
        .unwrap_or_else(|e| panic!("failed to read output of mosaico {args:?}: {e}"))
}

/// Runs a CLI command that leaves a daemon behind, returning only whether
/// it succeeded.
///
/// `mosaico start` spawns the daemon as a grandchild that outlives the
/// command. If its output were piped, the daemon would inherit the write
/// end and reading to end of file would never finish. Sending both
/// streams to the null device removes anything to wait on.
fn run_cli_detached(args: &[&str]) -> bool {
    let mut child = Command::new(env!("CARGO_BIN_EXE_mosaico"))
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|e| panic!("failed to spawn mosaico {args:?}: {e}"));

    let deadline = Instant::now() + CLI_TIMEOUT;
    loop {
        match child.try_wait().expect("failed to poll mosaico") {
            Some(status) => return status.success(),
            None if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                panic!("mosaico {args:?} did not exit within {CLI_TIMEOUT:?}");
            }
            None => std::thread::sleep(Duration::from_millis(50)),
        }
    }
}

#[test]
fn help_exits_successfully() {
    // Act
    let output = run_cli(&["--help"]);

    // Assert
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("tiling window manager"));
}

#[test]
fn version_exits_successfully() {
    // Act
    let output = run_cli(&["--version"]);

    // Assert
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("mosaico"));
}

#[test]
fn start_and_stop_lifecycle() {
    let _guard = test_guard();

    // Arrange — make sure no daemon is already running
    let _ = run_cli(&["stop"]);

    // Act — start the daemon
    let started = run_cli_detached(&["start"]);

    // Assert — start should succeed
    assert!(started);

    // Give the daemon a moment to create its pipe
    std::thread::sleep(Duration::from_secs(1));

    // Act — check status
    let status_output = run_cli(&["status"]);

    assert!(status_output.status.success());
    let status_stdout = String::from_utf8_lossy(&status_output.stdout);
    assert!(
        status_stdout.contains("running"),
        "Expected 'running', got: {status_stdout}"
    );

    // Act — stop the daemon
    let stop_output = run_cli(&["stop"]);

    // Assert — stop should succeed
    assert!(stop_output.status.success());
    let stop_stdout = String::from_utf8_lossy(&stop_output.stdout);
    assert!(
        stop_stdout.contains("stopped"),
        "Expected 'stopped', got: {stop_stdout}"
    );
}

#[test]
fn second_daemon_is_rejected() {
    let _guard = test_guard();

    // Arrange — stop any running daemon first
    let _ = run_cli(&["stop"]);

    // Start the daemon via `mosaico start` (detached process)
    let _ = run_cli_detached(&["start"]);

    // Wait for the daemon to acquire the mutex
    std::thread::sleep(Duration::from_secs(1));

    // Act — try to start a second daemon directly.
    // A rejected daemon prints to stderr and exits. If the guard ever
    // stops rejecting it, this one keeps running and run_cli kills it,
    // so the test fails with a clear message rather than hanging.
    let output = run_cli(&["daemon"]);

    // Cleanup — stop the running daemon
    let _ = run_cli(&["stop"]);

    // Assert — second daemon should fail with "already running"
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
    // Act
    let output = run_cli(&["debug", "list"]);

    // Assert
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("windows found"));
}
