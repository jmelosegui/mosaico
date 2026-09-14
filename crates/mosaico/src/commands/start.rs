use std::os::windows::process::CommandExt;
use std::process::Command;
use std::time::{Duration, Instant};

/// Windows process creation flags for launching a fully detached daemon.
///
/// `CREATE_NEW_PROCESS_GROUP` (0x200) — the daemon gets its own process
/// group, so Ctrl+C in the CLI terminal won't kill it.
///
/// `CREATE_NO_WINDOW` (0x08000000) — the daemon doesn't get a console
/// window. This also prevents inheriting the parent's console handles,
/// which avoids handle leaks that cause `cmd.output()` to hang in tests.
const DETACH_FLAGS: u32 = 0x08000000 | 0x00000200;

pub fn execute() {
    // Check if the daemon is already running
    if mosaico_windows::ipc::is_daemon_running() {
        println!("Mosaico is already running.");
        return;
    }

    // Clean up stale PID file from a previous unclean shutdown
    if let Ok(Some(pid)) = mosaico_core::pid::read_pid_file() {
        if mosaico_windows::process::is_process_alive(pid) {
            println!("Mosaico process exists (PID: {pid}) but is not responding.");
            return;
        }
        let _ = mosaico_core::pid::remove_pid_file();
    }

    // Get the path to the current executable so we can re-spawn it
    let exe = std::env::current_exe().expect("failed to get current executable path");

    // Spawn the daemon as a fully detached background process.
    // We re-run ourselves with the hidden `daemon` subcommand.
    // DETACH_FLAGS prevent handle inheritance so the parent can exit
    // immediately without waiting for the daemon to finish.
    let mut child = Command::new(exe)
        .arg("daemon")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .creation_flags(DETACH_FLAGS)
        .spawn()
        .expect("failed to start daemon");

    let pid = child.id();

    // Confirm it is actually up before claiming it started. The daemon
    // takes the single instance mutex first thing, and a daemon that is
    // still shutting down holds that mutex well after `mosaico stop`
    // returned, so a start issued in that window is rejected and exits.
    // Reporting success for a pid that is already dead is how that
    // window went unnoticed.
    if !wait_for_daemon(&mut child) {
        eprintln!("Error: the daemon exited immediately after starting.");
        eprintln!(
            "A previous daemon may still be shutting down and holding the single instance lock."
        );
        eprintln!("Check 'mosaico status', then try again.");
        std::process::exit(1);
    }

    print_banner(pid);
}

/// How long to wait for a freshly spawned daemon to answer.
const STARTUP_TIMEOUT: Duration = Duration::from_secs(10);

/// How often to check whether it is up yet.
const POLL_INTERVAL: Duration = Duration::from_millis(50);

/// Waits until the spawned daemon answers on its IPC pipe.
///
/// Returns false if it exits first, which is what a daemon rejected by
/// the single instance guard does.
fn wait_for_daemon(child: &mut std::process::Child) -> bool {
    let deadline = Instant::now() + STARTUP_TIMEOUT;
    loop {
        if mosaico_windows::ipc::is_daemon_running() {
            return true;
        }
        // Exited without ever answering.
        if matches!(child.try_wait(), Ok(Some(_))) {
            return false;
        }
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(POLL_INTERVAL);
    }
}

/// Tips shown on startup, rotated by PID so users see a different
/// one each time they start the daemon.
const TIPS: &[&str] = &[
    "Run 'mosaico doctor' to check your setup",
    "Run 'mosaico status' to check if the daemon is running",
    "Edit keybindings in ~/.config/mosaico/keybindings.toml",
    "Add window rules in ~/.config/mosaico/rules.toml",
    "Adjust gap and ratio in ~/.config/mosaico/config.toml",
    "Run 'mosaico init' to reset config files to defaults",
    "Run 'mosaico debug list' to see all managed windows",
    "Run 'mosaico debug events' to watch window events live",
];

fn print_banner(pid: u32) {
    let d = "\x1b[90m"; // Dim gray — labels
    let w = "\x1b[1;97m"; // Bold bright white — values
    let r = "\x1b[0m"; // Reset
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let tip = TIPS[secs as usize % TIPS.len()];

    super::banner::print_logo();
    println!();
    println!("  {d}Config{r}   ~/.config/mosaico/");
    println!("  {d}Daemon{r}   Started (PID: {w}{pid}{r})");
    println!("  {d}Repo{r}     https://github.com/jmelosegui/mosaico");
    println!("  {d}Tip{r}      {tip}");

    if let Some(remote) = mosaico_windows::version_check::check_for_update() {
        let y = "\x1b[33m"; // Yellow
        let local = env!("CARGO_PKG_VERSION");
        println!();
        println!("  {y}Update available: v{local} -> {remote}{r}");
        println!("  {y}Run 'mosaico update' to install it{r}");
    }

    println!();
}
