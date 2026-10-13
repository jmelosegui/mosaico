//! Integration tests for keeping the layout across display changes.
//!
//! These tests require a real desktop session. They start/stop the
//! daemon and compare where it keeps each window before and after.

use std::process::Command;
use std::sync::{Mutex, MutexGuard};
use std::thread;
use std::time::Duration;

#[allow(non_snake_case, clippy::upper_case_acronyms)]
mod win32 {
    use std::ffi::c_void;

    pub type HWND = *mut c_void;
    pub type BOOL = i32;
    pub type LPARAM = isize;

    pub const WM_DISPLAYCHANGE: u32 = 0x007E;

    #[link(name = "user32")]
    unsafe extern "system" {
        pub fn FindWindowW(class: *const u16, title: *const u16) -> HWND;
        pub fn PostMessageW(hwnd: HWND, msg: u32, wparam: usize, lparam: LPARAM) -> BOOL;
    }
}

/// `(monitor key, workspace number, window handles in order)`.
type Placement = Vec<(String, u32, Vec<String>)>;

fn mosaico(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_mosaico"))
        .args(args)
        .output()
        .expect("failed to run mosaico");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// Both tests drive the one daemon, so they must not overlap.
static DAEMON: Mutex<()> = Mutex::new(());

/// Keeps the daemon running for one test and stops it afterwards, also
/// when the test panics. A daemon left behind keeps tiling the desktop
/// and holds the test harness's output pipe open.
struct Daemon<'a> {
    _lock: MutexGuard<'a, ()>,
}

impl Drop for Daemon<'_> {
    fn drop(&mut self) {
        mosaico(&["stop"]);
    }
}

fn start_daemon() -> Daemon<'static> {
    let lock = DAEMON.lock().unwrap_or_else(|e| e.into_inner());
    mosaico(&["stop"]);
    thread::sleep(Duration::from_secs(1));
    let status = Command::new(env!("CARGO_BIN_EXE_mosaico"))
        .arg("start")
        .spawn()
        .expect("failed to spawn mosaico")
        .wait()
        .expect("failed to wait for mosaico");
    assert!(status.success(), "daemon failed to start");
    thread::sleep(Duration::from_secs(3));
    Daemon { _lock: lock }
}

/// Reads which windows the daemon keeps on which monitor and workspace.
fn placement() -> Placement {
    let mut out: Placement = Vec::new();
    let mut key = String::new();
    for line in mosaico(&["debug", "state"]).lines() {
        if let Some(rest) = line.strip_prefix("Monitor ") {
            key = rest
                .split("key=")
                .nth(1)
                .and_then(|k| k.split(", work_area=").next())
                .unwrap_or_default()
                .to_string();
        } else if let Some(rest) = line.trim_start().strip_prefix("Workspace ") {
            let n = rest
                .split(' ')
                .next()
                .and_then(|n| n.parse().ok())
                .unwrap_or(0);
            out.push((key.clone(), n, Vec::new()));
        } else if line.trim_start().starts_with("0x")
            && let Some(last) = out.last_mut()
        {
            let hwnd = line.split_whitespace().next().unwrap_or_default();
            last.2.push(hwnd.to_string());
        }
    }
    out.sort();
    out
}

fn monitor_count() -> usize {
    mosaico(&["debug", "state"])
        .lines()
        .filter(|l| l.starts_with("Monitor "))
        .count()
}

/// Tells the daemon the display configuration changed, as Windows does.
fn notify_display_changed() {
    let title: Vec<u16> = "MosaicoEventSink\0".encode_utf16().collect();
    // SAFETY: FindWindowW/PostMessageW are safe on any (or a null) HWND.
    unsafe {
        let sink = win32::FindWindowW(std::ptr::null(), title.as_ptr());
        assert!(!sink.is_null(), "daemon event sink window not found");
        win32::PostMessageW(sink, win32::WM_DISPLAYCHANGE, 0, 0);
    }
}

fn display_switch(mode: &str) {
    Command::new("DisplaySwitch.exe")
        .arg(mode)
        .status()
        .expect("failed to run DisplaySwitch");
    thread::sleep(Duration::from_secs(8));
}

#[test]
fn burst_of_display_changes_keeps_every_window_in_place() {
    // Arrange
    let _daemon = start_daemon();
    let before = placement();
    assert!(monitor_count() > 0, "daemon did not start");

    // Act: a KVM or dock sends several notifications in a row.
    for _ in 0..5 {
        notify_display_changed();
        thread::sleep(Duration::from_millis(100));
    }
    thread::sleep(Duration::from_secs(2));

    // Assert
    assert_eq!(placement(), before, "windows moved on a no-op change");
}

/// Disconnects every monitor but one with `DisplaySwitch /internal`, then
/// reconnects them with `/extend`, and checks every window came back.
///
/// Ignored by default because it really reconfigures the displays. Run
/// it on a machine with two or more monitors in extend mode:
/// `cargo test -p mosaico --test display_change -- --ignored`
#[test]
#[ignore = "reconfigures the real displays"]
fn monitor_round_trip_restores_the_layout() {
    // Arrange
    let _daemon = start_daemon();
    if monitor_count() < 2 {
        eprintln!("needs two or more monitors, skipping");
        return;
    }
    let before = placement();

    // Act
    display_switch("/internal");
    let away = monitor_count();
    display_switch("/extend");

    // Assert
    assert_eq!(away, 1, "DisplaySwitch /internal did not remove a monitor");
    assert_eq!(placement(), before, "layout not restored after reconnect");
}
