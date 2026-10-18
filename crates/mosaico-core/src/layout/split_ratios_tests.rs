use super::*;
use crate::id::wids;

/// A 1920x1080 area, laid out without gaps so the slots are easy to read.
const AREA: Rect = Rect {
    x: 0,
    y: 0,
    width: 1920,
    height: 1080,
};

/// The ratios of a BSP chain of `count` windows, none of them moved.
pub(super) fn chain(count: usize) -> SplitRatios {
    let mut ratios = SplitRatios::default();
    for index in 0..count {
        ratios.insert(index, index);
    }
    ratios
}

/// The ratio of each split of the BSP chain of `ratios`, for `count`
/// windows.
pub(super) fn bsp(ratios: &SplitRatios, count: usize) -> Vec<Option<f64>> {
    (0..count.saturating_sub(1))
        .map(|k| ratios.ratio(LayoutKind::Bsp, k))
        .collect()
}

#[test]
fn splits_the_user_never_moved_follow_the_configured_ratio() {
    // Arrange
    let ratios = chain(3);
    let windows = wids(&[1, 2, 3]);

    for kind in [
        LayoutKind::Bsp,
        LayoutKind::VerticalStack,
        LayoutKind::ThreeColumn,
    ] {
        // Act
        let laid_out = ratios.apply(kind, 8, 0.6, &windows, &AREA);

        // Assert -- as the layout lays them out on its own.
        let expected = match kind {
            LayoutKind::Bsp => BspLayout { gap: 8, ratio: 0.6 }.apply(&windows, &AREA),
            LayoutKind::VerticalStack => {
                VerticalStackLayout { gap: 8, ratio: 0.6 }.apply(&windows, &AREA)
            }
            LayoutKind::ThreeColumn => {
                ThreeColumnLayout { gap: 8, ratio: 0.6 }.apply(&windows, &AREA)
            }
        };
        assert_eq!(laid_out, expected, "{kind:?}");
    }
}

#[test]
fn each_split_of_the_bsp_chain_divides_its_area_at_its_own_ratio() {
    // Arrange -- the first split at three quarters, the second at a
    // quarter.
    let mut ratios = chain(3);
    ratios.set(LayoutKind::Bsp, 0, 0.75);
    ratios.set(LayoutKind::Bsp, 1, 0.25);

    // Act
    let laid_out = ratios.apply(LayoutKind::Bsp, 0, 0.5, &wids(&[1, 2, 3]), &AREA);

    // Assert
    let rects: Vec<Rect> = laid_out.iter().map(|&(_, rect)| rect).collect();
    assert_eq!(
        rects,
        [
            Rect::new(0, 0, 1440, 1080),
            Rect::new(1440, 0, 480, 270),
            Rect::new(1440, 270, 480, 810),
        ]
    );
}

#[test]
fn the_master_of_the_stack_and_of_the_three_columns_has_its_own_ratio() {
    // Arrange
    let mut ratios = SplitRatios::default();
    ratios.set(LayoutKind::VerticalStack, 0, 0.75);
    ratios.set(LayoutKind::ThreeColumn, 0, 0.25);
    let windows = wids(&[1, 2, 3]);

    // Act
    let stack = ratios.apply(LayoutKind::VerticalStack, 0, 0.5, &windows, &AREA);
    let columns = ratios.apply(LayoutKind::ThreeColumn, 0, 0.5, &windows, &AREA);

    // Assert -- the three columns' master stays centered.
    assert_eq!(stack[0].1, Rect::new(0, 0, 1440, 1080));
    assert_eq!(columns[0].1, Rect::new(720, 0, 480, 1080));
}

#[test]
fn each_layout_keeps_its_own_ratios() {
    // Arrange
    let mut ratios = chain(2);

    // Act
    ratios.set(LayoutKind::Bsp, 0, 0.7);

    // Assert
    assert_eq!(ratios.ratio(LayoutKind::Bsp, 0), Some(0.7));
    assert_eq!(ratios.ratio(LayoutKind::VerticalStack, 0), None);
    assert_eq!(ratios.ratio(LayoutKind::ThreeColumn, 0), None);
}

#[test]
fn a_ratio_is_kept_within_a_tenth_and_nine_tenths() {
    // Arrange
    let mut ratios = chain(3);

    // Act
    ratios.set(LayoutKind::Bsp, 0, 0.95);
    ratios.set(LayoutKind::Bsp, 1, -0.2);
    ratios.set(LayoutKind::VerticalStack, 0, 0.05);

    // Assert
    assert_eq!(
        bsp(&ratios, 3),
        [Some(MAX_SPLIT_RATIO), Some(MIN_SPLIT_RATIO)]
    );
    let master = ratios.ratio(LayoutKind::VerticalStack, 0);
    assert_eq!(master, Some(MIN_SPLIT_RATIO));
}

#[test]
fn a_split_the_layout_does_not_have_is_left_alone() {
    // Arrange -- three windows have two splits in BSP, and the stack and
    // the columns only split 0.
    let mut ratios = chain(3);

    // Act
    ratios.set(LayoutKind::Bsp, 2, 0.7);
    ratios.set(LayoutKind::VerticalStack, 1, 0.7);
    ratios.set(LayoutKind::ThreeColumn, 1, 0.7);

    // Assert
    assert_eq!(ratios, chain(3));
}
