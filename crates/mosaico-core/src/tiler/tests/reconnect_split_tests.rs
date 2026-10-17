//! A monitor that disconnects and comes back gives its windows back the
//! splits of the BSP chain they had there, whichever layout is on screen,
//! as it does the masters of its other layouts, and its bar learns of the
//! windows that fill it.

use crate::id::wid;
use crate::tiler::tests::fake_desktop::FakeWindow;
use crate::tiler::tests::{Harness, displays};
use crate::tiler::{BarReservation, Display, Input};
use crate::{Action, LayoutKind, Rect, WindowEvent};

/// The monitors of `h` are now `monitors`.
pub(super) fn show(h: &mut Harness, monitors: Vec<Display>) {
    let bar = BarReservation::default();
    h.step(Input::Displays { monitors, bar });
}

/// Connects the first `count` monitors of [`displays`].
pub(super) fn connect(h: &mut Harness, count: usize) {
    show(h, displays(count));
}

/// Moves the splits of the BSP chain on monitor `mon` of `h` to `ratios`,
/// and lays it out again.
pub(super) fn move_splits(h: &mut Harness, mon: usize, ratios: &[f64]) {
    let ws = h.tiler.monitors[mon].active_ws_mut();
    for (split, &ratio) in ratios.iter().enumerate() {
        ws.set_split_ratio(split, ratio);
    }
    h.action(Action::Retile);
}

/// Windows `ids` tiled in BSP on the second of two monitors, with the
/// splits of the chain there moved to `ratios`.
pub(super) fn moved_on_second(ids: &[u64], ratios: &[f64]) -> Harness {
    let windows: Vec<(u64, usize)> = ids.iter().map(|&id| (id, 1)).collect();
    let mut h = Harness::new(2, &windows);
    move_splits(&mut h, 1, ratios);
    h
}

/// The ratio of each split of the BSP chain on monitor `mon` of `h`.
pub(super) fn chain(h: &Harness, mon: usize) -> Vec<Option<f64>> {
    let ws = h.workspace(mon, 0);
    (0..ws.len().saturating_sub(1))
        .map(|split| ws.split_ratios().ratio(LayoutKind::Bsp, split))
        .collect()
}

#[test]
fn a_monitor_that_comes_back_gives_its_windows_back_their_splits() {
    // Arrange
    let mut h = moved_on_second(&[2, 3, 4], &[0.75, 0.25]);
    let before: Vec<Rect> = [2, 3, 4].map(|id| h.rect_of(id)).to_vec();

    // Act -- it disconnects, its windows go to the first, and it is back.
    connect(&mut h, 1);
    connect(&mut h, 2);

    // Assert
    assert_eq!(h.active(1), vec![2, 3, 4]);
    assert_eq!(chain(&h, 1), [Some(0.75), Some(0.25)]);
    let after: Vec<Rect> = [2, 3, 4].map(|id| h.rect_of(id)).to_vec();
    assert_eq!(after, before);
}

#[test]
fn a_monitor_that_comes_back_in_another_layout_gets_its_bsp_splits_back() {
    // Arrange -- the BSP chain moved, then the vertical stack on screen,
    // with its master moved too.
    let mut h = moved_on_second(&[2, 3, 4], &[0.75, 0.25]);
    let ws = h.tiler.monitors[1].active_ws_mut();
    ws.set_layout_kind(LayoutKind::VerticalStack);
    ws.set_split_ratio(0, 0.3);

    // Act
    connect(&mut h, 1);
    connect(&mut h, 2);

    // Assert -- the stack is still on screen, and both layouts kept theirs.
    let ws = h.workspace(1, 0);
    assert_eq!(ws.layout_kind(), LayoutKind::VerticalStack);
    let master = ws.split_ratios().ratio(LayoutKind::VerticalStack, 0);
    assert_eq!(master, Some(0.3));
    assert_eq!(chain(&h, 1), [Some(0.75), Some(0.25)]);
}

#[test]
fn a_monitor_that_comes_back_gets_its_splits_and_its_bar_the_window_that_fills_it() {
    // Arrange -- window 9, which the tiler leaves alone, is maximized on
    // the second monitor, whose splits moved.
    let mut h = moved_on_second(&[2, 3, 4], &[0.75, 0.25]);
    let mut player = FakeWindow::app(1);
    (player.facts, player.maximized) = (None, true);
    h.desktop.windows.insert(wid(9), player);

    // Act -- it disconnects and comes back, and then window 9 is restored.
    connect(&mut h, 1);
    connect(&mut h, 2);
    let mut states = h.tiler.bar_states("", false);
    h.tiler.mark_maximized(&mut states, &h.desktop);
    h.tiler.take_bars_changed();
    h.window(9).maximized = false;
    h.event(WindowEvent::LocationChanged { window: wid(9) });

    // Assert -- the splits are back, and so is the bar: it lets the pointer
    // through over window 9, and is drawn again once window 9 is restored,
    // as the windows that fill a monitor are noted once it is back.
    assert_eq!(chain(&h, 1), [Some(0.75), Some(0.25)]);
    assert!(states[1].maximized);
    assert!(h.tiler.take_bars_changed());
}
