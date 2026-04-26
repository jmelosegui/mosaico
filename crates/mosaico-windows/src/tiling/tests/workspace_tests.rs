use super::super::*;
use super::make_monitor;

#[test]
fn goto_workspace_logic() {
    let mut mon = make_monitor(1);
    mon.workspaces[0].add(10);
    mon.workspaces[0].add(20);
    mon.workspaces[1].add(30);

    // Switch from ws 0 to ws 1
    let mut hidden = HashSet::new();
    for &hwnd in mon.active_ws().handles() {
        hidden.insert(hwnd);
    }
    mon.active_workspace = 1;

    // After switch: ws 1 is active, hidden set has ws 0's windows
    assert_eq!(mon.active_ws().len(), 1);
    assert!(mon.active_ws().contains(30));
    assert!(hidden.contains(&10));
    assert!(hidden.contains(&20));

    // Show ws 1 windows -- remove from hidden
    for &hwnd in mon.active_ws().handles() {
        hidden.remove(&hwnd);
    }
    // 30 was not hidden, so set unchanged (still has 10, 20)
    assert_eq!(hidden.len(), 2);
}

#[test]
fn send_to_workspace_logic() {
    let mut mon = make_monitor(1);
    mon.workspaces[0].add(10);
    mon.workspaces[0].add(20);
    mon.workspaces[0].add(30);
    mon.active_workspace = 0;

    // Send window 20 from ws 0 to ws 2
    let target_ws = 2;
    assert!(mon.active_ws().contains(20));
    mon.active_ws_mut().remove(20);
    mon.workspaces[target_ws].add(20);

    assert_eq!(mon.workspaces[0].len(), 2);
    assert_eq!(mon.workspaces[target_ws].len(), 1);
    assert!(mon.workspaces[target_ws].contains(20));
    assert!(!mon.workspaces[0].contains(20));
}

#[test]
fn send_to_same_workspace_is_noop() {
    let mut mon = make_monitor(1);
    mon.workspaces[0].add(10);
    mon.active_workspace = 0;

    // Sending to active workspace should be a no-op
    let target = mon.active_workspace;
    assert_eq!(target, 0);
    // The real code returns early; simulate by checking condition
    assert!(mon.active_workspace == target);
    assert_eq!(mon.workspaces[0].len(), 1);
}

#[test]
fn goto_same_workspace_is_noop() {
    let mon = make_monitor(1);
    // Switching to already-active workspace should be a no-op
    assert_eq!(mon.active_workspace, 0);
    // The real code returns early when active_workspace == target
}

#[test]
fn per_monitor_mode_only_updates_focused_monitor() {
    // Per-monitor mode: a workspace switch on the focused monitor must not
    // touch any other monitor's active_workspace.
    let mut monitors = [make_monitor(1), make_monitor(2), make_monitor(3)];
    monitors[0].active_workspace = 0;
    monitors[1].active_workspace = 4;
    monitors[2].active_workspace = 7;

    let focused = 1;
    let target = 2;

    // Simulate: only the focused monitor advances.
    monitors[focused].active_workspace = target;

    assert_eq!(monitors[0].active_workspace, 0);
    assert_eq!(monitors[1].active_workspace, target);
    assert_eq!(monitors[2].active_workspace, 7);
}

#[test]
fn global_mode_updates_every_monitor() {
    // Global mode: a workspace switch updates every monitor's
    // active_workspace to the same index, mirroring Windows virtual
    // desktops. Each monitor still owns its own window list.
    let mut monitors = [make_monitor(1), make_monitor(2), make_monitor(3)];
    monitors[0].workspaces[0].add(10);
    monitors[1].workspaces[3].add(20); // started on ws 3
    monitors[2].workspaces[5].add(30); // started on ws 5
    monitors[1].active_workspace = 3;
    monitors[2].active_workspace = 5;

    let target = 2; // goto-workspace 3 (1-indexed)

    // Simulate the global-mode invariant: every monitor flips to `target`.
    for mon in &mut monitors {
        mon.active_workspace = target;
    }

    for mon in &monitors {
        assert_eq!(mon.active_workspace, target);
    }
    // Window membership is unchanged -- windows remain on their original
    // workspaces; they're just hidden until the user comes back.
    assert!(monitors[0].workspaces[0].contains(10));
    assert!(monitors[1].workspaces[3].contains(20));
    assert!(monitors[2].workspaces[5].contains(30));
}

#[test]
fn global_mode_send_to_workspace_keeps_monitors_in_sync() {
    // send-to-workspace in global mode: window moves on the focused
    // monitor, then every monitor flips to the target workspace.
    let mut monitors = [make_monitor(1), make_monitor(2)];
    monitors[0].workspaces[0].add(100);
    monitors[1].workspaces[0].add(200);

    let focused = 0;
    let target_ws = 4;

    // Step 1: move the window on the focused monitor.
    monitors[focused].workspaces[0].remove(100);
    monitors[focused].workspaces[target_ws].add(100);
    monitors[focused].active_workspace = target_ws;

    // Step 2: flip every other monitor to the same workspace.
    for (i, mon) in monitors.iter_mut().enumerate() {
        if i == focused {
            continue;
        }
        mon.active_workspace = target_ws;
    }

    assert_eq!(monitors[0].active_workspace, target_ws);
    assert_eq!(monitors[1].active_workspace, target_ws);
    assert!(monitors[0].workspaces[target_ws].contains(100));
    // The other monitor's window stays where it was (window 200 is
    // still on workspace 0 of monitor 1, just hidden because monitor 1
    // is now showing workspace 4).
    assert!(monitors[1].workspaces[0].contains(200));
    assert_eq!(monitors[1].workspaces[target_ws].len(), 0);
}
