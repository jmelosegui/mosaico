use super::*;

#[test]
fn a_widget_moves_within_its_side_and_to_the_other() {
    // Arrange
    let mut widgets = BarWidgets::default();
    let first_left = widgets.left[0].clone();
    let second_left = widgets.left[1].clone();

    // Act
    widgets.move_to(BarZone::Left, 0, BarZone::Left, 1);
    widgets.move_to(BarZone::Left, 0, BarZone::Right, 0);

    // Assert
    assert_eq!(widgets.left[0], first_left);
    assert_eq!(widgets.right[0], second_left);
}

#[test]
fn a_move_from_a_place_without_a_widget_changes_nothing() {
    let mut widgets = BarWidgets::default();
    let before = widgets.clone();

    widgets.move_to(BarZone::Left, 99, BarZone::Right, 0);

    assert_eq!(widgets, before);
}
