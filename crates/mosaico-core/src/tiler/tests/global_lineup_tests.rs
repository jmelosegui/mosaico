//! Global workspaces through a display change (#124).
//!
//! In global mode every monitor shows the same workspace number. A
//! display change keeps the one the monitors showed, and a monitor that
//! connects joins it, even when the monitor that had focus is the one
//! replaced. A monitor that still switches workspace hides and shows its
//! windows the way a workspace switch does. Every window is hidden or
//! shown once, even when the change moves it twice.

use crate::Direction;
use crate::config::{HidingBehaviour, WorkspaceMode};

use super::*;

/// Where the only window on monitor 0 goes, with the default gap.
const PRIMARY_SLOT: Rect = Rect {
    x: 8,
    y: 8,
    width: MON_W - 16,
    height: MON_H - 16,
};

fn settings(how: HidingBehaviour, mode: WorkspaceMode) -> Settings {
    let mut settings = Settings::default();
    settings.layout.hiding = how;
    settings.workspaces.mode = mode;
    settings
}

/// Connects the first `count` monitors of [`displays`].
fn connect(h: &mut Harness, count: usize) {
    h.step(Input::Displays {
        monitors: displays(count),
        bar: BarReservation::default(),
    });
}

/// Replaces monitor 1 with another monitor at the same position, in one
/// display change, as a dock or a KVM switch can.
fn replace_monitor_1(h: &mut Harness) {
    let mut monitors = displays(2);
    monitors[1].key = key("other");
    monitors[1].id = MonitorId::from_raw(7);
    h.step(Input::Displays {
        monitors,
        bar: BarReservation::default(),
    });
}

/// Windows 1 and 2 sit on workspace 1 of monitors 0 and 1, and window 3
/// on workspace 2 of monitor 0. Both monitors show workspace 2, so
/// windows 1 and 2 are hidden `how`, and focus is on monitor 1.
fn both_monitors_on_workspace_2(how: HidingBehaviour, mode: WorkspaceMode) -> Harness {
    let mut h = Harness::with_settings(settings(how, mode), 2, &[(1, 0), (2, 1)]);
    h.action(Action::GoToWorkspace(2));
    h.open(3, 0);
    h.action(Action::Focus(Direction::Right));
    // Only per-monitor mode switches monitor 1 here.
    h.action(Action::GoToWorkspace(2));
    h
}

/// Monitor 0 shows workspace 1, with window 2, and has window 1 hidden
/// `how` on workspace 2. Monitor 1 shows workspace 2, with focus, and has
/// window 3 hidden on workspace 1. Per-monitor mode allows that.
///
/// The mode is then set to global without lining the monitors up. No
/// input leaves them out of step in global mode since a reload that turns
/// it on switches them (#128), but a display change must still hide and
/// show the windows of a monitor it switches.
fn out_of_step(how: HidingBehaviour) -> Harness {
    let settings = settings(how, WorkspaceMode::PerMonitor);
    let mut h = Harness::with_settings(settings, 2, &[(1, 0), (2, 0), (3, 1)]);
    // Window 1 has focus at the start.
    h.action(Action::SendToWorkspace(2));
    h.action(Action::GoToWorkspace(1));
    h.event(WindowEvent::Focused { window: wid(3) });
    h.action(Action::GoToWorkspace(2));
    h.tiler.workspace_mode = WorkspaceMode::Global;
    h
}

/// The index of the workspace each monitor shows: 1 is workspace 2.
fn in_view(h: &Harness) -> Vec<usize> {
    h.tiler
        .monitors()
        .iter()
        .map(MonitorState::active_workspace)
        .collect()
}

/// Whether window `id` is out of view the way `how` hides it.
fn hidden(h: &Harness, id: u64, how: HidingBehaviour) -> bool {
    let w = &h.desktop.windows[&wid(id)];
    match how {
        HidingBehaviour::Cloak => w.cloaked,
        HidingBehaviour::Hide => !w.visible,
        HidingBehaviour::Minimize => w.minimized,
    }
}

/// Whether window `id` is in view.
fn in_sight(h: &Harness, id: u64) -> bool {
    let w = &h.desktop.windows[&wid(id)];
    w.visible && !w.cloaked && !w.minimized
}

/// The windows the last step hid, and the ones it showed, in order.
fn hid_and_shown(h: &Harness) -> (Vec<u64>, Vec<u64>) {
    let (mut hid, mut shown) = (Vec::new(), Vec::new());
    for effect in &h.effects {
        match *effect {
            Effect::Hide { window, .. } => hid.push(window.raw()),
            Effect::Show { window, .. } => shown.push(window.raw()),
            _ => {}
        }
    }
    (hid, shown)
}

/// Replaces monitor 1 while it has focus, and checks that monitor 0
/// keeps workspace 2 in view as it was and the new monitor joins it.
fn replace_the_focused_monitor(how: HidingBehaviour) {
    // Arrange
    let mut h = both_monitors_on_workspace_2(how, WorkspaceMode::Global);

    // Act
    replace_monitor_1(&mut h);

    // Assert -- #124: both monitors used to switch to workspace 1, the new
    // monitor's, without hiding or showing anything.
    assert_eq!(in_view(&h), vec![1, 1]);
    assert_eq!(h.active(0), vec![3]);
    assert!(in_sight(&h, 3));
    assert_eq!(h.rect_of(3), PRIMARY_SLOT);
    // Window 2 moved to monitor 0's workspace 1, where window 1 is.
    let ws1 = h.workspace(0, 0);
    assert!(ws1.contains(wid(1)) && ws1.contains(wid(2)));
    assert!(hidden(&h, 1, how) && hidden(&h, 2, how));
    assert_eq!(hid_and_shown(&h).1, Vec::<u64>::new());
}

#[test]
fn replacing_the_focused_monitor_keeps_the_workspace_in_view() {
    replace_the_focused_monitor(HidingBehaviour::Cloak);
}

#[test]
fn replacing_the_focused_monitor_keeps_hidden_windows_hidden() {
    replace_the_focused_monitor(HidingBehaviour::Hide);
}

#[test]
fn replacing_the_focused_monitor_keeps_minimized_windows_minimized() {
    replace_the_focused_monitor(HidingBehaviour::Minimize);
}

#[test]
fn a_new_monitor_joins_the_workspace_in_view() {
    // Arrange -- focus is on monitor 1, which stays.
    let how = HidingBehaviour::Cloak;
    let mut h = both_monitors_on_workspace_2(how, WorkspaceMode::Global);

    // Act
    connect(&mut h, 3);

    // Assert
    assert_eq!(in_view(&h), vec![1, 1, 1]);
    assert_eq!(h.active(0), vec![3]);
    assert_eq!(hid_and_shown(&h), (vec![], vec![]));
}

#[test]
fn a_returning_monitor_joins_the_workspace_in_view() {
    // Arrange -- window 2 waits on monitor 0 while monitor 1 is away,
    // and is hidden there when monitor 0 goes to workspace 3.
    let how = HidingBehaviour::Cloak;
    let settings = settings(how, WorkspaceMode::Global);
    let mut h = Harness::with_settings(settings, 2, &[(1, 0), (2, 1)]);
    connect(&mut h, 1);
    h.action(Action::GoToWorkspace(3));

    // Act
    connect(&mut h, 2);

    // Assert -- window 2 goes back to monitor 1's workspace 1, which is
    // no longer in view, and is hidden there once.
    assert_eq!(in_view(&h), vec![2, 2]);
    assert!(h.workspace(1, 0).contains(wid(2)));
    assert_eq!(hid_and_shown(&h), (vec![2], vec![]));
    assert!(hidden(&h, 2, how));
}

#[test]
fn per_monitor_mode_leaves_a_new_monitor_on_workspace_1() {
    // Arrange
    let how = HidingBehaviour::Cloak;
    let mut h = both_monitors_on_workspace_2(how, WorkspaceMode::PerMonitor);

    // Act
    replace_monitor_1(&mut h);

    // Assert -- each monitor keeps its own workspace, and the new one
    // starts on workspace 1.
    assert_eq!(in_view(&h), vec![1, 0]);
    assert_eq!(h.active(0), vec![3]);
    assert!(h.workspace(0, 0).contains(wid(2)));
    assert!(hidden(&h, 1, how) && hidden(&h, 2, how));
    assert_eq!(hid_and_shown(&h).1, Vec::<u64>::new());
}

/// Connects a third monitor while monitor 0 is out of step, and checks
/// that monitor 0 switches to workspace 2 the way a workspace switch
/// does. Returns where window 1 was placed in its slot, and shown.
fn line_up_monitor_0(how: HidingBehaviour) -> (Option<usize>, Option<usize>) {
    // Arrange
    let mut h = out_of_step(how);

    // Act
    connect(&mut h, 3);

    // Assert -- #124: monitor 0 used to switch without hiding or showing.
    assert_eq!(in_view(&h), vec![1, 1, 1]);
    assert_eq!(hid_and_shown(&h), (vec![2], vec![1]));
    assert_eq!(h.active(0), vec![1]);
    assert!(in_sight(&h, 1) && hidden(&h, 2, how) && hidden(&h, 3, how));
    let slot = PRIMARY_SLOT;
    let place = h.effects.iter().position(
        |e| matches!(*e, Effect::Place { window, rect, .. } if window == wid(1) && rect == slot),
    );
    let show = h
        .effects
        .iter()
        .position(|e| matches!(*e, Effect::Show { window, .. } if window == wid(1)));
    (place, show)
}

#[test]
fn a_monitor_lining_up_shows_the_workspace_it_switches_to() {
    let (place, show) = line_up_monitor_0(HidingBehaviour::Cloak);
    assert!(place.is_some() && place < show, "placed, then uncloaked");
}

#[test]
fn a_monitor_lining_up_places_hidden_windows_before_showing_them() {
    let (place, show) = line_up_monitor_0(HidingBehaviour::Hide);
    assert!(place.is_some() && place < show, "placed hidden, then shown");
}

#[test]
fn a_monitor_lining_up_restores_minimized_windows_before_placing_them() {
    let (place, show) = line_up_monitor_0(HidingBehaviour::Minimize);
    assert!(show.is_some() && show < place, "restored, then placed");
}

/// Window 1 sits on the laptop, monitor 0. Windows 2 and 3 sit on the
/// second monitor, 2 hidden `how` on workspace 2 and 3 on workspace 1,
/// which is in view. Undocking sends windows 2 and 3 to the laptop. Then
/// docking with the lid closed, in one change, takes the laptop away,
/// brings the second monitor back as the primary, and adds a new one.
fn dock_with_the_lid_closed(how: HidingBehaviour) -> Harness {
    let settings = settings(how, WorkspaceMode::Global);
    let mut h = Harness::with_settings(settings, 2, &[(1, 0), (2, 1), (3, 1)]);
    h.event(WindowEvent::Focused { window: wid(2) });
    h.action(Action::SendToWorkspace(2));
    h.action(Action::GoToWorkspace(1));
    connect(&mut h, 1);
    let mut monitors = displays(2);
    monitors[0].key = key("mon1");
    monitors[0].id = MonitorId::from_raw(5);
    monitors[1].key = key("new");
    monitors[1].id = MonitorId::from_raw(6);
    h.step(Input::Displays {
        monitors,
        bar: BarReservation::default(),
    });
    h
}

/// Docks with the lid closed, and checks that each window is hidden or
/// shown once.
fn dock(how: HidingBehaviour) {
    // Arrange and act
    let h = dock_with_the_lid_closed(how);

    // Assert -- the second monitor shows workspace 1 again, with windows
    // 3 and 1, and window 2 is back on its workspace 2.
    assert_eq!(in_view(&h), vec![0, 0]);
    assert_eq!(h.active(0), vec![3, 1]);
    assert!(h.workspace(0, 1).contains(wid(2)));
    // The change moves windows 2 and 3 twice: off the laptop, then back
    // home to the second monitor. They used to be hidden or shown twice.
    assert_eq!(hid_and_shown(&h), (vec![2], vec![1, 3]));
    assert!(in_sight(&h, 1) && in_sight(&h, 3) && hidden(&h, 2, how));
}

#[test]
fn docking_with_the_lid_closed_handles_each_window_once_when_cloaking() {
    dock(HidingBehaviour::Cloak);
}

#[test]
fn docking_with_the_lid_closed_handles_each_window_once_when_hiding() {
    dock(HidingBehaviour::Hide);
}

#[test]
fn docking_with_the_lid_closed_handles_each_window_once_when_minimizing() {
    dock(HidingBehaviour::Minimize);
}
