//! Runtime health checks for `mosaico doctor`.
//!
//! These checks inspect live system state (autostart registration,
//! daemon process, monitors) rather than static configuration files.

use mosaico_core::system::AutostartStatus;

use super::autostart;
use super::context::{Context, say};

const OK: &str = "\x1b[32m[ok]\x1b[0m";
const WARN: &str = "\x1b[33m[warn]\x1b[0m";
const FAIL: &str = "\x1b[31m[fail]\x1b[0m";
const FIXED: &str = "\x1b[36m[fixed]\x1b[0m";

pub fn check_rules_cache_age() {
    let Some(path) = mosaico_core::config::rules_path() else {
        return;
    };
    if !path.exists() {
        println!("  {WARN} Community rules not cached (will download on first start)");
        return;
    }
    let age = path
        .metadata()
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.elapsed().ok());
    match age {
        Some(d) if d.as_secs() < 86_400 => {
            println!("  {OK} Community rules cached (updated today)");
        }
        Some(d) => {
            let days = d.as_secs() / 86_400;
            println!("  {WARN} Community rules cached ({days} day(s) ago)");
        }
        None => {
            println!("  {WARN} Community rules cached (unknown age)");
        }
    }
}

/// Reports whether this mosaico starts at login. Only reports: another
/// registration may be one the user wants, such as a development build.
pub fn check_autostart(cx: &mut Context) {
    match autostart::check(cx) {
        Ok(AutostartStatus::Enabled) => say!(cx, "  {OK} Autostart is enabled"),
        Ok(AutostartStatus::Disabled) => say!(
            cx,
            "  {WARN} Autostart is disabled (run `mosaico autostart enable` to set up)"
        ),
        Ok(AutostartStatus::Other {
            registered,
            expected,
        }) => say!(
            cx,
            "  {WARN} Autostart starts {registered} instead of {expected} (run `mosaico autostart enable` to switch)"
        ),
        Ok(AutostartStatus::Missing(command)) => say!(
            cx,
            "  {FAIL} Autostart starts \"{}\", which is not a file (run `mosaico autostart enable` to fix)",
            command.program.display()
        ),
        Err(e) => say!(cx, "  {FAIL} Autostart: {e}"),
    }
}

pub fn check_daemon(cx: &mut Context) {
    let pid = cx.pid_file.read().ok().flatten();
    if cx.daemon.is_running() {
        match pid {
            Some(pid) => say!(cx, "  {OK} Daemon is running (PID: {pid})"),
            None => say!(cx, "  {OK} Daemon is running"),
        }
        return;
    }
    match pid {
        Some(pid) if cx.processes.is_alive(pid) => {
            say!(
                cx,
                "  {WARN} Process exists (PID: {pid}) but not responding"
            );
        }
        Some(pid) => {
            let _ = cx.pid_file.remove();
            say!(cx, "  {FIXED} Removed stale PID file (PID: {pid})");
        }
        None => say!(cx, "  {WARN} Daemon is not running"),
    }
}

pub fn check_monitors(cx: &mut Context) {
    match cx.desktop.monitors() {
        Ok(monitors) if monitors.is_empty() => {
            say!(cx, "  {FAIL} No monitors detected");
        }
        Ok(monitors) => {
            say!(cx, "  {OK} {} monitor(s) detected", monitors.len());
            for (i, m) in monitors.iter().enumerate() {
                let wa = &m.work_area;
                say!(
                    cx,
                    "       Monitor {i}: {}x{} at ({}, {})",
                    wa.width,
                    wa.height,
                    wa.x,
                    wa.y
                );
            }
        }
        Err(e) => {
            say!(cx, "  {FAIL} Could not enumerate monitors: {e}");
        }
    }
}

#[cfg(test)]
#[path = "tests/doctor_runtime_tests.rs"]
mod tests;
