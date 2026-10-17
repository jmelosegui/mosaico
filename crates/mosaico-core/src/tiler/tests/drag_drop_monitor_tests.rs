//! Drops onto another monitor's workspace: in monocle, in global mode,
//! and one the system maximizes there; and a drop over a tile of the
//! window's own monitor while most of it, rescaled, is on another.

use crate::config::WorkspaceMode;
use crate::id::wid;
use crate::tiler::Effect;
use crate::tiler::tests::{Harness, MON_H, MON_W};
use crate::{Action, Rect, WindowEvent};

use super::super::support::{drag_to, drop_at, grab, move_to, places, swapping};

/// Window 1, of windows 1 and 2 on the first monitor, dragged to the top
/// edge of the second, where window 3 is, and maximized there by the
/// system as it is dropped. Returns what the drop does.
fn maximized_on_second(h: &mut Harness) -> Vec<Effect> {
    let top = (MON_W + MON_W / 2, 0);
    grab(h, 1);
    move_to(h, 1, top);
    h.window(1).rect = Rect::new(MON_W, 0, MON_W, MON_H);
    h.window(1).maximized = true;
    drop_at(h, 1, top)
}

#[test]
fn a_window_dropped_on_a_monocle_workspace_joins_it_behind_nothing() {
    // Arrange -- the second monitor shows window 3 in monocle, window 4
    // behind it; then window 1 on the first takes focus back.
    let windows = [(1, 0), (2, 0), (3, 1), (4, 1)];
    let mut h = Harness::with_settings(swapping(), 2, &windows);
    h.event(WindowEvent::Focused { window: wid(3) });
    h.action(Action::ToggleMonocle);
    h.event(WindowEvent::Focused { window: wid(1) });
    let front = h.rect_of(3);

    // Act -- onto the monocle window.
    let effects = drag_to(&mut h, 1, (front.center_x(), front.center_y()));

    // Assert -- as for any window dragged onto a monocle workspace, it
    // joins the end, only the monocle window is placed there, and nothing
    // is put behind it.
    assert_eq!(h.active(1), vec![3, 4, 1]);
    assert!(places(&effects, 3), "{effects:?}");
    assert!(!places(&effects, 1) && !places(&effects, 4), "{effects:?}");
    let behind = |e: &Effect| matches!(e, Effect::PutBehind { .. });
    assert!(!effects.iter().any(behind), "{effects:?}");
}

#[test]
fn in_global_mode_a_window_dropped_on_another_monitor_switches_no_workspace() {
    // Arrange
    let mut settings = swapping();
    settings.workspaces.mode = WorkspaceMode::Global;
    let mut h = Harness::with_settings(settings, 2, &[(1, 0), (2, 0), (3, 1)]);
    let alone = h.rect_of(3);

    // Act
    let effects = drag_to(&mut h, 1, (alone.center_x(), alone.center_y()));

    // Assert -- it joins the workspace on screen there, and every monitor
    // stays on it.
    assert_eq!(h.active(1), vec![1, 3]);
    let monitors = h.tiler.monitors();
    assert!(monitors.iter().all(|m| m.active_workspace() == 0));
    let switches = |e: &Effect| matches!(e, Effect::Hide { .. } | Effect::Show { .. });
    assert!(!effects.iter().any(switches), "{effects:?}");
}

#[test]
fn a_window_the_system_maximizes_on_another_monitor_joins_it_maximized() {
    // Arrange
    let mut h = Harness::with_settings(swapping(), 2, &[(1, 0), (2, 0), (3, 1)]);

    // Act
    let effects = maximized_on_second(&mut h);

    // Assert -- it joins the workspace there, as a half snap there does,
    // and stays maximized: nothing places it.
    assert_eq!((h.active(0), h.active(1)), (vec![2], vec![3, 1]));
    assert!(!places(&effects, 1), "{effects:?}");
    assert!(h.window(1).maximized);
}

#[test]
fn a_window_dropped_maximized_on_another_monitor_is_laid_out_there_once_restored() {
    // Arrange
    let mut h = Harness::with_settings(swapping(), 2, &[(1, 0), (2, 0), (3, 1)]);
    maximized_on_second(&mut h);

    // Act -- the user takes it out of maximized.
    h.window(1).maximized = false;
    let effects = h
        .event(WindowEvent::LocationChanged { window: wid(1) })
        .to_vec();

    // Assert -- into its slot there.
    assert!(places(&effects, 1), "{effects:?}");
    assert!(h.rect_of(1).x >= MON_W);
}

#[test]
fn a_window_rescaled_across_two_monitors_over_a_tile_of_its_own_swaps_with_it() {
    // Arrange -- grabbed near its right edge and dragged over window 2,
    // by the edge of the first monitor, until most of it is on the second,
    // where the system scales it up for a monitor of another scale.
    let mut h = Harness::with_settings(swapping(), 2, &[(1, 0), (2, 0), (3, 1)]);
    let right = h.rect_of(2);
    let at = (MON_W - 20, right.center_y());
    grab(&mut h, 1);
    h.window(1).rect = Rect::new(MON_W - 300, right.y, 1422, 1000);
    h.window(1).monitor = 1;

    // Act
    drop_at(&mut h, 1, at);

    // Assert -- a move onto window 2, not a snap.
    assert_eq!(h.active(0), vec![2, 1]);
}
