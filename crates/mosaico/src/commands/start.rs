use std::time::Duration;

use mosaico_core::update;

use super::context::{Context, Exit, complain, say};

/// How long to wait for a freshly launched daemon to answer.
const STARTUP_TIMEOUT: Duration = Duration::from_secs(10);

/// How often to check whether it is up yet.
const POLL_INTERVAL: Duration = Duration::from_millis(50);

pub fn execute(cx: &mut Context) -> Exit {
    announced_as(cx, env!("CARGO_PKG_VERSION"))
}

/// Starts the daemon like [`execute`], but prints nothing unless it
/// fails and does not look for a newer release. Autostart runs this at
/// login, where nobody reads the banner and the network may not be up
/// yet, so the lookup would only keep the login waiting.
pub fn quietly(cx: &mut Context) -> Exit {
    launch(cx, None)
}

/// Starts the daemon like [`execute`], showing `version` in the banner.
/// After an update the daemon runs the new binary while the process
/// starting it is still the old one, so it announces the new version.
pub fn announced_as(cx: &mut Context, version: &str) -> Exit {
    launch(cx, Some(version))
}

/// Starts the daemon unless it runs already, then shows the banner for
/// `banner`, the version to announce, if there is one. Failures are
/// reported either way.
fn launch(cx: &mut Context, banner: Option<&str>) -> Exit {
    if cx.daemon.is_running() {
        if banner.is_some() {
            say!(cx, "Mosaico is already running.");
        }
        return Exit::Success;
    }

    // Clean up a stale PID file from a previous unclean shutdown.
    if let Ok(Some(pid)) = cx.pid_file.read() {
        if cx.processes.is_alive(pid) {
            if banner.is_some() {
                say!(
                    cx,
                    "Mosaico process exists (PID: {pid}) but is not responding."
                );
            }
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

    if let Some(version) = banner {
        print_banner(cx, pid, version);
    }
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

fn print_banner(cx: &mut Context, pid: u32, version: &str) {
    let d = "\x1b[90m"; // Dim gray: labels
    let w = "\x1b[1;97m"; // Bold bright white: values
    let r = "\x1b[0m"; // Reset
    let tip = TIPS[pid as usize % TIPS.len()];

    (cx.logo)(&mut *cx.out, version);
    say!(cx, "");
    say!(cx, "  {d}Config{r}   {}", cx.config.dir().display());
    say!(cx, "  {d}Daemon{r}   Started (PID: {w}{pid}{r})");
    say!(cx, "  {d}Repo{r}     https://github.com/jmelosegui/mosaico");
    say!(cx, "  {d}Tip{r}      {tip}");

    if let Ok(Some(remote)) = update::newer_release(cx.http, version) {
        let y = "\x1b[33m"; // Yellow
        say!(cx, "");
        say!(cx, "  {y}Update available: v{version} -> {remote}{r}");
        say!(cx, "  {y}Run 'mosaico update' to install it{r}");
    }

    say!(cx, "");
}

#[cfg(test)]
#[path = "tests/start_tests.rs"]
mod tests;
