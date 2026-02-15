use std::os::windows::process::CommandExt;
use std::process::Command;

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

    // Detach: drop our handle so the daemon outlives the CLI process.
    // We call try_wait() to acknowledge the child without blocking.
    let _ = child.try_wait();

    print_banner(pid);
}

fn print_banner(pid: u32) {
    let b = "\x1b[94m"; // Bright blue — tile 1 (largest)
    let g = "\x1b[92m"; // Bright green — tile 2
    let y = "\x1b[93m"; // Bright yellow — tile 3
    let re = "\x1b[91m"; // Bright red — tile 4
    let d = "\x1b[90m"; // Dim gray — frame and labels
    let w = "\x1b[1;97m"; // Bold bright white — name
    let r = "\x1b[0m"; // Reset
    let v = env!("CARGO_PKG_VERSION");
    let pid_len = pid.to_string().len();

    const W: usize = 60;
    let pad = |n: usize| " ".repeat(W.saturating_sub(n));
    let rule: String = "─".repeat(W);

    println!();
    println!("  {d}╭{rule}╮{r}");
    println!("  {d}│  ┌────────────────┐{s}│{r}", s = pad(20));
    println!(
        "  {d}│  │{b}████████{g}████████{d}│  {w}mosaico{r} {d}v{v}{s}│{r}",
        s = pad(31 + v.len())
    );
    println!(
        "  {d}│  │{b}████████{g}████████{d}│  Tiling window manager{s}│{r}",
        s = pad(43)
    );
    println!("  {d}│  │{b}████████{g}████████{d}│{s}│{r}", s = pad(20));
    println!(
        "  {d}│  │{b}████████{y}████{re}████{d}│  Daemon started (PID: {pid}){s}│{r}",
        s = pad(44 + pid_len)
    );
    println!(
        "  {d}│  │{b}████████{y}████{re}████{d}│  Config: ~/.config/mosaico/{s}│{r}",
        s = pad(48)
    );
    println!("  {d}│  └────────────────┘{s}│{r}", s = pad(20));
    println!("  {d}╰{rule}╯{r}");
    println!();
}
