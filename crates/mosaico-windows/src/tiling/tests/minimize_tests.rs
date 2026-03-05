use super::super::*;
use super::{find_window_in, make_monitors};

#[test]
fn minimize_removes_window_from_active_workspace() {
    let mut monitors = make_monitors(1);
    monitors[0].workspaces[0].add(100);
    monitors[0].workspaces[0].add(200);

    // Simulate the Minimized event handler: remove the window from
    // the active workspace.
    let hwnd = 100;
    let (mon_idx, ws_idx) = find_window_in(&monitors, hwnd).unwrap();
    assert_eq!(ws_idx, monitors[mon_idx].active_workspace);
    monitors[mon_idx].workspaces[ws_idx].remove(hwnd);

    assert!(!monitors[0].workspaces[0].contains(100));
    assert!(monitors[0].workspaces[0].contains(200));
    assert_eq!(monitors[0].workspaces[0].len(), 1);
}

#[test]
fn minimize_focused_window_clears_focus() {
    let mut monitors = make_monitors(1);
    monitors[0].workspaces[0].add(100);
    let mut focused_window: Option<usize> = Some(100);

    // Simulate the Minimized event handler for the focused window.
    let hwnd = 100;
    if let Some((mon_idx, ws_idx)) = find_window_in(&monitors, hwnd) {
        if ws_idx == monitors[mon_idx].active_workspace {
            monitors[mon_idx].workspaces[ws_idx].remove(hwnd);
            // The bug fix: clear focused_window when the minimized
            // window was focused.
            if focused_window == Some(hwnd) {
                focused_window = None;
            }
        }
    }

    assert_eq!(focused_window, None, "focus should be cleared after minimizing the focused window");
    assert!(!monitors[0].workspaces[0].contains(100));
}

#[test]
fn minimize_unfocused_window_preserves_focus() {
    let mut monitors = make_monitors(1);
    monitors[0].workspaces[0].add(100);
    monitors[0].workspaces[0].add(200);
    let mut focused_window: Option<usize> = Some(200);

    // Minimize window 100 while window 200 is focused.
    let hwnd = 100;
    if let Some((mon_idx, ws_idx)) = find_window_in(&monitors, hwnd) {
        if ws_idx == monitors[mon_idx].active_workspace {
            monitors[mon_idx].workspaces[ws_idx].remove(hwnd);
            if focused_window == Some(hwnd) {
                focused_window = None;
            }
        }
    }

    assert_eq!(focused_window, Some(200), "focus should remain on the other window");
    assert!(!monitors[0].workspaces[0].contains(100));
    assert!(monitors[0].workspaces[0].contains(200));
}

#[test]
fn minimize_ignores_window_hidden_by_switch() {
    let mut monitors = make_monitors(1);
    monitors[0].workspaces[0].add(100);
    let mut hidden_by_switch = HashSet::new();
    hidden_by_switch.insert(100_usize);

    // Simulate the Minimized event for a window that was hidden by
    // workspace switch — should be ignored.
    let hwnd = 100;
    let mut removed = false;
    if !hidden_by_switch.contains(&hwnd) {
        if let Some((mon_idx, ws_idx)) = find_window_in(&monitors, hwnd) {
            if ws_idx == monitors[mon_idx].active_workspace {
                monitors[mon_idx].workspaces[ws_idx].remove(hwnd);
                removed = true;
            }
        }
    }

    assert!(!removed, "window hidden by switch should not be removed");
    assert!(monitors[0].workspaces[0].contains(100));
}

#[test]
fn minimize_ignores_window_on_inactive_workspace() {
    let mut monitors = make_monitors(1);
    monitors[0].workspaces[1].add(100); // window on ws 1
    monitors[0].active_workspace = 0; // ws 0 is active

    // Simulate the Minimized event — should only act on active workspace.
    let hwnd = 100;
    let mut removed = false;
    if let Some((mon_idx, ws_idx)) = find_window_in(&monitors, hwnd) {
        if ws_idx == monitors[mon_idx].active_workspace {
            monitors[mon_idx].workspaces[ws_idx].remove(hwnd);
            removed = true;
        }
    }

    assert!(!removed, "window on inactive workspace should not be removed");
    assert!(monitors[0].workspaces[1].contains(100));
}
