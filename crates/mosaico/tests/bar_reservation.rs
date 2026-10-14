//! Integration tests for reserving the status bar's space when tiling.
//!
//! These tests require a real desktop session. They start/stop the
//! daemon and compare its tiling areas against the bar windows it draws.

use std::thread;
use std::time::Duration;

mod common;

use common::{RunningDaemon, TestInstance, require_no_user_daemon};

/// The daemon instance these tests run, separate from the user's own.
fn instance() -> &'static TestInstance {
    static INSTANCE: std::sync::OnceLock<TestInstance> = std::sync::OnceLock::new();
    INSTANCE.get_or_init(|| TestInstance::new("bar-reservation"))
}

#[allow(non_snake_case, clippy::upper_case_acronyms)]
mod win32 {
    use std::ffi::c_void;

    pub type HWND = *mut c_void;
    pub type BOOL = i32;
    pub type LPARAM = isize;

    pub const WM_SETTINGCHANGE: u32 = 0x001A;
    pub const SPI_SETWORKAREA: usize = 0x002F;

    pub type WNDENUMPROC = unsafe extern "system" fn(hwnd: HWND, lparam: LPARAM) -> BOOL;

    #[repr(C)]
    pub struct RECT {
        pub left: i32,
        pub top: i32,
        pub right: i32,
        pub bottom: i32,
    }

    #[link(name = "user32")]
    unsafe extern "system" {
        pub fn FindWindowW(class: *const u16, title: *const u16) -> HWND;
        pub fn EnumWindows(cb: WNDENUMPROC, lparam: LPARAM) -> BOOL;
        pub fn GetClassNameW(hwnd: HWND, name: *mut u16, max: i32) -> i32;
        pub fn GetWindowRect(hwnd: HWND, rect: *mut RECT) -> BOOL;
        pub fn IsWindowVisible(hwnd: HWND) -> BOOL;
        pub fn PostMessageW(hwnd: HWND, msg: u32, wparam: usize, lparam: LPARAM) -> BOOL;
    }
}

fn mosaico(args: &[&str]) -> String {
    let output = instance()
        .command(args)
        .output()
        .expect("failed to run mosaico");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn start_daemon() -> RunningDaemon<'static> {
    require_no_user_daemon();
    mosaico(&["stop"]);
    thread::sleep(Duration::from_secs(1));
    let status = instance()
        .command(&["start"])
        .spawn()
        .expect("failed to spawn mosaico")
        .wait()
        .expect("failed to wait for mosaico");
    assert!(status.success(), "daemon failed to start");
    thread::sleep(Duration::from_secs(3));
    RunningDaemon::new(instance())
}

/// Returns `(left, bottom)` of every visible bar window.
fn bar_windows() -> Vec<(i32, i32)> {
    unsafe extern "system" fn collect(hwnd: win32::HWND, lparam: win32::LPARAM) -> win32::BOOL {
        // SAFETY: lparam is the &mut Vec passed to EnumWindows below,
        // which outlives the synchronous enumeration.
        let bars = unsafe { &mut *(lparam as *mut Vec<(i32, i32)>) };
        let mut name = [0u16; 64];
        // SAFETY: the buffer length matches the passed maximum.
        let len = unsafe { win32::GetClassNameW(hwnd, name.as_mut_ptr(), 64) };
        let is_bar = String::from_utf16_lossy(&name[..len as usize]) == "MosaicoBar";
        // SAFETY: read-only queries on a handle from EnumWindows.
        if is_bar && unsafe { win32::IsWindowVisible(hwnd) } != 0 {
            let mut r = win32::RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };
            unsafe { win32::GetWindowRect(hwnd, &mut r) };
            bars.push((r.left, r.bottom));
        }
        1
    }
    let mut bars: Vec<(i32, i32)> = Vec::new();
    // SAFETY: the callback only touches `bars` during the call.
    unsafe { win32::EnumWindows(collect, &mut bars as *mut _ as win32::LPARAM) };
    bars
}

/// Parses `(x, y)` of each monitor's tiling area from `debug state`.
fn tiling_area_origins() -> Vec<(i32, i32)> {
    mosaico(&["debug", "state"])
        .lines()
        .filter_map(|line| line.split("work_area=").nth(1))
        .filter_map(|rest| {
            let geometry = rest.split(',').next()?;
            let mut parts = geometry.split('+').skip(1);
            let x = parts.next()?.parse().ok()?;
            let y = parts.next()?.parse().ok()?;
            Some((x, y))
        })
        .collect()
}

/// Tells the daemon the work area changed, as the taskbar does.
fn notify_work_area_changed() {
    let title: Vec<u16> = "MosaicoEventSink\0".encode_utf16().collect();
    // SAFETY: FindWindowW/PostMessageW are safe on any (or a null) HWND.
    unsafe {
        let sink = win32::FindWindowW(std::ptr::null(), title.as_ptr());
        assert!(!sink.is_null(), "daemon event sink window not found");
        win32::PostMessageW(sink, win32::WM_SETTINGCHANGE, win32::SPI_SETWORKAREA, 0);
    }
}

/// Asserts every monitor with a bar tiles exactly below that bar.
fn assert_tiling_starts_below_bars(when: &str) {
    let bars = bar_windows();
    let origins = tiling_area_origins();
    assert!(!origins.is_empty(), "{when}: no monitors in debug state");
    for (left, bottom) in bars {
        let (_, y) = origins
            .iter()
            .find(|(x, _)| *x == left)
            .unwrap_or_else(|| panic!("{when}: no monitor for bar at x={left}"));
        assert_eq!(*y, bottom, "{when}: tiling area on monitor at x={left}");
    }
}

#[test]
fn work_area_changes_and_retile_keep_bar_reserved_once() {
    // Arrange
    let _daemon = start_daemon();
    assert!(!tiling_area_origins().is_empty(), "daemon did not start");
    if bar_windows().is_empty() {
        eprintln!("bar disabled in config, skipping");
        mosaico(&["stop"]);
        return;
    }
    assert_tiling_starts_below_bars("after start");

    // Act: repeated taskbar notifications used to stack the bar offset.
    for _ in 0..3 {
        notify_work_area_changed();
        thread::sleep(Duration::from_millis(300));
    }
    thread::sleep(Duration::from_secs(1));
    let after_changes = tiling_area_origins();
    mosaico(&["action", "retile"]);
    thread::sleep(Duration::from_secs(1));

    // Assert
    assert_tiling_starts_below_bars("after work area changes");
    assert_eq!(tiling_area_origins(), after_changes, "retile moved areas");
    mosaico(&["stop"]);
}
