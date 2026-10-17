//! A stack nobody resized is laid out as stacks always were, in the
//! vertical stack and in the three columns, whatever the size of the area,
//! the gap, the master's ratio and the number of windows.

use crate::id::wid;
use crate::{LayoutKind, Rect, WindowId};

use super::stack_reference::{three_column, vertical_stack};
use super::{Layout, SplitRatios, ThreeColumnLayout, VerticalStackLayout};

#[test]
fn a_stack_nobody_resized_is_laid_out_as_it_always_was() {
    // Arrange -- sizes from one pixel up, odd ones and those of screens,
    // even and odd gaps up to the 200 the config allows, and none, and the
    // master's ratio at both ends of what the config allows.
    let sizes = [1, 2, 3, 7, 50, 199, 1080, 1441, 2561];
    let gaps = [0, 1, 2, 3, 7, 8, 9, 16, 25, 100, 101, 199, 200];
    let ratios = [0.1, 0.33, 0.5, 0.618, 0.9];
    let mut cases = 0;

    for count in 0..=9 {
        let windows: Vec<WindowId> = (1..=count as u64).map(wid).collect();
        let mut unmoved = SplitRatios::default();
        (0..count).for_each(|index| unmoved.insert(index, index));
        for (&width, &height) in sizes.iter().flat_map(|w| sizes.iter().map(move |h| (w, h))) {
            for (&gap, &ratio) in gaps.iter().flat_map(|g| ratios.iter().map(move |r| (g, r))) {
                // Act
                let area = Rect::new(13, -7, width, height);
                let stack = VerticalStackLayout { gap, ratio }.apply(&windows, &area);
                let columns = ThreeColumnLayout { gap, ratio }.apply(&windows, &area);
                let laid_out = |kind| unmoved.apply(kind, gap, ratio, &windows, &area);

                // Assert -- to the pixel, through the layouts and through
                // the splits of a workspace nobody resized.
                let case = format!("{count} windows in {width}x{height}, gap {gap}, {ratio}");
                let was = (
                    vertical_stack(gap, ratio, &windows, &area),
                    three_column(gap, ratio, &windows, &area),
                );
                assert_eq!(stack, was.0, "{case}");
                assert_eq!(laid_out(LayoutKind::VerticalStack), was.0, "{case}");
                assert_eq!(columns, was.1, "{case}");
                assert_eq!(laid_out(LayoutKind::ThreeColumn), was.1, "{case}");
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 10 * 9 * 9 * 13 * 5);
}

#[test]
fn each_window_of_a_resized_stack_gets_its_share_of_the_column() {
    // Arrange -- five windows, the stack at four tenths, three, two and
    // one, and the columns, windows 2 and 4 on the left and 3 and 5 on the
    // right, at three quarters and a quarter, and the other way; no gaps,
    // for round numbers.
    let windows: Vec<WindowId> = (1..=5).map(wid).collect();
    let mut ratios = SplitRatios::default();
    (0..5).for_each(|index| ratios.insert(index, index));
    ratios.set_heights(LayoutKind::VerticalStack, 1, &[0.4, 0.3, 0.2, 0.1]);
    ratios.set_heights(LayoutKind::ThreeColumn, 1, &[0.75, 0.25]);
    ratios.set_heights(LayoutKind::ThreeColumn, 2, &[0.25, 0.75]);
    let area = Rect::new(0, 0, 1920, 1080);

    // Act
    let stack = ratios.apply(LayoutKind::VerticalStack, 0, 0.5, &windows, &area);
    let columns = ratios.apply(LayoutKind::ThreeColumn, 0, 0.5, &windows, &area);

    // Assert
    let slot = |laid_out: &[(WindowId, Rect)], id| {
        let window = laid_out.iter().find(|w| w.0 == wid(id));
        window.map(|w| w.1)
    };
    let stacked = [2, 3, 4, 5].map(|id| slot(&stack, id));
    let in_columns = [2, 4, 3, 5].map(|id| slot(&columns, id));
    let rows = [(0, 432), (432, 324), (756, 216), (972, 108)];
    assert_eq!(stacked, rows.map(|(y, h)| Some(Rect::new(960, y, 960, h))));
    assert_eq!(
        in_columns,
        [
            Some(Rect::new(0, 0, 480, 810)),
            Some(Rect::new(0, 810, 480, 270)),
            Some(Rect::new(1440, 0, 480, 270)),
            Some(Rect::new(1440, 270, 480, 810)),
        ]
    );
}
