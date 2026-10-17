use super::*;

use super::tests::{bsp, chain};

#[test]
fn a_window_inserted_at_k_gets_the_configured_ratio_at_split_k() {
    // Arrange -- three windows, both splits moved.
    let mut ratios = chain(3);
    ratios.set(LayoutKind::Bsp, 0, 0.7);
    ratios.set(LayoutKind::Bsp, 1, 0.3);

    // Act -- a fourth window goes in second.
    ratios.insert(1, 3);

    // Assert
    assert_eq!(bsp(&ratios, 4), [Some(0.7), None, Some(0.3)]);
}

#[test]
fn the_first_window_gets_no_split_and_one_added_at_the_end_the_last() {
    // Arrange
    let mut ratios = SplitRatios::default();

    // Act -- the first window, then a second and a third at the end, with
    // the first split moved in between.
    ratios.insert(0, 0);
    let alone = ratios.clone();
    ratios.insert(1, 1);
    ratios.set(LayoutKind::Bsp, 0, 0.75);
    ratios.insert(2, 2);

    // Assert
    assert_eq!(alone, SplitRatios::default());
    assert_eq!(bsp(&ratios, 3), [Some(0.75), None]);
}

#[test]
fn a_window_that_leaves_takes_its_split_with_it() {
    // Arrange -- four windows, every split moved.
    let moved = || {
        let mut ratios = chain(4);
        for (k, ratio) in [0.7, 0.3, 0.6].into_iter().enumerate() {
            ratios.set(LayoutKind::Bsp, k, ratio);
        }
        ratios
    };

    // Act -- the first, the second, and the last leave.
    let left = [0, 1, 3].map(|index| {
        let mut ratios = moved();
        ratios.remove(index, 4);
        bsp(&ratios, 3)
    });

    // Assert -- the last window's split is the last one.
    assert_eq!(
        left,
        [
            [Some(0.3), Some(0.6)],
            [Some(0.7), Some(0.6)],
            [Some(0.7), Some(0.3)],
        ]
    );
}

#[test]
fn the_last_window_to_leave_leaves_no_split() {
    // Arrange
    let mut ratios = chain(2);
    ratios.set(LayoutKind::Bsp, 0, 0.7);

    // Act
    ratios.remove(1, 2);
    ratios.remove(0, 1);

    // Assert
    assert_eq!(ratios, SplitRatios::default());
}

#[test]
fn the_masters_keep_their_ratios_as_windows_come_and_go() {
    // Arrange
    let mut ratios = chain(2);
    ratios.set(LayoutKind::VerticalStack, 0, 0.7);
    ratios.set(LayoutKind::ThreeColumn, 0, 0.3);

    // Act -- a window goes in first, then the first two leave.
    ratios.insert(0, 2);
    ratios.remove(0, 3);
    ratios.remove(0, 2);

    // Assert
    assert_eq!(ratios.ratio(LayoutKind::VerticalStack, 0), Some(0.7));
    assert_eq!(ratios.ratio(LayoutKind::ThreeColumn, 0), Some(0.3));
}

#[test]
fn forgetting_the_ratios_keeps_a_split_for_each_window_but_the_last() {
    // Arrange -- three windows, both splits and the masters moved.
    let mut ratios = chain(3);
    ratios.set(LayoutKind::Bsp, 0, 0.7);
    ratios.set(LayoutKind::Bsp, 1, 0.3);
    ratios.set(LayoutKind::VerticalStack, 0, 0.6);
    ratios.set(LayoutKind::ThreeColumn, 0, 0.4);

    // Act -- as a retile does, then the second split moved again.
    ratios.forget();
    let forgotten = ratios.clone();
    ratios.set(LayoutKind::Bsp, 1, 0.8);

    // Assert -- as three windows that nobody moved, and the chain still
    // has the split the next resize moves.
    assert_eq!(forgotten, chain(3));
    assert_eq!(bsp(&ratios, 3), [None, Some(0.8)]);
}
