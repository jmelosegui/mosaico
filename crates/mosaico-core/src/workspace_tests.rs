use super::*;
use crate::Rect;
use crate::id::{wid, wids};
use crate::layout::{BspLayout, Layout};

#[test]
fn add_and_remove_windows() {
    // Arrange
    let mut ws = Workspace::new();

    // Act
    assert!(ws.add(wid(1)));
    assert!(ws.add(wid(2)));
    assert!(!ws.add(wid(1))); // duplicate

    // Assert
    assert_eq!(ws.len(), 2);
    assert!(ws.contains(wid(1)));
    assert!(ws.remove(wid(1)));
    assert_eq!(ws.len(), 1);
    assert!(!ws.contains(wid(1)));
}

#[test]
fn insert_at_position() {
    let mut ws = Workspace::new();
    ws.add(wid(1));
    ws.add(wid(2));

    // Insert at front
    assert!(ws.insert(0, wid(3)));
    assert_eq!(ws.windows(), wids(&[3, 1, 2]));

    // Duplicate rejected
    assert!(!ws.insert(0, wid(1)));

    // Index clamped to len
    assert!(ws.insert(100, wid(4)));
    assert_eq!(ws.windows(), wids(&[3, 1, 2, 4]));
}

#[test]
fn compute_layout_delegates_to_layout() {
    // Arrange
    let mut ws = Workspace::new();
    ws.add(wid(100));
    ws.add(wid(200));
    let layout = BspLayout { gap: 0, ratio: 0.5 };
    let area = Rect::new(0, 0, 1920, 1080);

    // Act
    let positions = ws.compute_layout(0, 0.5, &area);

    // Assert
    assert_eq!(positions, layout.apply(ws.windows(), &area));
    assert_eq!(positions[0].0, wid(100));
    assert_eq!(positions[1].0, wid(200));
}
