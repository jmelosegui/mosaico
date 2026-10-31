use super::*;
use crate::id::{wid, wids};

#[test]
fn a_floating_window_is_held_but_not_tiled() {
    // Arrange
    let mut ws = Workspace::new();
    ws.add(wid(1));

    // Act
    let added = ws.add_floating(wid(2));

    // Assert
    assert!(added);
    assert_eq!(ws.windows(), wids(&[1]));
    assert_eq!(ws.floating(), wids(&[2]));
    assert!(ws.holds(wid(2)) && !ws.contains(wid(2)));
}

#[test]
fn a_window_is_never_both_tiled_and_floating() {
    // Arrange
    let mut ws = Workspace::new();
    ws.add(wid(1));
    ws.add_floating(wid(2));

    // Act / Assert
    assert!(!ws.add_floating(wid(1)));
    assert!(!ws.add(wid(2)));
    assert_eq!(ws.windows(), wids(&[1]));
    assert_eq!(ws.floating(), wids(&[2]));
}

#[test]
fn raising_a_floating_window_puts_it_in_front() {
    // Arrange
    let mut ws = Workspace::new();
    for id in [1, 2, 3] {
        ws.add_floating(wid(id));
    }

    // Act
    ws.raise_floating(wid(1));

    // Assert -- back to front.
    assert_eq!(ws.floating(), wids(&[2, 3, 1]));
}

#[test]
fn removing_the_remembered_floating_window_forgets_it() {
    // Arrange
    let mut ws = Workspace::new();
    ws.add_floating(wid(1));
    ws.set_last_focused(Some(wid(1)));

    // Act
    let removed = ws.remove_floating(wid(1));

    // Assert
    assert!(removed);
    assert!(ws.floating().is_empty());
    assert_eq!(ws.last_focused(), None);
}

#[test]
fn monocle_shows_only_the_tiles() {
    // Arrange
    let mut ws = Workspace::new();
    ws.add(wid(1));
    ws.add_floating(wid(2));

    // Act
    let tiled = ws.shown();
    ws.set_monocle(true);
    let monocle = ws.shown();

    // Assert
    assert_eq!(tiled, wids(&[1, 2]));
    assert_eq!(monocle, wids(&[1]));
}
