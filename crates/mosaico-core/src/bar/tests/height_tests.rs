//! The height the manager reports to the tiler when the bars change.

use super::{MOCHA, bar, left, right, start};
use crate::bar::fakes::{FakeRenderer, FakeSystem};
use crate::config::BarConfig;

#[test]
fn unplugging_the_only_monitor_with_a_bar_leaves_no_height() {
    // Arrange -- #91: the bar is only on the second monitor.
    let mut manager = start(
        bar("monitors = [1]"),
        &[left(), right()],
        &FakeRenderer::default(),
        &FakeSystem::default(),
    );

    // Act
    manager.rebuild_for_monitors(vec![left()], MOCHA);

    // Assert
    assert!(manager.bar_monitor_indices().is_empty());
    assert_eq!(manager.bar_height(), 0);
}

#[test]
fn reloading_a_bar_no_monitor_gets_returns_no_height() {
    // Arrange -- #91
    let mut manager = start(
        BarConfig::default(),
        &[left()],
        &FakeRenderer::default(),
        &FakeSystem::default(),
    );

    // Act
    let height = manager.reload(bar("monitors = [1]"));

    // Assert
    assert_eq!(height, 0);
}
