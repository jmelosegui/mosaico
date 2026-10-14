use std::time::Duration;

use super::context::{Context, Exit, complain, say};

/// How long to wait for a freshly launched daemon to answer.
const STARTUP_TIMEOUT: Duration = Duration::from_secs(10);

/// How often to check whether it is up yet.
const POLL_INTERVAL: Duration = Duration::from_millis(50);

pub fn execute(cx: &mut Context) -> Exit {
    if cx.daemon.is_running() {
        say!(cx, "Mosaico is already running.");
        return Exit::Success;
    }

    // Clean up a stale PID file from a previous unclean shutdown.
    if let Ok(Some(pid)) = cx.pid_file.read() {
        if cx.processes.is_alive(pid) {
            say!(
                cx,
                "Mosaico process exists (PID: {pid}) but is not responding."
            );
            return Exit::Success;
        }
        let _ = cx.pid_file.remove();
    }

    let pid = match cx.processes.launch_daemon() {
        Ok(pid) => pid,
        Err(e) => {
            complain!(cx, "Error: {e}");
            return Exit::Failure;
        }
    };

    // Confirm it is actually up before claiming it started. The daemon
    // takes the single instance lock first thing, and a daemon that is
    // still shutting down holds that lock well after `mosaico stop`
    // returned, so a start issued in that window is rejected and exits.
    // Reporting success for a pid that is already dead is how that
    // window went unnoticed.
    if !wait_for_daemon(cx, pid) {
        complain!(cx, "Error: the daemon exited immediately after starting.");
        complain!(
            cx,
            "A previous daemon may still be shutting down and holding the single instance lock."
        );
        complain!(cx, "Check 'mosaico status', then try again.");
        return Exit::Failure;
    }

    print_banner(cx, pid);
    Exit::Success
}

/// Waits until the launched daemon answers on its IPC channel.
///
/// Returns false if it exits first, which is what a daemon rejected by
/// the single instance guard does, or if it never answers in time.
fn wait_for_daemon(cx: &Context, pid: u32) -> bool {
    let deadline = cx.clock.now() + STARTUP_TIMEOUT;
    loop {
        if cx.daemon.is_running() {
            return true;
        }
        if !cx.processes.is_alive(pid) || cx.clock.now() >= deadline {
            return false;
        }
        cx.clock.sleep(POLL_INTERVAL);
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

fn print_banner(cx: &mut Context, pid: u32) {
    let d = "\x1b[90m"; // Dim gray: labels
    let w = "\x1b[1;97m"; // Bold bright white: values
    let r = "\x1b[0m"; // Reset
    let tip = TIPS[pid as usize % TIPS.len()];

    (cx.logo)(&mut *cx.out);
    say!(cx, "");
    say!(cx, "  {d}Config{r}   {}", cx.config_dir.display());
    say!(cx, "  {d}Daemon{r}   Started (PID: {w}{pid}{r})");
    say!(cx, "  {d}Repo{r}     https://github.com/jmelosegui/mosaico");
    say!(cx, "  {d}Tip{r}      {tip}");

    if let Some(remote) = (cx.check_update)() {
        let y = "\x1b[33m"; // Yellow
        let local = env!("CARGO_PKG_VERSION");
        say!(cx, "");
        say!(cx, "  {y}Update available: v{local} -> {remote}{r}");
        say!(cx, "  {y}Run 'mosaico update' to install it{r}");
    }

    say!(cx, "");
}

#[cfg(test)]
#[path = "tests/start_tests.rs"]
mod tests;
