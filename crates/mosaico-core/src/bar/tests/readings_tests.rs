use super::*;
use crate::bar::CpuTimes;
use crate::bar::fakes::FakeSystem;

/// A bar config written the way `bar.toml` is.
fn bar(toml: &str) -> BarConfig {
    toml::from_str(toml).expect("a valid bar.toml")
}

/// A bar with `widgets`, inline tables of `bar.toml`, on its right.
fn right(widgets: &str) -> BarConfig {
    bar(&format!("left = []\nright = [{widgets}]"))
}

#[test]
fn memory_is_read_when_a_ram_widget_shows() {
    // Arrange
    let system = FakeSystem::default();
    system.set(|m| m.memory = Some(42));

    // Act
    let readings = SystemReadings::read(&system, &right(r#"{ type = "ram" }"#), None);

    // Assert
    assert_eq!(readings.memory_usage, 42);
    assert_eq!(system.reads().memory, 1);
}

#[test]
fn a_ram_widget_on_the_left_reads_memory_too() {
    // Arrange
    let system = FakeSystem::default();
    system.set(|m| m.memory = Some(42));
    let config = bar("right = []\nleft = [{ type = \"ram\" }]");

    // Act
    let readings = SystemReadings::read(&system, &config, None);

    // Assert
    assert_eq!(readings.memory_usage, 42);
}

#[test]
fn memory_is_not_read_without_an_enabled_ram_widget() {
    // Arrange
    let system = FakeSystem::default();
    system.set(|m| m.memory = Some(42));
    let config = right(r#"{ type = "ram", enabled = false }, { type = "clock" }"#);

    // Act
    let readings = SystemReadings::read(&system, &config, None);

    // Assert
    assert_eq!(readings.memory_usage, 0);
    assert_eq!(system.reads().memory, 0);
}

#[test]
fn memory_that_cannot_be_read_shows_as_zero() {
    // Arrange
    let system = FakeSystem::default();

    // Act
    let readings = SystemReadings::read(&system, &right(r#"{ type = "ram" }"#), None);

    // Assert
    assert_eq!(readings.memory_usage, 0);
}

#[test]
fn media_is_read_when_a_media_widget_shows() {
    // Arrange
    let system = FakeSystem::default();
    let playing = NowPlaying {
        artist: "Artist".into(),
        title: "Title".into(),
    };
    system.set(|m| m.playing = Some(playing.clone()));

    // Act
    let readings = SystemReadings::read(&system, &right(r#"{ type = "media" }"#), None);

    // Assert
    assert_eq!(readings.now_playing, Some(playing));
    assert_eq!(system.reads().media, 1);
}

#[test]
fn media_is_not_read_without_a_media_widget() {
    // Arrange
    let system = FakeSystem::default();

    // Act
    SystemReadings::read(&system, &right(r#"{ type = "clock" }"#), None);

    // Assert
    assert_eq!(system.reads().media, 0);
}

#[test]
fn media_that_cannot_be_read_shows_as_nothing_playing() {
    // Arrange
    let system = FakeSystem::default();
    system.set(|m| m.media_fails = true);

    // Act
    let readings = SystemReadings::read(&system, &right(r#"{ type = "media" }"#), None);

    // Assert
    assert_eq!(readings.now_playing, None);
}

#[test]
fn cpu_usage_comes_from_the_tracker() {
    // Arrange -- 30 of the next 100 units are idle.
    let system = FakeSystem::default();
    system.set(|m| m.cpu = Some(CpuTimes::default()));
    let mut tracker = CpuTracker::new(&system);
    system.set(|m| {
        m.cpu = Some(CpuTimes {
            idle: 30,
            total: 100,
        })
    });

    // Act
    let readings = SystemReadings::read(&system, &right(r#"{ type = "cpu" }"#), Some(&mut tracker));

    // Assert
    assert_eq!(readings.cpu_usage, 70);
}

#[test]
fn without_a_tracker_the_cpu_is_not_read() {
    // Arrange
    let system = FakeSystem::default();

    // Act
    let readings = SystemReadings::read(&system, &right(r#"{ type = "cpu" }"#), None);

    // Assert
    assert_eq!(readings.cpu_usage, 0);
    assert_eq!(system.reads().cpu, 0);
}

#[test]
fn the_local_time_is_read_when_a_clock_or_a_date_shows() {
    // Arrange
    let system = FakeSystem::default();
    let time = LocalTime {
        hour: 9,
        minute: 30,
        ..LocalTime::default()
    };
    system.set(|m| m.time = time);

    // Act
    let clock = SystemReadings::read(&system, &right(r#"{ type = "clock" }"#), None);
    let date = SystemReadings::read(&system, &right(r#"{ type = "date" }"#), None);

    // Assert
    assert_eq!(clock.time, time);
    assert_eq!(date.time, time);
}

#[test]
fn the_local_time_is_not_read_without_an_enabled_clock_or_date() {
    // Arrange
    let system = FakeSystem::default();
    let config = right(r#"{ type = "clock", enabled = false }, { type = "ram" }"#);

    // Act
    SystemReadings::read(&system, &config, None);

    // Assert
    assert_eq!(system.reads().time, 0);
}
