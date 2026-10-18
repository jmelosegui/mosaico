//! The minimum size of a window is read once, not on every layout.

use crate::Action;
use crate::tiler::tests::Harness;

#[test]
fn the_minimum_is_read_once_per_window_until_a_retile() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    let reads = h.desktop.min_size_reads.get();

    // Act -- a new window and more layouts of the same ones.
    h.open(3, 0);
    let after_open = h.desktop.min_size_reads.get();
    h.action(Action::CycleLayout);
    h.action(Action::CycleLayout);

    // Assert -- only the new window was read; a retile reads all again.
    assert_eq!(after_open - reads, 1);
    assert_eq!(h.desktop.min_size_reads.get(), after_open);
    h.action(Action::Retile);
    assert_eq!(h.desktop.min_size_reads.get() - after_open, 3);
}
