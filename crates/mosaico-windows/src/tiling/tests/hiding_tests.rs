use super::super::*;
use super::make_monitor;

#[test]
fn cloak_mode_does_not_populate_hidden_by_switch() {
    // Arrange
    let mut mon = make_monitor(1);
    mon.workspaces[0].add(10);
    mon.workspaces[0].add(20);
    mon.workspaces[1].add(30);

    let hiding = HidingBehaviour::Cloak;
    let mut hidden_by_switch = HashSet::new();

    // Act — simulate goto_workspace with Cloak mode
    for &hwnd in mon.active_ws().handles() {
        if hiding != HidingBehaviour::Cloak {
            hidden_by_switch.insert(hwnd);
        }
    }
    mon.active_workspace = 1;

    // Assert — hidden_by_switch must stay empty for Cloak
    assert!(hidden_by_switch.is_empty());
}

#[test]
fn hide_mode_populates_hidden_by_switch() {
    // Arrange
    let mut mon = make_monitor(1);
    mon.workspaces[0].add(10);
    mon.workspaces[0].add(20);
    mon.workspaces[1].add(30);

    let hiding = HidingBehaviour::Hide;
    let mut hidden_by_switch = HashSet::new();

    // Act — simulate goto_workspace with Hide mode
    for &hwnd in mon.active_ws().handles() {
        if hiding != HidingBehaviour::Cloak {
            hidden_by_switch.insert(hwnd);
        }
    }
    mon.active_workspace = 1;

    // Assert — hidden_by_switch must contain ws 0's windows
    assert_eq!(hidden_by_switch.len(), 2);
    assert!(hidden_by_switch.contains(&10));
    assert!(hidden_by_switch.contains(&20));
}

#[test]
fn minimize_mode_populates_hidden_by_switch() {
    // Arrange
    let mut mon = make_monitor(1);
    mon.workspaces[0].add(10);

    let hiding = HidingBehaviour::Minimize;
    let mut hidden_by_switch = HashSet::new();

    // Act
    for &hwnd in mon.active_ws().handles() {
        if hiding != HidingBehaviour::Cloak {
            hidden_by_switch.insert(hwnd);
        }
    }

    // Assert
    assert!(hidden_by_switch.contains(&10));
}
