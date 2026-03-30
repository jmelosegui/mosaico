use super::*;

// -- BSP tests --

#[test]
fn single_window_fills_work_area() {
    let layout = BspLayout { gap: 0, ratio: 0.5 };
    let area = Rect::new(0, 0, 1920, 1080);
    let result = layout.apply(&[1], &area);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0], (1, Rect::new(0, 0, 1920, 1080)));
}

#[test]
fn two_windows_split_horizontally() {
    let layout = BspLayout { gap: 0, ratio: 0.5 };
    let area = Rect::new(0, 0, 1920, 1080);
    let result = layout.apply(&[1, 2], &area);

    assert_eq!(result.len(), 2);
    assert_eq!(result[0], (1, Rect::new(0, 0, 960, 1080)));
    assert_eq!(result[1], (2, Rect::new(960, 0, 960, 1080)));
}

#[test]
fn three_windows_bsp_split() {
    let layout = BspLayout { gap: 0, ratio: 0.5 };
    let area = Rect::new(0, 0, 1920, 1080);
    let result = layout.apply(&[1, 2, 3], &area);

    assert_eq!(result.len(), 3);
    assert_eq!(result[0], (1, Rect::new(0, 0, 960, 1080)));
    assert_eq!(result[1], (2, Rect::new(960, 0, 960, 540)));
    assert_eq!(result[2], (3, Rect::new(960, 540, 960, 540)));
}

#[test]
fn empty_handles_returns_empty() {
    let layout = BspLayout::default();
    let area = Rect::new(0, 0, 1920, 1080);
    assert!(layout.apply(&[], &area).is_empty());
}

#[test]
fn large_gap_never_produces_negative_dimensions() {
    let layout = BspLayout {
        gap: 500,
        ratio: 0.5,
    };
    let area = Rect::new(0, 0, 200, 200);
    let result = layout.apply(&[1, 2], &area);

    for (_hwnd, rect) in &result {
        assert!(rect.width > 0, "width was {}", rect.width);
        assert!(rect.height > 0, "height was {}", rect.height);
    }
}

// -- LayoutKind tests --

#[test]
fn layout_kind_cycles() {
    assert_eq!(LayoutKind::Bsp.next(), LayoutKind::VerticalStack);
    assert_eq!(LayoutKind::VerticalStack.next(), LayoutKind::ThreeColumn);
    assert_eq!(LayoutKind::ThreeColumn.next(), LayoutKind::Bsp);
}

#[test]
fn layout_kind_names() {
    assert_eq!(LayoutKind::Bsp.name(), "BSP");
    assert_eq!(LayoutKind::VerticalStack.name(), "VStack");
    assert_eq!(LayoutKind::ThreeColumn.name(), "3Col");
}

// -- VerticalStack tests --

#[test]
fn vstack_single_window_fills_work_area() {
    let layout = VerticalStackLayout { gap: 0, ratio: 0.5 };
    let area = Rect::new(0, 0, 1920, 1080);
    let result = layout.apply(&[1], &area);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0], (1, Rect::new(0, 0, 1920, 1080)));
}

#[test]
fn vstack_two_windows_master_and_stack() {
    let layout = VerticalStackLayout { gap: 0, ratio: 0.5 };
    let area = Rect::new(0, 0, 1920, 1080);
    let result = layout.apply(&[1, 2], &area);

    assert_eq!(result.len(), 2);
    assert_eq!(result[0], (1, Rect::new(0, 0, 960, 1080)));
    assert_eq!(result[1], (2, Rect::new(960, 0, 960, 1080)));
}

#[test]
fn vstack_three_windows_stack_splits_equally() {
    let layout = VerticalStackLayout { gap: 0, ratio: 0.5 };
    let area = Rect::new(0, 0, 1920, 1080);
    let result = layout.apply(&[1, 2, 3], &area);

    assert_eq!(result.len(), 3);
    assert_eq!(result[0], (1, Rect::new(0, 0, 960, 1080)));
    assert_eq!(result[1], (2, Rect::new(960, 0, 960, 540)));
    assert_eq!(result[2], (3, Rect::new(960, 540, 960, 540)));
}

#[test]
fn vstack_five_windows() {
    let layout = VerticalStackLayout { gap: 0, ratio: 0.5 };
    let area = Rect::new(0, 0, 1920, 1080);
    let result = layout.apply(&[1, 2, 3, 4, 5], &area);

    assert_eq!(result.len(), 5);
    assert_eq!(result[0].1.width, 960);
    assert_eq!(result[0].1.height, 1080);
    for r in &result[1..] {
        assert_eq!(r.1.width, 960);
    }
    let stack_top = result[1].1.y;
    let stack_bottom = result[4].1.y + result[4].1.height;
    assert_eq!(stack_bottom - stack_top, 1080);
}

#[test]
fn vstack_empty_returns_empty() {
    let layout = VerticalStackLayout::default();
    let area = Rect::new(0, 0, 1920, 1080);
    assert!(layout.apply(&[], &area).is_empty());
}

// -- ThreeColumn tests --

#[test]
fn three_col_single_window_fills_area() {
    let layout = ThreeColumnLayout { gap: 0, ratio: 0.5 };
    let area = Rect::new(0, 0, 1920, 1080);
    let result = layout.apply(&[1], &area);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0], (1, Rect::new(0, 0, 1920, 1080)));
}

#[test]
fn three_col_two_windows_master_and_right() {
    let layout = ThreeColumnLayout { gap: 0, ratio: 0.5 };
    let area = Rect::new(0, 0, 1920, 1080);
    let result = layout.apply(&[1, 2], &area);

    assert_eq!(result.len(), 2);
    assert_eq!(result[0], (1, Rect::new(0, 0, 960, 1080)));
    assert_eq!(result[1], (2, Rect::new(960, 0, 960, 1080)));
}

#[test]
fn three_col_three_windows_center_master() {
    let layout = ThreeColumnLayout { gap: 0, ratio: 0.5 };
    let area = Rect::new(0, 0, 1920, 1080);
    let result = layout.apply(&[1, 2, 3], &area);

    assert_eq!(result.len(), 3);
    assert_eq!(result[0].0, 1);
    assert_eq!(result[0].1.width, 960);
    assert_eq!(result[1].0, 2);
    assert!(result[1].1.x < result[0].1.x);
    assert_eq!(result[2].0, 3);
    assert!(result[2].1.x > result[0].1.x);
}

#[test]
fn three_col_five_windows_alternates_sides() {
    let layout = ThreeColumnLayout { gap: 0, ratio: 0.5 };
    let area = Rect::new(0, 0, 1920, 1080);
    let result = layout.apply(&[1, 2, 3, 4, 5], &area);

    assert_eq!(result.len(), 5);
    assert_eq!(result[0].0, 1);
    assert_eq!(result[1].0, 2);
    assert_eq!(result[2].0, 4);
    assert_eq!(result[3].0, 3);
    assert_eq!(result[4].0, 5);
    let left_bottom = result[2].1.y + result[2].1.height;
    assert_eq!(left_bottom, 1080);
    let right_bottom = result[4].1.y + result[4].1.height;
    assert_eq!(right_bottom, 1080);
}

#[test]
fn three_col_empty_returns_empty() {
    let layout = ThreeColumnLayout::default();
    let area = Rect::new(0, 0, 1920, 1080);
    assert!(layout.apply(&[], &area).is_empty());
}
