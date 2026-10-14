use super::*;

#[test]
fn window_events_carry_their_window() {
    // Arrange
    let window = WindowId::from_raw(0x10);
    let event = WindowEvent::Focused { window };

    // Act / Assert
    assert_eq!(event.window(), Some(window));
    assert_eq!(event.to_string(), "Focused 0x10");
}

#[test]
fn display_events_have_no_window() {
    // Arrange
    let event = WindowEvent::DisplayChanged;

    // Act / Assert
    assert_eq!(event.window(), None);
    assert_eq!(event.to_string(), "DisplayChanged");
}
