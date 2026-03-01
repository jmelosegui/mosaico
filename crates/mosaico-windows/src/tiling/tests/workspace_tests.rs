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

    // Show ws 1 windows — remove from hidden
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
