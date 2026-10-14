//! Laying every window out again draws every border from scratch, since
//! what a platform kept of a border may be lost by then. Other steps keep
//! what still looks right (#125).

use crate::Direction;
use crate::config::Config;

use super::fake_platform::FakePlatform;
use super::manager_support::start;
use super::*;

/// Three windows side by side on the first of `monitors` monitors, the
/// left one focused, with focused and unfocused borders in different
/// colors. Every border has been drawn once.
fn three_windows(monitors: usize) -> (FakePlatform, TilingManager<FakePlatform>) {
    let mut config = Config::default();
    config.borders.colors.focused = "#111111".into();
    config.borders.colors.unfocused = "#222222".into();
    let platform = FakePlatform::new(monitors, &[(1, 0), (2, 0), (3, 0)]);
    let manager = start(&platform, None, &config);
    platform.take_renders();
    (platform, manager)
}

/// Where each of the three windows is.
fn slots(platform: &FakePlatform) -> Vec<Rect> {
    [1, 2, 3].map(|w| platform.window(w).rect).to_vec()
}

/// The windows whose border was drawn from scratch since the last look.
fn drawn_from_scratch(platform: &FakePlatform) -> Vec<u64> {
    let mut windows = platform.take_renders();
    windows.sort_unstable();
    windows
}

#[test]
fn a_retile_draws_every_border_from_scratch() {
    // Arrange
    let (platform, mut manager) = three_windows(1);
    let before = slots(&platform);

    // Act
    manager.handle_action(&Action::Retile);

    // Assert -- although no window moved.
    assert_eq!(slots(&platform), before);
    assert_eq!(drawn_from_scratch(&platform), [1, 2, 3]);
}

#[test]
fn reading_the_work_areas_again_draws_every_border_from_scratch() {
    // Arrange -- as a retile from the daemon does.
    let (platform, mut manager) = three_windows(1);
    let before = slots(&platform);

    // Act
    let read = manager.reset_and_adjust_work_areas(0, &[]);

    // Assert
    assert!(read.is_ok());
    assert_eq!(slots(&platform), before);
    assert_eq!(drawn_from_scratch(&platform), [1, 2, 3]);
}

#[test]
fn a_display_change_draws_every_border_from_scratch() {
    // Arrange
    let (platform, mut manager) = three_windows(2);
    let before = slots(&platform);

    // Act -- the second monitor, which has no windows, goes away.
    manager.handle_display_change(displays(1), 0, &[]);

    // Assert
    assert_eq!(slots(&platform), before);
    assert_eq!(drawn_from_scratch(&platform), [1, 2, 3]);
}

#[test]
fn a_focus_change_draws_from_scratch_only_the_borders_that_change() {
    // Arrange
    let (platform, mut manager) = three_windows(1);

    // Act -- focus moves from the left window to the middle one.
    manager.handle_action(&Action::Focus(Direction::Right));

    // Assert -- the right one keeps its unfocused border as it was drawn.
    assert_eq!(drawn_from_scratch(&platform), [1, 2]);
}
