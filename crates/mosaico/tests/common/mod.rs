//! Helpers shared by the integration tests.
//!
//! Every test runs the CLI against its own daemon instance, with its own
//! pipe, single instance lock, PID file, config and logs in a temporary
//! directory, so it never talks to the user's daemon or touches their
//! config. See `mosaico_core::instance`.

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant};

use mosaico_core::instance::{CONFIG_DIR_VAR, DATA_DIR_VAR, INSTANCE_VAR};

/// How long any single CLI invocation is allowed to take.
pub const CLI_TIMEOUT: Duration = Duration::from_secs(30);

/// A daemon instance private to one test.
///
/// Dropping it stops its daemon, if one is still running, and deletes
/// its directory, so a failing test leaves nothing behind.
pub struct TestInstance {
    name: String,
    root: PathBuf,
}

impl TestInstance {
    /// A fresh instance; `label` names it in process lists and logs.
    pub fn new(label: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let label: String = label
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
            .take(32)
            .collect();
        let name = format!(
            "test-{label}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        );
        let root = std::env::temp_dir().join("mosaico-tests").join(&name);
        std::fs::create_dir_all(root.join("config")).expect("failed to create test config dir");
        std::fs::create_dir_all(root.join("data")).expect("failed to create test data dir");
        Self { name, root }
    }

    /// The instance's config directory.
    pub fn config_dir(&self) -> PathBuf {
        self.root.join("config")
    }

    /// Writes a file into the instance's config directory.
    pub fn write_config(&self, file: &str, contents: &str) {
        std::fs::write(self.config_dir().join(file), contents)
            .unwrap_or_else(|e| panic!("failed to write {file}: {e}"));
    }

    /// The `mosaico` binary, pointed at this instance.
    pub fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_mosaico"));
        command
            .args(args)
            .env(INSTANCE_VAR, &self.name)
            .env(CONFIG_DIR_VAR, self.config_dir())
            .env(DATA_DIR_VAR, self.root.join("data"));
        command
    }

    /// Runs the CLI and gives up after [`CLI_TIMEOUT`] instead of
    /// blocking.
    ///
    /// `Command::output` waits for the child to exit and for every write
    /// end of its pipes to close, so a daemon that outlives the command
    /// would hang the caller for good. Panics with the arguments when
    /// the timeout is reached, after killing the child.
    pub fn run(&self, args: &[&str]) -> Output {
        let child = self
            .command(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap_or_else(|e| panic!("failed to spawn mosaico {args:?}: {e}"));
        wait_for(child, args)
            .wait_with_output()
            .unwrap_or_else(|e| panic!("failed to read output of mosaico {args:?}: {e}"))
    }

    /// Runs the CLI and returns its standard output as text.
    pub fn stdout(&self, args: &[&str]) -> String {
        String::from_utf8_lossy(&self.run(args).stdout).into_owned()
    }

    /// Runs a command that leaves a daemon behind, returning whether it
    /// succeeded.
    ///
    /// `mosaico start` spawns the daemon as a grandchild that outlives
    /// the command. If its output were piped, the daemon would inherit
    /// the write end and reading to end of file would never finish, so
    /// both streams go to the null device.
    pub fn run_detached(&self, args: &[&str]) -> bool {
        let child = self
            .command(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap_or_else(|e| panic!("failed to spawn mosaico {args:?}: {e}"));
        wait_for(child, args)
            .wait()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    /// Whether this instance's daemon answers on its pipe.
    pub fn daemon_is_running(&self) -> bool {
        self.stdout(&["status"]).contains("is running")
    }

    /// Starts this instance's daemon and waits until it answers.
    ///
    /// Retried, because a daemon that was just stopped holds its single
    /// instance lock until it has finished unwinding, and a start in
    /// that window is rejected.
    pub fn start_daemon(&self) {
        let deadline = Instant::now() + CLI_TIMEOUT;
        loop {
            assert!(
                self.run_detached(&["start"]),
                "mosaico start exited with a failure"
            );
            let attempt = Instant::now() + Duration::from_secs(3);
            while Instant::now() < attempt {
                if self.daemon_is_running() {
                    return;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            assert!(
                Instant::now() < deadline,
                "daemon did not come up within {CLI_TIMEOUT:?}"
            );
        }
    }

    /// Stops this instance's daemon and waits until it stops answering.
    pub fn stop_daemon(&self) {
        let _ = self.run(&["stop"]);
        let deadline = Instant::now() + CLI_TIMEOUT;
        while self.daemon_is_running() {
            assert!(
                Instant::now() < deadline,
                "daemon still answering {CLI_TIMEOUT:?} after stop"
            );
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    /// The daemon's log file.
    pub fn log_path(&self) -> PathBuf {
        self.config_dir().join("logs").join("mosaico.log")
    }

    /// Root of the instance's files.
    pub fn root(&self) -> &Path {
        &self.root
    }
}

impl Drop for TestInstance {
    fn drop(&mut self) {
        // Best effort: never panic while a failing test is unwinding.
        let _ = self
            .command(&["stop"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

/// Stops a test daemon when dropped, including when the test panics.
///
/// A daemon left behind keeps tiling the desktop, so every test that
/// starts one holds this for as long as it needs it.
#[must_use = "dropping the guard stops the daemon straight away"]
pub struct RunningDaemon<'a> {
    instance: &'a TestInstance,
}

impl<'a> RunningDaemon<'a> {
    /// Takes responsibility for stopping `instance`'s daemon.
    pub fn new(instance: &'a TestInstance) -> Self {
        Self { instance }
    }
}

impl Drop for RunningDaemon<'_> {
    fn drop(&mut self) {
        let _ = self
            .instance
            .command(&["stop"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

/// Waits for `child` to exit, killing it and panicking after
/// [`CLI_TIMEOUT`].
fn wait_for(mut child: Child, args: &[&str]) -> Child {
    let deadline = Instant::now() + CLI_TIMEOUT;
    loop {
        match child.try_wait().expect("failed to poll mosaico") {
            Some(_) => return child,
            None if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                panic!("mosaico {args:?} did not exit within {CLI_TIMEOUT:?}");
            }
            None => std::thread::sleep(Duration::from_millis(50)),
        }
    }
}

/// Serializes the tests in one binary that start a daemon.
///
/// Their instances are separate, but each daemon manages the same
/// desktop and registers the same global hotkeys, so two at once would
/// fight. Also checks that the user's own daemon is not running.
pub fn desktop_guard() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let guard = LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    require_no_user_daemon();
    guard
}

/// Fails the test if the user's own daemon is running.
///
/// Tests that start a daemon put a second window manager on the same
/// desktop. Isolating the instance keeps it from talking to the user's
/// daemon, but two window managers would still fight over every window,
/// so these tests refuse to run alongside it instead of stopping it.
pub fn require_no_user_daemon() {
    let output = Command::new(env!("CARGO_BIN_EXE_mosaico"))
        .arg("status")
        .env_remove(INSTANCE_VAR)
        .env_remove(CONFIG_DIR_VAR)
        .env_remove(DATA_DIR_VAR)
        .stdin(Stdio::null())
        .output()
        .expect("failed to run mosaico status");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("is running"),
        "your own mosaico daemon is running; stop it with `mosaico stop` before running \
         the tests that start a daemon, or two window managers will fight over the desktop"
    );
}
