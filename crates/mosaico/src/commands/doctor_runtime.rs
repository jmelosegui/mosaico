//! Runtime health checks for `mosaico doctor`.
//!
//! These checks inspect live system state (daemon process, monitors)
//! rather than static configuration files.

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

pub fn check_daemon() {
    if mosaico_windows::ipc::is_daemon_running() {
        if let Ok(Some(pid)) = mosaico_core::pid::read_pid_file() {
            println!("  {OK} Daemon is running (PID: {pid})");
        } else {
            println!("  {OK} Daemon is running");
        }
        return;
    }
    if let Ok(Some(pid)) = mosaico_core::pid::read_pid_file() {
        if mosaico_windows::process::is_process_alive(pid) {
            println!("  {WARN} Process exists (PID: {pid}) but not responding");
        } else {
            let _ = mosaico_core::pid::remove_pid_file();
            println!("  {FIXED} Removed stale PID file (PID: {pid})");
        }
    } else {
        println!("  {WARN} Daemon is not running");
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
