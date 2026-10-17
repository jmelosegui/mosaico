use super::*;
use crate::LayoutKind;
use crate::id::{wid, wids};

/// A BSP workspace with `windows`, whose splits are moved to `ratios`, in
/// order.
fn moved(windows: &[u64], ratios: &[f64]) -> Workspace {
    let mut ws = Workspace::new();
    for &window in windows {
        ws.add(wid(window));
    }
    for (split, &ratio) in ratios.iter().enumerate() {
        ws.set_split_ratio(split, ratio);
    }
    ws
}

/// The ratio of each split of the BSP chain of `ws`.
fn chain(ws: &Workspace) -> Vec<Option<f64>> {
    (0..ws.len().saturating_sub(1))
        .map(|split| ws.split_ratios().ratio(LayoutKind::Bsp, split))
        .collect()
}

#[test]
fn a_swap_keeps_every_split_where_it_is() {
    // Arrange
    let mut ws = moved(&[1, 2, 3], &[0.7, 0.3]);

    // Act
    ws.swap(0, 2);

    // Assert -- the windows trade places in slots of the same sizes.
    assert_eq!(ws.windows(), wids(&[3, 2, 1]));
    assert_eq!(chain(&ws), [Some(0.7), Some(0.3)]);
}

#[test]
fn a_window_added_or_inserted_gets_a_split_at_the_configured_ratio() {
    // Arrange
    let mut ws = moved(&[1, 2, 3], &[0.7, 0.3]);

    // Act -- one goes in second, and one at the end.
    ws.insert(1, wid(4));
    ws.add(wid(5));

    // Assert
    assert_eq!(ws.windows(), wids(&[1, 4, 2, 3, 5]));
    assert_eq!(chain(&ws), [Some(0.7), None, Some(0.3), None]);
}

#[test]
fn a_window_removed_takes_its_split_with_it() {
    // Arrange
    let mut ws = moved(&[1, 2, 3, 4], &[0.7, 0.3, 0.6]);

    // Act
    ws.remove(wid(2));

    // Assert
    assert_eq!(chain(&ws), [Some(0.7), Some(0.6)]);
}

#[test]
fn a_window_the_workspace_has_already_or_does_not_have_changes_no_split() {
    // Arrange
    let mut ws = moved(&[1, 2, 3], &[0.7, 0.3]);

    // Act
    let added = ws.add(wid(2));
    let inserted = ws.insert(0, wid(3));
    let removed = ws.remove(wid(9));

    // Assert
    assert!(!added && !inserted && !removed);
    assert_eq!(chain(&ws), [Some(0.7), Some(0.3)]);
}

#[test]
fn each_layout_of_the_workspace_keeps_its_own_splits() {
    // Arrange -- the first split moved in BSP.
    let mut ws = moved(&[1, 2, 3], &[0.7]);

    // Act -- the stack's master moved, and back to BSP.
    ws.set_layout_kind(LayoutKind::VerticalStack);
    ws.set_split_ratio(0, 0.25);
    ws.set_layout_kind(LayoutKind::Bsp);

    // Assert
    let ratios = ws.split_ratios();
    assert_eq!(chain(&ws), [Some(0.7), None]);
    assert_eq!(ratios.ratio(LayoutKind::VerticalStack, 0), Some(0.25));
    assert_eq!(ratios.ratio(LayoutKind::ThreeColumn, 0), None);
}

#[test]
fn monocle_leaves_the_splits_alone() {
    // Arrange
    let mut ws = moved(&[1, 2, 3], &[0.7, 0.3]);

    // Act
    ws.set_monocle(true);
    ws.set_monocle_window(Some(wid(2)));
    ws.set_monocle(false);

    // Assert
    assert_eq!(chain(&ws), [Some(0.7), Some(0.3)]);
}

#[test]
fn compute_layout_puts_each_split_where_the_user_moved_it() {
    // Arrange -- the first split at three quarters of the width.
    let ws = moved(&[1, 2], &[0.75]);
    let area = Rect::new(0, 0, 1920, 1080);

    // Act
    let positions = ws.compute_layout(0, 0.5, &area);

    // Assert
    let widths: Vec<i32> = positions.iter().map(|&(_, rect)| rect.width).collect();
    assert_eq!(widths, [1440, 480]);
}

#[test]
fn heights_for_another_number_of_windows_than_the_column_has_are_left_out() {
    // Arrange -- the master and three windows down the stack, and the
    // master, three windows on the left and two on the right in three
    // columns.
    let (mut stack, mut columns) = (moved(&[1, 2, 3, 4], &[]), moved(&[1, 2, 3, 4, 5, 6], &[]));
    stack.set_layout_kind(LayoutKind::VerticalStack);
    columns.set_layout_kind(LayoutKind::ThreeColumn);

    // Act -- two shares for the three of the stack, then three, and two
    // for the right column, then three.
    stack.set_heights(1, &[0.5, 0.5]);
    let left_out = stack.split_ratios().heights(LayoutKind::VerticalStack, 1);
    stack.set_heights(1, &[0.5, 0.25, 0.25]);
    columns.set_heights(2, &[0.25, 0.75]);
    columns.set_heights(2, &[0.25, 0.25, 0.5]);

    // Assert
    assert_eq!(left_out, None);
    let shares = stack.split_ratios().heights(LayoutKind::VerticalStack, 1);
    assert_eq!(shares, Some(vec![0.5, 0.25, 0.25]));
    let right = columns.split_ratios().heights(LayoutKind::ThreeColumn, 2);
    assert_eq!(right, Some(vec![0.25, 0.75]));
}
