//! A monitor that disconnects and comes back gives its windows back the
//! heights of its stacks, as it gives them back the splits of the BSP
//! chain: down the vertical stack, and down each column of the three
//! columns, but never a column's height to a window now in the other
//! column.

use crate::id::wid;
use crate::tiler::tests::Harness;
use crate::tiler::{BarReservation, Input};
use crate::{LayoutKind, Rect, WindowEvent};

use super::split_tests::connect;

/// Window 1 on the first of two monitors, and windows 2 to 6 on the second
/// in `kind`, the columns of the windows at the indices of `resized` there
/// at their heights.
fn stacked_on_second(kind: LayoutKind, resized: &[(usize, &[f64])]) -> Harness {
    let windows = [(1, 0), (2, 1), (3, 1), (4, 1), (5, 1), (6, 1)];
    let mut h = Harness::new(2, &windows);
    let ws = h.tiler.monitors[1].active_ws_mut();
    ws.set_layout_kind(kind);
    for &(index, shares) in resized {
        ws.set_heights(index, shares);
    }
    h.step(Input::Bar(BarReservation::default()));
    h
}

/// The heights of the column of the window at `index` of `kind` on the
/// second monitor of `h`, to a millionth.
fn heights(h: &Harness, kind: LayoutKind, index: usize) -> Vec<f64> {
    let shares = h.workspace(1, 0).split_ratios().heights(kind, index);
    let round = |share: f64| (share * 1e6).round() / 1e6;
    shares.unwrap_or_default().into_iter().map(round).collect()
}

#[test]
fn a_monitor_that_comes_back_gives_its_windows_back_the_heights_of_their_stacks() {
    let stack: &[(usize, &[f64])] = &[(1, &[0.4, 0.3, 0.2, 0.1])];
    let columns: &[(usize, &[f64])] = &[(1, &[0.7, 0.3]), (2, &[0.35, 0.65])];
    for (kind, resized) in [
        (LayoutKind::VerticalStack, stack),
        (LayoutKind::ThreeColumn, columns),
    ] {
        // Arrange
        let mut h = stacked_on_second(kind, resized);
        let before: Vec<Rect> = (2..=6).map(|id| h.rect_of(id)).collect();

        // Act -- it disconnects, its windows go to the first, and it is back.
        connect(&mut h, 1);
        connect(&mut h, 2);

        // Assert -- to the pixel.
        for &(index, shares) in resized {
            assert_eq!(heights(&h, kind, index), shares, "{kind:?} {index}");
        }
        let after: Vec<Rect> = (2..=6).map(|id| h.rect_of(id)).collect();
        assert_eq!(after, before, "{kind:?}");
    }
}

#[test]
fn a_window_that_comes_back_in_the_other_column_brings_no_height_of_its_column() {
    // Arrange -- windows 3 and 5 on the left of the master, window 2, and
    // windows 4 and 6 on the right, both columns resized.
    let resized: &[(usize, &[f64])] = &[(1, &[0.7, 0.3]), (2, &[0.35, 0.65])];
    let mut h = stacked_on_second(LayoutKind::ThreeColumn, resized);

    // Act -- window 3 closes while the monitor is away, which puts each of
    // the windows after it in the other column once they are back.
    connect(&mut h, 1);
    h.desktop.windows.remove(&wid(3));
    h.event(WindowEvent::Destroyed { window: wid(3) });
    connect(&mut h, 2);

    // Assert -- both columns are shared equally.
    assert_eq!(h.workspace(1, 0).windows().len(), 4);
    let both = [1, 2].map(|index| heights(&h, LayoutKind::ThreeColumn, index));
    assert_eq!(both, [Vec::<f64>::new(), Vec::new()]);
}

#[test]
fn a_window_back_in_the_other_column_evens_both_columns_as_a_close_at_home_does() {
    // Arrange -- windows 2 to 8 on the second monitor: windows 3, 5 and 7
    // on the left, at 349, 664 and 35 pixels, and 4, 6 and 8 on the right.
    let stacked = || {
        let windows: Vec<(u64, usize)> = [(1, 0)]
            .into_iter()
            .chain((2..=8).map(|id| (id, 1)))
            .collect();
        let mut h = Harness::new(2, &windows);
        let ws = h.tiler.monitors[1].active_ws_mut();
        ws.set_layout_kind(LayoutKind::ThreeColumn);
        ws.set_heights(1, &[349.0, 664.0, 35.0]);
        h.step(Input::Bar(BarReservation::default()));
        h
    };
    let (mut home, mut away) = (stacked(), stacked());
    let close = |h: &mut Harness| {
        h.desktop.windows.remove(&wid(6));
        h.event(WindowEvent::Destroyed { window: wid(6) });
    };

    // Act -- window 6 closes at home, and while the monitor is away, which
    // takes windows 7 and 8 to the other column.
    close(&mut home);
    connect(&mut away, 1);
    close(&mut away);
    connect(&mut away, 2);

    // Assert -- window 8 shares the left column equally, as at home.
    let rows = |h: &Harness| [3, 5, 8].map(|id| (h.rect_of(id).y, h.rect_of(id).height));
    assert_eq!(rows(&home), [(8, 349), (365, 349), (722, 350)]);
    assert_eq!(rows(&away), rows(&home));
}
