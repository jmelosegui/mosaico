use mosaico_core::{HidingSupport, WindowControl};

use super::Win32WindowControl;

#[test]
fn every_way_of_hiding_is_available_with_the_usual_reports() {
    // Arrange
    let control = Win32WindowControl::default();

    // Act
    let support = HidingSupport::new(|how| control.hiding_events(how));

    // Assert -- the default is what the tiler assumed of every platform
    // before asking, so on Windows nothing changes.
    assert_eq!(support, HidingSupport::default());
}
