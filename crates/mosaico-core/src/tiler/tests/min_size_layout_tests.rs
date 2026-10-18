//! Layouts give a window its minimum size where the area has room, read
//! once per window, and stay as they were where no minimum binds.

use crate::id::wid;
use crate::tiler::tests::{FakeWindow, Harness};
use crate::{Action, LayoutKind, MinSize, Rect, WindowEvent};

fn laid_out(kind: LayoutKind, count: u64) -> Harness {
    let windows: Vec<(u64, usize)> = (1..=count).map(|id| (id, 0)).collect();
    let mut h = Harness::new(1, &windows);
    while h.tiler.monitors()[0].active().layout_kind() != kind {
        h.action(Action::CycleLayout);
    }
    h
}

/// Opens window `id` on the first monitor with the given minimum.
fn open_with(h: &mut Harness, id: u64, min: MinSize) {
    let mut window = FakeWindow::app(0);
    window.min_size = Some(min);
    h.desktop.windows.insert(wid(id), window);
    h.desktop.raise(wid(id));
    h.event(WindowEvent::Created { window: wid(id) });
}

fn rects(h: &Harness, count: u64) -> Vec<Rect> {
    (1..=count).map(|id| h.rect_of(id)).collect()
}

#[test]
fn a_new_window_gets_its_minimum_width_in_bsp() {
    // Arrange
    let mut h = laid_out(LayoutKind::Bsp, 1);

    // Act
    open_with(&mut h, 2, MinSize::new(1200, 0));

    // Assert
    assert_eq!(h.rect_of(2), Rect::new(712, 8, 1200, 1064));
    assert_eq!(h.rect_of(1), Rect::new(8, 8, 696, 1064));
}

#[test]
fn a_new_window_gets_its_minimum_height_in_bsp() {
    // Arrange
    let mut h = laid_out(LayoutKind::Bsp, 2);

    // Act -- window 3 takes the bottom of the right half.
    open_with(&mut h, 3, MinSize::new(0, 800));

    // Assert
    assert_eq!(h.rect_of(3).height, 800);
    assert_eq!(h.rect_of(2).y + h.rect_of(2).height + 8, h.rect_of(3).y);
}

#[test]
fn the_master_gets_its_minimum_width_in_a_vertical_stack() {
    // Arrange
    let mut h = laid_out(LayoutKind::VerticalStack, 2);

    // Act -- the first window opened has a minimum wider than half.
    h.window(1).min_size = Some(MinSize::new(1300, 0));
    h.action(Action::Retile);

    // Assert
    assert_eq!(h.rect_of(1).width, 1300);
    assert_eq!(h.rect_of(2), Rect::new(1316, 8, 596, 1064));
}

#[test]
fn a_stack_window_gets_its_minimum_height_from_the_others() {
    // Arrange
    let mut h = laid_out(LayoutKind::VerticalStack, 3);

    // Act
    h.window(2).min_size = Some(MinSize::new(0, 800));
    h.action(Action::Retile);

    // Assert
    let (top, bottom) = (h.rect_of(2), h.rect_of(3));
    assert_eq!(top.height, 800);
    assert_eq!(bottom.y + bottom.height, 1072);
    assert_eq!(top.y + top.height + 8, bottom.y);
}

#[test]
fn the_center_gets_its_minimum_width_in_three_columns() {
    // Arrange
    let mut h = laid_out(LayoutKind::ThreeColumn, 3);

    // Act
    h.window(1).min_size = Some(MinSize::new(1400, 0));
    h.action(Action::Retile);

    // Assert -- the sides share what is left.
    assert_eq!(h.rect_of(1).width, 1400);
    assert!(h.rect_of(2).width > 0 && h.rect_of(3).width > 0);
}

#[test]
fn a_minimum_that_binds_nowhere_changes_nothing() {
    for kind in [
        LayoutKind::Bsp,
        LayoutKind::VerticalStack,
        LayoutKind::ThreeColumn,
    ] {
        // Arrange
        let mut plain = laid_out(kind, 4);
        let mut small = laid_out(kind, 4);

        // Act
        small.window(2).min_size = Some(MinSize::new(100, 100));
        small.action(Action::Retile);
        plain.action(Action::Retile);

        // Assert
        assert_eq!(rects(&small, 4), rects(&plain, 4), "{kind:?}");
    }
}

#[test]
fn minimums_that_cannot_all_fit_leave_the_layout_as_it_was() {
    // Arrange
    let mut plain = laid_out(LayoutKind::Bsp, 2);
    let mut crowded = laid_out(LayoutKind::Bsp, 2);

    // Act -- two windows each wanting more than the monitor is wide.
    for id in [1, 2] {
        crowded.window(id).min_size = Some(MinSize::new(1500, 0));
    }
    crowded.action(Action::Retile);
    plain.action(Action::Retile);

    // Assert
    assert_eq!(rects(&crowded, 2), rects(&plain, 2));
}

#[path = "min_size_read_tests.rs"]
mod read_tests;
