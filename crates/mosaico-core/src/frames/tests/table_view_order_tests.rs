//! Changes of a window's show state posted one after the other: the note
//! of the last one goes only once the window has been seen otherwise, as
//! the one ahead of it may still be queued.

use super::*;

/// Whether window 1 is to be in view once what was posted lands.
fn in_view(table: &mut FrameTable<i32>, probe: &Probe) -> Option<bool> {
    table.in_view_once_landed(wid(1), probe)
}

#[test]
fn a_show_posted_behind_a_hide_to_a_window_in_view_is_noted_until_both_land() {
    // Arrange -- in view, with a hide and then a show posted.
    let mut table = FrameTable::default();
    let mut probe = Probe::new(true);
    table.posted_show_state(wid(1), &probe, probe.now, false);
    table.posted_show_state(wid(1), &probe, probe.now, true);

    // Act -- seen in view before either lands, then hidden, then shown.
    let queued = in_view(&mut table, &probe);
    probe.window().shown = false;
    let hidden = in_view(&mut table, &probe);
    probe.window().shown = true;
    let shown = in_view(&mut table, &probe);

    // Assert
    assert_eq!([queued, hidden, shown], [Some(true), Some(true), None]);
}

#[test]
fn a_hide_posted_behind_a_show_to_a_window_out_of_view_is_noted_until_both_land() {
    // Arrange -- out of view, with a show and then a hide posted.
    let mut table = FrameTable::default();
    let mut probe = Probe::new(true);
    probe.window().shown = false;
    table.posted_show_state(wid(1), &probe, probe.now, true);
    table.posted_show_state(wid(1), &probe, probe.now, false);

    // Act -- seen out of view before either lands, then shown, then hidden.
    let queued = in_view(&mut table, &probe);
    probe.window().shown = true;
    let shown = in_view(&mut table, &probe);
    probe.window().shown = false;
    let hidden = in_view(&mut table, &probe);

    // Assert
    assert_eq!([queued, shown, hidden], [Some(false), Some(false), None]);
}
