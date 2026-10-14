//! Runtime health checks for `mosaico doctor`.
//!
//! These checks inspect live system state (daemon process, monitors)
//! rather than static configuration files.

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

pub fn check_autostart(cx: &mut Context) {
    if cx.autostart.is_enabled() {
        say!(cx, "  {OK} Autostart is enabled");
    } else {
        say!(
            cx,
            "  {WARN} Autostart is disabled (run `mosaico autostart enable` to set up)"
        );
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

pub fn check_monitors() {
    match mosaico_windows::monitor::enumerate_monitors() {
        Ok(monitors) if monitors.is_empty() => {
            println!("  {FAIL} No monitors detected");
        }
        Ok(monitors) => {
            println!("  {OK} {} monitor(s) detected", monitors.len());
            for (i, m) in monitors.iter().enumerate() {
                let wa = &m.work_area;
                println!(
                    "       Monitor {i}: {}x{} at ({}, {})",
                    wa.width, wa.height, wa.x, wa.y
                );
            }
        }
        Err(e) => {
            println!("  {FAIL} Could not enumerate monitors: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::context::Exit;
    use crate::commands::test_support::{Harness, Lifetime};

    #[test]
    fn a_running_daemon_is_reported_with_its_pid() {
        // Arrange
        let mut h = Harness::new("doctor-running");
        h.world.daemon_up(42);
        h.pid_file.write(42).unwrap();

        // Act
        h.run(|cx| {
            check_daemon(cx);
            Exit::Success
        });

        // Assert
        assert!(
            h.out().contains("Daemon is running (PID: 42)"),
            "{}",
            h.out()
        );
    }

    #[test]
    fn a_stale_pid_file_is_removed() {
        // Arrange
        let mut h = Harness::new("doctor-stale");
        h.pid_file.write(42).unwrap();

        // Act
        h.run(|cx| {
            check_daemon(cx);
            Exit::Success
        });

        // Assert
        assert!(
            h.out().contains("Removed stale PID file (PID: 42)"),
            "{}",
            h.out()
        );
        assert_eq!(h.pid_file.read().unwrap(), None);
    }

    #[test]
    fn a_hung_daemon_is_flagged() {
        // Arrange
        let mut h = Harness::new("doctor-hung");
        h.world.processes.borrow_mut().insert(42, Lifetime::Forever);
        h.pid_file.write(42).unwrap();

        // Act
        h.run(|cx| {
            check_daemon(cx);
            Exit::Success
        });

        // Assert
        assert!(h.out().contains("not responding"), "{}", h.out());
    }
}
