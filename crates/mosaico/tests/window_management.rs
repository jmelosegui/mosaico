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

#[allow(non_snake_case, non_camel_case_types, clippy::upper_case_acronyms)]
mod win32 {
    use std::ffi::c_void;

    pub type HWND = *mut c_void;
    pub type BOOL = i32;
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

    pub const GW_OWNER: UINT = 4;
    pub const PM_REMOVE: UINT = 0x0001;

    #[repr(C)]
    pub struct MSG {
        pub hwnd: HWND,
        pub message: UINT,
        pub wparam: WPARAM,
        pub lparam: LPARAM,
        pub time: u32,
        pub pt_x: i32,
        pub pt_y: i32,
    }

    #[link(name = "user32")]
    unsafe extern "system" {
        pub fn FindWindowW(class: *const u16, title: *const u16) -> HWND;
        pub fn IsIconic(hwnd: HWND) -> BOOL;
        pub fn IsZoomed(hwnd: HWND) -> BOOL;
        pub fn IsWindowVisible(hwnd: HWND) -> BOOL;
        pub fn ShowWindow(hwnd: HWND, cmd: i32) -> BOOL;
        pub fn EnumWindows(cb: WNDENUMPROC, lparam: LPARAM) -> BOOL;
        pub fn PostMessageW(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> BOOL;
        pub fn GetWindowRect(hwnd: HWND, rect: *mut RECT) -> BOOL;
        pub fn SetForegroundWindow(hwnd: HWND) -> BOOL;
        pub fn GetWindow(hwnd: HWND, cmd: UINT) -> HWND;
        pub fn RealGetWindowClassW(hwnd: HWND, string: *mut u16, max_count: UINT) -> UINT;
        pub fn CreateWindowExW(
            ex_style: u32,
            class_name: *const u16,
            window_name: *const u16,
            style: u32,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
            parent: HWND,
            menu: *mut c_void,
            instance: *mut c_void,
            param: *mut c_void,
        ) -> HWND;
        pub fn DestroyWindow(hwnd: HWND) -> BOOL;
        pub fn PeekMessageW(msg: *mut MSG, hwnd: HWND, min: UINT, max: UINT, remove: UINT) -> BOOL;
        pub fn TranslateMessage(msg: *const MSG) -> BOOL;
        pub fn DispatchMessageW(msg: *const MSG) -> isize;
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
///
/// Also closes any leftover Notepad windows from previous tests
/// to prevent Win11 session restore from creating extra windows.
fn start_daemon() {
    // Make sure no daemon is already running.
    let _ = Command::new(env!("CARGO_BIN_EXE_mosaico"))
        .arg("stop")
        .output();
    thread::sleep(Duration::from_secs(1));

    // Close leftover notepads gracefully so session restore
    // doesn't pollute the next test.
    close_all_notepads();

    let status = mosaico(&["start"]);
    assert!(status.success(), "daemon failed to start");
    // Give the daemon time to set up the event loop and IPC pipe.
    thread::sleep(Duration::from_secs(3));
}

/// Stops the daemon.
fn stop_daemon() {
    let _ = mosaico(&["stop"]);
    thread::sleep(Duration::from_millis(500));
}

/// Collects all visible top-level windows with class "Notepad" that
/// are large enough to be actual application windows.
///
/// On Windows 11, the UWP notepad creates multiple "Notepad"-class
/// windows per instance (tabs, frame, etc.).  Only the main window
/// is large enough to tile.  We filter by a minimum size to skip
/// helper windows.
fn find_notepad_windows() -> Vec<win32::HWND> {
    const MIN_SIZE: i32 = 100;

    struct Search {
        results: Vec<win32::HWND>,
    }

    unsafe extern "system" fn enum_cb(hwnd: win32::HWND, lparam: win32::LPARAM) -> win32::BOOL {
        let search = unsafe { &mut *(lparam as *mut Search) };
        unsafe {
            if win32::IsWindowVisible(hwnd) == 0 {
                return 1;
            }
            let mut buf = [0u16; 256];
            let len = win32::RealGetWindowClassW(hwnd, buf.as_mut_ptr(), 256);
            let class = String::from_utf16_lossy(&buf[..len as usize]);
            if class != "Notepad" {
                return 1;
            }
            // Skip small helper/tab windows created by UWP notepad.
            let mut rect = win32::RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };
            win32::GetWindowRect(hwnd, &mut rect);
            let w = rect.right - rect.left;
            let h = rect.bottom - rect.top;
            if w >= MIN_SIZE && h >= MIN_SIZE {
                search.results.push(hwnd);
            }
        }
        1
    }

    let mut search = Search {
        results: Vec::new(),
    };
    unsafe {
        win32::EnumWindows(enum_cb, &mut search as *mut Search as win32::LPARAM);
    }
    search.results
}

/// Launches notepad.exe and waits for its window to appear.
///
/// Diffs the set of "Notepad"-class windows before and after spawning
/// to find the new window.  This works even when notepad.exe is the
/// Windows Store/UWP version that hosts all windows under a single
/// process (different from the spawned PID).
fn launch_notepad() -> (Child, win32::HWND) {
    let before = find_notepad_windows();

    let child = Command::new("notepad.exe")
        .spawn()
        .expect("failed to launch notepad.exe");

    // Wait for a new Notepad-class window to appear (up to 10 seconds).
    let mut hwnd = std::ptr::null_mut();
    for _ in 0..20 {
        thread::sleep(Duration::from_millis(500));
        let after = find_notepad_windows();
        for w in &after {
            if !before.contains(w) {
                hwnd = *w;
                break;
            }
        }
        if !hwnd.is_null() {
            break;
        }
    }
    assert!(!hwnd.is_null(), "notepad window did not appear within 10s");

    // Give the tiling manager time to pick up and tile the window.
    thread::sleep(Duration::from_secs(2));

    (child, hwnd)
}

/// Sends WM_CLOSE to the notepad window and waits for it to close.
///
/// Win11 Notepad saves session state on graceful close.  If we
/// force-kill instead, it will restore all "unsaved" windows next
/// time it opens, polluting subsequent tests.  We wait up to 5s
/// for the window to disappear before giving up.
fn close_notepad(hwnd: win32::HWND, mut child: Child) {
    if !hwnd.is_null() {
        unsafe {
            win32::PostMessageW(hwnd, win32::WM_CLOSE, 0, 0);
        }
        // Wait for the window to actually disappear.
        for _ in 0..10 {
            thread::sleep(Duration::from_millis(500));
            if unsafe { win32::IsWindowVisible(hwnd) } == 0 {
                break;
            }
        }
    }
    let _ = child.kill();
    let _ = child.wait();
}

/// Gracefully closes ALL visible Notepad windows by sending WM_CLOSE.
///
/// Prevents Win11 Notepad's session restore from polluting the next
/// test with leftover windows.
fn close_all_notepads() {
    let windows = find_notepad_windows();
    for &hwnd in &windows {
        unsafe {
            win32::PostMessageW(hwnd, win32::WM_CLOSE, 0, 0);
        }
    }
    if !windows.is_empty() {
        // Wait for all notepad windows to close gracefully.
        for _ in 0..10 {
            thread::sleep(Duration::from_millis(500));
            if find_notepad_windows().is_empty() {
                break;
            }
        }
    }
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

/// Switches to workspace 8 so the test runs with only its own
/// notepad window(s), free from interference by other desktop windows.
fn isolate_workspace() {
    mosaico(&["action", "go-to-workspace", "8"]);
    thread::sleep(Duration::from_millis(500));
}

/// Minimize a window, then restore it by clicking (simulated via ShowWindow).
/// The border should update to the restored window's position.
#[test]
fn minimize_and_restore_updates_border() {
    start_daemon();
    isolate_workspace();
    let (child, hwnd) = launch_notepad();

    assert!(is_border_visible(), "border should be visible initially");

    take_screenshot("minimize_restore_before");

    // Minimize notepad.
    unsafe {
        win32::ShowWindow(hwnd, win32::SW_MINIMIZE);
    }
    thread::sleep(Duration::from_secs(2));

    let iconic = unsafe { win32::IsIconic(hwnd) };
    assert!(iconic != 0, "notepad should be minimized");

    take_screenshot("minimize_restore_minimized");

    // Restore notepad (simulates clicking the taskbar button).
    unsafe {
        win32::ShowWindow(hwnd, win32::SW_RESTORE);
    }
    thread::sleep(Duration::from_secs(2));

    let iconic_after = unsafe { win32::IsIconic(hwnd) };
    assert!(iconic_after == 0, "notepad should no longer be minimized");

    // Border should be visible and surround the restored notepad.
    assert!(
        is_border_visible(),
        "border should be visible after restore"
    );
    assert_border_surrounds("after restore", hwnd);

    take_screenshot("minimize_restore_after");

    close_notepad(hwnd, child);
    stop_daemon();
}

/// With monocle active, launch a second notepad. The new window should
/// receive monocle sizing (fill the work area) instead of BSP sizing.
#[test]
fn monocle_new_window_gets_monocle_size() {
    start_daemon();
    isolate_workspace();
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

    close_notepad(hwnd2, child2);
    close_notepad(hwnd1, child1);
    stop_daemon();
}

/// Maximize a window. The border should hide and the window should
/// NOT be retiled back to BSP layout.
#[test]
fn maximize_hides_border_and_preserves_maximize() {
    start_daemon();
    isolate_workspace();
    let (child, hwnd) = launch_notepad();

    assert!(is_border_visible(), "border should be visible initially");
    take_screenshot("maximize_before");

    // Maximize notepad.
    unsafe {
        win32::ShowWindow(hwnd, win32::SW_MAXIMIZE);
    }
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

    close_notepad(hwnd, child);
    stop_daemon();
}

/// Maximize a window then restore it. The border should reappear
/// and the window should be retiled back to BSP layout.
#[test]
fn restore_from_maximize_shows_border() {
    start_daemon();
    isolate_workspace();
    let (child, hwnd) = launch_notepad();

    let _tiled_rect = get_window_rect(hwnd);

    // Maximize.
    unsafe {
        win32::ShowWindow(hwnd, win32::SW_MAXIMIZE);
    }
    thread::sleep(Duration::from_secs(2));

    assert!(
        !is_border_visible(),
        "border should be hidden while maximized"
    );

    // Restore.
    unsafe {
        win32::ShowWindow(hwnd, win32::SW_RESTORE);
    }
    thread::sleep(Duration::from_secs(2));

    let zoomed = unsafe { win32::IsZoomed(hwnd) };
    assert!(zoomed == 0, "notepad should not be maximized after restore");

    // Border should be visible again.
    assert!(
        is_border_visible(),
        "border should reappear after restoring from maximize"
    );

    // The border should surround the restored notepad window.
    assert_border_surrounds("after restore from maximize", hwnd);

    take_screenshot("restore_from_maximize");

    close_notepad(hwnd, child);
    stop_daemon();
}

// ---------------------------------------------------------------------------
// Dialog helpers
// ---------------------------------------------------------------------------

const WS_POPUP: u32 = 0x80000000;
const WS_VISIBLE: u32 = 0x10000000;
const WS_CAPTION_STYLE: u32 = 0x00C00000;
const WS_EX_DLGMODALFRAME: u32 = 0x00000001;

/// Creates a simple owned popup window that acts as a test dialog.
///
/// Uses `CreateWindowExW` with the owner set atomically at creation
/// time, avoiding the Win32 race condition where `GetWindow(GW_OWNER)`
/// returns NULL during `EVENT_OBJECT_CREATE` for cross-thread dialogs.
///
/// Unlike `MessageBoxA`, this is non-modal and will not disable the
/// owner window, making it safe for testing even if the test crashes.
fn create_test_dialog(owner: win32::HWND) -> win32::HWND {
    let class: Vec<u16> = "Static\0".encode_utf16().collect();
    let title: Vec<u16> = "MosaicoTestDialog\0".encode_utf16().collect();
    unsafe {
        win32::CreateWindowExW(
            WS_EX_DLGMODALFRAME,
            class.as_ptr(),
            title.as_ptr(),
            WS_POPUP | WS_VISIBLE | WS_CAPTION_STYLE,
            100,
            100,
            300,
            200,
            owner,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    }
}

/// Drains the calling thread's message queue so pending activation
/// and focus messages are processed.  Without this, windows created
/// on the test thread never receive `WM_ACTIVATE` and Windows never
/// fires `EVENT_SYSTEM_FOREGROUND` for them.
fn pump_messages() {
    unsafe {
        let mut msg: win32::MSG = std::mem::zeroed();
        while win32::PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, win32::PM_REMOVE) != 0 {
            win32::TranslateMessage(&msg);
            win32::DispatchMessageW(&msg);
        }
    }
}

/// Destroys the test dialog window.
fn close_test_dialog(hwnd: win32::HWND) {
    if !hwnd.is_null() {
        unsafe {
            win32::DestroyWindow(hwnd);
        }
    }
}

/// Asserts the border roughly surrounds the given window.
///
/// The daemon positions the border around the DWM visible rect,
/// but the test queries `GetWindowRect` which includes invisible
/// DWM borders (~7-10 physical px, ~3-5 logical px at high DPI).
/// A tolerance of 5 logical pixels covers this offset while still
/// catching real positioning bugs (which would be off by hundreds).
fn assert_border_surrounds(label: &str, target: win32::HWND) {
    const TOL: i32 = 5;
    let border = find_border_hwnd();
    assert!(!border.is_null(), "{label}: border window not found");
    let br = get_window_rect(border);
    let tr = get_window_rect(target);
    assert!(
        br.0 <= tr.0 + TOL && br.2 >= tr.2 - TOL,
        "{label}: border should horizontally surround target: border={br:?} target={tr:?}"
    );
    assert!(
        br.1 <= tr.1 + TOL && br.3 >= tr.3 - TOL,
        "{label}: border should vertically surround target: border={br:?} target={tr:?}"
    );
}

// ---------------------------------------------------------------------------
// Dialog tests
// ---------------------------------------------------------------------------

/// An owned dialog should not be tiled. The notepad window should
/// stay at the same position and the border should still surround it.
#[test]
fn owned_dialog_is_not_tiled() {
    start_daemon();
    isolate_workspace();
    let (child, hwnd) = launch_notepad();

    assert!(is_border_visible(), "border should be visible initially");
    let notepad_before = get_window_rect(hwnd);

    take_screenshot("dialog_not_tiled_before");

    // Create an owned popup dialog.
    let dialog = create_test_dialog(hwnd);
    assert!(!dialog.is_null(), "failed to create test dialog");
    thread::sleep(Duration::from_secs(2));

    take_screenshot("dialog_not_tiled_dialog_open");

    // The dialog should be owned by notepad.
    let owner = unsafe { win32::GetWindow(dialog, win32::GW_OWNER) };
    assert_eq!(owner, hwnd, "dialog should be owned by notepad");

    // The dialog should not have been tiled — notepad should still
    // be at the same position (not retiled for a new window).
    let notepad_after = get_window_rect(hwnd);
    assert_eq!(
        notepad_before, notepad_after,
        "notepad should not have been retiled when dialog appeared"
    );

    // Border should still be visible and surround notepad.
    assert!(is_border_visible(), "border should still be visible");
    assert_border_surrounds("border on notepad with dialog open", hwnd);

    // Close the dialog.
    close_test_dialog(dialog);
    thread::sleep(Duration::from_secs(1));

    take_screenshot("dialog_not_tiled_after");

    close_notepad(hwnd, child);
    stop_daemon();
}

/// Focusing an owned dialog should move the border to the dialog's
/// owner window, not leave it on a previously focused window.
#[test]
fn focus_dialog_moves_border_to_owner() {
    start_daemon();
    isolate_workspace();
    let (child1, hwnd1) = launch_notepad();
    let (child2, hwnd2) = launch_notepad();

    // Border should be on notepad2 (last launched/focused).
    assert!(is_border_visible(), "border should be visible");
    assert_border_surrounds("initial focus on notepad2", hwnd2);

    take_screenshot("dialog_focus_two_notepads");

    // Create a dialog owned by notepad1 and focus it.
    let dialog = create_test_dialog(hwnd1);
    assert!(!dialog.is_null(), "failed to create test dialog");
    unsafe {
        win32::SetForegroundWindow(dialog);
    }
    // Drain the message queue so WM_ACTIVATE is processed and
    // Windows fires EVENT_SYSTEM_FOREGROUND for the dialog.
    pump_messages();
    thread::sleep(Duration::from_secs(2));

    take_screenshot("dialog_focus_dialog_open");

    // The dialog got focus, so the border should now surround
    // notepad1 (the owner), not notepad2.
    assert!(
        is_border_visible(),
        "border should be visible with dialog focused"
    );
    assert_border_surrounds("border follows owner of focused dialog", hwnd1);

    take_screenshot("dialog_focus_after_close");

    // Close the dialog.
    close_test_dialog(dialog);
    pump_messages();
    thread::sleep(Duration::from_secs(1));

    close_notepad(hwnd2, child2);
    close_notepad(hwnd1, child1);
    stop_daemon();
}
