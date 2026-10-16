//! Each widget's text, from a [`BarState`] and a fake machine.

use super::*;
use crate::LayoutKind;
use crate::bar::fakes::FakeSystem;
use crate::bar::{CpuTimes, CpuTracker, LocalTime};
use crate::config::bar::BarConfig;

/// A bar whose only widget is `widget`, an inline table of `bar.toml`.
pub(super) fn bar_of(widget: &str) -> BarConfig {
    toml::from_str(&format!("left = []\nright = [{widget}]")).expect("a valid bar.toml")
}

/// What `widget` reads on a monitor in `state`, on `system`.
pub(super) fn label_of(widget: &str, state: &BarState, system: &FakeSystem) -> String {
    let config = bar_of(widget);
    let readings = SystemReadings::read(system, &config, None);
    label(&config.right[0], state, &readings)
}

/// Whether `widget` shows on a monitor in `state`, on `system`.
pub(super) fn shows(widget: &str, state: &BarState, system: &FakeSystem) -> bool {
    let config = bar_of(widget);
    let readings = SystemReadings::read(system, &config, None);
    is_shown(&config.right[0], state, &readings)
}

/// A machine whose local time is `time`.
fn at(time: LocalTime) -> FakeSystem {
    let system = FakeSystem::default();
    system.set(|m| m.time = time);
    system
}

/// What a CPU widget reads after the processors spent `idle` of the
/// next `total` units of time idle.
fn cpu_label(idle: u64, total: u64) -> String {
    let system = FakeSystem::default();
    system.set(|m| m.cpu = Some(CpuTimes::default()));
    let mut tracker = CpuTracker::new(&system);
    system.set(|m| m.cpu = Some(CpuTimes { idle, total }));
    let config = bar_of(r#"{ type = "cpu" }"#);
    let readings = SystemReadings::read(&system, &config, Some(&mut tracker));
    label(&config.right[0], &BarState::default(), &readings)
}

#[test]
fn the_layout_widget_shows_the_layout_name() {
    // Arrange
    let state = BarState {
        layout: LayoutKind::VerticalStack,
        ..BarState::default()
    };

    // Act
    let label = label_of(r#"{ type = "layout" }"#, &state, &FakeSystem::default());

    // Assert
    assert_eq!(label, "VStack");
}

#[test]
fn monocle_mode_shows_after_the_layout_name() {
    // Arrange
    let state = BarState {
        monocle: true,
        ..BarState::default()
    };

    // Act
    let label = label_of(r#"{ type = "layout" }"#, &state, &FakeSystem::default());

    // Assert
    assert_eq!(label, "BSP | M");
}

#[test]
fn the_clock_shows_the_local_time_in_its_format() {
    // Arrange
    let system = at(LocalTime {
        hour: 21,
        minute: 4,
        second: 9,
        ..LocalTime::default()
    });

    // Act
    let label = label_of(
        r#"{ type = "clock", format = "%H:%M" }"#,
        &BarState::default(),
        &system,
    );

    // Assert
    assert_eq!(label, "21:04");
}

#[test]
fn the_date_shows_the_local_date_in_its_default_format() {
    // Arrange
    let system = at(LocalTime {
        year: 2026,
        month: 10,
        day: 13,
        weekday: 2,
        ..LocalTime::default()
    });

    // Act
    let label = label_of(r#"{ type = "date" }"#, &BarState::default(), &system);

    // Assert
    assert_eq!(label, "Tuesday 13 October 2026");
}

#[test]
fn the_ram_widget_shows_the_memory_in_use() {
    // Arrange
    let system = FakeSystem::default();
    system.set(|m| m.memory = Some(42));

    // Act
    let label = label_of(r#"{ type = "ram" }"#, &BarState::default(), &system);

    // Assert
    assert_eq!(label, "RAM: 42%");
}

#[test]
fn the_ram_widget_shows_zero_when_memory_cannot_be_read() {
    // Arrange
    let system = FakeSystem::default();

    // Act
    let label = label_of(r#"{ type = "ram" }"#, &BarState::default(), &system);

    // Assert
    assert_eq!(label, "RAM: 0%");
}

#[test]
fn the_cpu_widget_pads_the_usage_to_three_digits() {
    // Act
    let low = cpu_label(95, 100);
    let full = cpu_label(0, 100);

    // Assert
    assert_eq!(low, "CPU:   5%");
    assert_eq!(full, "CPU: 100%");
}

#[test]
fn the_update_widget_shows_the_update_notice() {
    // Arrange
    let state = BarState {
        update_text: "v0.10.0 available".into(),
        ..BarState::default()
    };

    // Act
    let shown = shows(r#"{ type = "update" }"#, &state, &FakeSystem::default());
    let label = label_of(r#"{ type = "update" }"#, &state, &FakeSystem::default());

    // Assert
    assert!(shown);
    assert_eq!(label, "v0.10.0 available");
}

#[test]
fn the_update_widget_hides_without_an_update() {
    // Act
    let shown = shows(
        r#"{ type = "update" }"#,
        &BarState::default(),
        &FakeSystem::default(),
    );

    // Assert
    assert!(!shown);
}

#[test]
fn the_paused_widget_shows_while_the_hotkeys_are_paused() {
    // Arrange
    let state = BarState {
        paused: true,
        ..BarState::default()
    };

    // Act
    let shown = shows(r#"{ type = "paused" }"#, &state, &FakeSystem::default());
    let label = label_of(r#"{ type = "paused" }"#, &state, &FakeSystem::default());

    // Assert
    assert!(shown);
    assert_eq!(label, "PAUSED");
}

#[test]
fn the_paused_widget_hides_while_the_hotkeys_work() {
    // Act
    let shown = shows(
        r#"{ type = "paused" }"#,
        &BarState::default(),
        &FakeSystem::default(),
    );

    // Assert
    assert!(!shown);
}

#[test]
fn a_disabled_widget_never_shows() {
    // Act
    let shown = shows(
        r#"{ type = "layout", enabled = false }"#,
        &BarState::default(),
        &FakeSystem::default(),
    );

    // Assert
    assert!(!shown);
}

#[test]
fn the_icon_goes_before_the_text() {
    // Act
    let label = label_of(
        r#"{ type = "layout", icon = "L" }"#,
        &BarState::default(),
        &FakeSystem::default(),
    );

    // Assert
    assert_eq!(label, "L BSP");
}

#[test]
fn a_widget_with_no_text_shows_only_its_icon() {
    // Act
    let label = label_of(
        r#"{ type = "clock", format = "", icon = "C" }"#,
        &BarState::default(),
        &FakeSystem::default(),
    );

    // Assert
    assert_eq!(label, "C");
}
