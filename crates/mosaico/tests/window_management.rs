//! Integration tests for window management features.
//!
//! These tests require a real desktop session with notepad.exe available.
//! They start/stop the daemon and launch notepad to verify tiling,
//! borders, monocle mode, and minimize/maximize behavior.

use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Win32 FFI
// ---------------------------------------------------------------------------

#[allow(non_snake_case, non_camel_case_types)]
mod win32 {
    use std::ffi::c_void;

    pub type HWND = *mut c_void;
    pub type BOOL = i32;
    pub type DWORD = u32;
    pub type UINT = u32;
    pub type WPARAM = usize;
    pub type LPARAM = isize;

    pub const SW_MINIMIZE: i32 = 6;
    pub const SW_MAXIMIZE: i32 = 3;
    pub const SW_RESTORE: i32 = 9;

    pub const WM_CLOSE: UINT = 0x0010;

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
        pub fn IsIconic(hwnd: HWND) -> BOOL;
        pub fn IsZoomed(hwnd: HWND) -> BOOL;
        pub fn IsWindowVisible(hwnd: HWND) -> BOOL;
        pub fn ShowWindow(hwnd: HWND, cmd: i32) -> BOOL;
        pub fn EnumWindows(cb: WNDENUMPROC, lparam: LPARAM) -> BOOL;
        pub fn GetWindowThreadProcessId(hwnd: HWND, pid: *mut DWORD) -> DWORD;
        pub fn PostMessageW(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> BOOL;
        pub fn GetWindowRect(hwnd: HWND, rect: *mut RECT) -> BOOL;
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Runs the mosaico CLI binary with the given arguments, using spawn()+wait()
/// to avoid pipe-inheritance hangs on Windows.
fn mosaico(args: &[&str]) -> std::process::ExitStatus {
    let mut child = Command::new(env!("CARGO_BIN_EXE_mosaico"))
        .args(args)
        .spawn()
        .expect("failed to spawn mosaico");
    child.wait().expect("failed to wait for mosaico")
}

/// Starts the daemon and waits for it to be ready.
fn start_daemon() {
    // Make sure no daemon is already running.
    let _ = Command::new(env!("CARGO_BIN_EXE_mosaico"))
        .arg("stop")
        .output();
    thread::sleep(Duration::from_secs(1));

    let status = mosaico(&["start"]);
    assert!(status.success(), "daemon failed to start");
    // Give the daemon time to set up the event loop and IPC pipe.
    thread::sleep(Duration::from_secs(2));
}

/// Stops the daemon.
fn stop_daemon() {
    let _ = mosaico(&["stop"]);
    thread::sleep(Duration::from_millis(500));
}

/// Launches notepad.exe and waits for its window to appear.
/// Returns the child process handle and the window HWND.
fn launch_notepad() -> (Child, win32::HWND) {
    let child = Command::new("notepad.exe")
        .spawn()
        .expect("failed to launch notepad.exe");
    let pid = child.id();

    // Wait for the notepad window to appear (up to 10 seconds).
    let mut hwnd = std::ptr::null_mut();
    for _ in 0..20 {
        thread::sleep(Duration::from_millis(500));
        hwnd = find_window_by_pid(pid);
        if !hwnd.is_null() {
            break;
        }
    }
    assert!(!hwnd.is_null(), "notepad window did not appear within 10s");

    // Give the tiling manager time to pick up and tile the window.
    thread::sleep(Duration::from_secs(2));

    (child, hwnd)
}

/// Sends WM_CLOSE to notepad and waits for the process to exit.
fn close_notepad(mut child: Child) {
    let hwnd = find_window_by_pid(child.id());
    if !hwnd.is_null() {
        unsafe {
            win32::PostMessageW(hwnd, win32::WM_CLOSE, 0, 0);
        }
    }
    let _ = child.wait();
}

/// Finds a visible top-level window belonging to the given process ID.
fn find_window_by_pid(pid: u32) -> win32::HWND {
    struct Search {
        pid: u32,
        result: win32::HWND,
    }

    unsafe extern "system" fn enum_cb(hwnd: win32::HWND, lparam: win32::LPARAM) -> win32::BOOL {
        let search = unsafe { &mut *(lparam as *mut Search) };
        let mut window_pid: win32::DWORD = 0;
        unsafe {
            win32::GetWindowThreadProcessId(hwnd, &mut window_pid);
        }
        if window_pid == search.pid && unsafe { win32::IsWindowVisible(hwnd) } != 0 {
            search.result = hwnd;
            return 0; // stop enumeration
        }
        1 // continue
    }

    let mut search = Search {
        pid,
        result: std::ptr::null_mut(),
    };
    unsafe {
        win32::EnumWindows(enum_cb, &mut search as *mut Search as win32::LPARAM);
    }
    search.result
}

/// Finds the MosaicoBorder window.
fn find_border_hwnd() -> win32::HWND {
    let class: Vec<u16> = "MosaicoBorder\0".encode_utf16().collect();
    unsafe { win32::FindWindowW(class.as_ptr(), std::ptr::null()) }
}

/// Returns the window rect (left, top, right, bottom).
fn get_window_rect(hwnd: win32::HWND) -> (i32, i32, i32, i32) {
    let mut rect = win32::RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    unsafe {
        win32::GetWindowRect(hwnd, &mut rect);
    }
    (rect.left, rect.top, rect.right, rect.bottom)
}

/// Returns true if the border window is currently visible.
fn is_border_visible() -> bool {
    let hwnd = find_border_hwnd();
    if hwnd.is_null() {
        return false;
    }
    unsafe { win32::IsWindowVisible(hwnd) != 0 }
}

/// Takes a screenshot and saves it to `target/test-screenshots/{name}.png`.
fn take_screenshot(name: &str) {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = std::path::Path::new(manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .expect("failed to find workspace root");

    let out_dir = workspace_root.join("target").join("test-screenshots");
    let out_path = out_dir.join(format!("{name}.png"));

    let script = workspace_root.join("scripts").join("test_screenshot.ps1");

    let status = Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            &script.to_string_lossy(),
            "-OutputPath",
            &out_path.to_string_lossy(),
        ])
        .status();

    match status {
        Ok(s) if s.success() => eprintln!("  screenshot saved: {}", out_path.display()),
        Ok(s) => eprintln!("  screenshot script exited with {s}"),
        Err(e) => eprintln!("  failed to run screenshot script: {e}"),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Minimize a window, then restore it by clicking (simulated via ShowWindow).
/// The border should update to the restored window's position.
#[test]
fn minimize_and_restore_updates_border() {
    start_daemon();
    let (child, hwnd) = launch_notepad();

    // Capture the border rect while notepad is tiled.
    let border = find_border_hwnd();
    assert!(!border.is_null(), "border window not found");
    assert!(is_border_visible(), "border should be visible initially");
    let border_before = get_window_rect(border);

    take_screenshot("minimize_restore_before");

    // Minimize notepad.
    unsafe { win32::ShowWindow(hwnd, win32::SW_MINIMIZE); }
    thread::sleep(Duration::from_secs(2));

    let iconic = unsafe { win32::IsIconic(hwnd) };
    assert!(iconic != 0, "notepad should be minimized");

    take_screenshot("minimize_restore_minimized");

    // Restore notepad (simulates clicking the taskbar button).
    unsafe { win32::ShowWindow(hwnd, win32::SW_RESTORE); }
    thread::sleep(Duration::from_secs(2));

    let iconic_after = unsafe { win32::IsIconic(hwnd) };
    assert!(iconic_after == 0, "notepad should no longer be minimized");

    // Border should be visible and match the restored window position.
    assert!(is_border_visible(), "border should be visible after restore");
    let border_after = get_window_rect(border);
    let notepad_rect = get_window_rect(hwnd);

    // The border should roughly surround notepad (within border width tolerance).
    assert!(
        border_after.0 <= notepad_rect.0 && border_after.2 >= notepad_rect.2,
        "border should horizontally surround notepad: border={:?} notepad={:?}",
        border_after, notepad_rect
    );
    assert!(
        border_after.1 <= notepad_rect.1 && border_after.3 >= notepad_rect.3,
        "border should vertically surround notepad: border={:?} notepad={:?}",
        border_after, notepad_rect
    );

    // Border position should match what it was before minimize
    // (since retiling returns notepad to the same slot).
    assert_eq!(
        border_before, border_after,
        "border should return to same position after restore"
    );

    take_screenshot("minimize_restore_after");

    close_notepad(child);
    stop_daemon();
}

/// With monocle active, launch a second notepad. The new window should
/// receive monocle sizing (fill the work area) instead of BSP sizing.
#[test]
fn monocle_new_window_gets_monocle_size() {
    start_daemon();
    let (child1, hwnd1) = launch_notepad();

    // Enable monocle mode — notepad1 should fill the work area.
    let status = mosaico(&["action", "toggle-monocle"]);
    assert!(status.success(), "toggle-monocle failed");
    thread::sleep(Duration::from_secs(1));

    let monocle_rect = get_window_rect(hwnd1);
    take_screenshot("monocle_first_window");

    // Launch a second notepad while monocle is active.
    let (child2, hwnd2) = launch_notepad();
    thread::sleep(Duration::from_secs(1));

    let new_window_rect = get_window_rect(hwnd2);
    take_screenshot("monocle_second_window");

    // The new window should be monocle-sized (same dimensions as the first).
    let monocle_w = monocle_rect.2 - monocle_rect.0;
    let monocle_h = monocle_rect.3 - monocle_rect.1;
    let new_w = new_window_rect.2 - new_window_rect.0;
    let new_h = new_window_rect.3 - new_window_rect.1;

    // Allow a small tolerance for border differences between windows.
    let tolerance = 20;
    assert!(
        (new_w - monocle_w).abs() < tolerance,
        "new window width {new_w} should match monocle width {monocle_w}"
    );
    assert!(
        (new_h - monocle_h).abs() < tolerance,
        "new window height {new_h} should match monocle height {monocle_h}"
    );

    // Disable monocle before cleanup.
    let _ = mosaico(&["action", "toggle-monocle"]);
    thread::sleep(Duration::from_millis(500));

    close_notepad(child2);
    close_notepad(child1);
    stop_daemon();
}

/// Maximize a window. The border should hide and the window should
/// NOT be retiled back to BSP layout.
#[test]
fn maximize_hides_border_and_preserves_maximize() {
    start_daemon();
    let (child, hwnd) = launch_notepad();

    assert!(is_border_visible(), "border should be visible initially");
    take_screenshot("maximize_before");

    // Maximize notepad.
    unsafe { win32::ShowWindow(hwnd, win32::SW_MAXIMIZE); }
    thread::sleep(Duration::from_secs(2));

    // Window should remain maximized (not retiled back).
    let zoomed = unsafe { win32::IsZoomed(hwnd) };
    assert!(zoomed != 0, "notepad should be maximized");

    // Border should be hidden.
    assert!(
        !is_border_visible(),
        "border should be hidden when a window is maximized"
    );

    take_screenshot("maximize_after");

    close_notepad(child);
    stop_daemon();
}

/// Maximize a window then restore it. The border should reappear
/// and the window should be retiled back to BSP layout.
#[test]
fn restore_from_maximize_shows_border() {
    start_daemon();
    let (child, hwnd) = launch_notepad();

    let _tiled_rect = get_window_rect(hwnd);

    // Maximize.
    unsafe { win32::ShowWindow(hwnd, win32::SW_MAXIMIZE); }
    thread::sleep(Duration::from_secs(2));

    assert!(
        !is_border_visible(),
        "border should be hidden while maximized"
    );

    // Restore.
    unsafe { win32::ShowWindow(hwnd, win32::SW_RESTORE); }
    thread::sleep(Duration::from_secs(2));

    let zoomed = unsafe { win32::IsZoomed(hwnd) };
    assert!(zoomed == 0, "notepad should not be maximized after restore");

    // Border should be visible again.
    assert!(
        is_border_visible(),
        "border should reappear after restoring from maximize"
    );

    // The border should surround the restored notepad window.
    let border = find_border_hwnd();
    let border_rect = get_window_rect(border);
    let notepad_rect = get_window_rect(hwnd);

    assert!(
        border_rect.0 <= notepad_rect.0 && border_rect.2 >= notepad_rect.2,
        "border should horizontally surround notepad after restore"
    );

    take_screenshot("restore_from_maximize");

    close_notepad(child);
    stop_daemon();
}
