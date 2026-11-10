//! mosaico's own bar as a provider: where it makes bars, how far they
//! reach, and what it draws from the bar API's state.

use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::api::{Monitor, State, Theme as ApiTheme};
use crate::bar::LocalTime;
use crate::bar::fakes::BarEvent::{self, Created, Drawn, Dropped};
use crate::bar::fakes::{FakeRenderer, FakeSystem, NoPointer};
use crate::bar::refresh::READING_INTERVAL;
use crate::config::{BarConfig, CatppuccinFlavor, Theme};
use crate::tiler::{MonitorPlace, Reach};
use crate::{Rect, StableMonitorId};

use super::{BarProvider, BuiltinBar};

/// Two 1920 x 1080 monitors, side by side, with the system keeping
/// `taskbar` pixels at the top of each.
fn places(taskbar: i32) -> Vec<MonitorPlace> {
    (0..2)
        .map(|index| MonitorPlace {
            id: StableMonitorId::new(format!("m-{index}")),
            index,
            bounds: Rect::new(index as i32 * 1920, 0, 1920, 1080),
            work_area: Rect::new(index as i32 * 1920, taskbar, 1920, 1080 - taskbar),
            scale: 1.0,
        })
        .collect()
}

fn start(renderer: &FakeRenderer) -> BuiltinBar {
    start_with(renderer, &FakeSystem::default(), BarConfig::default())
}

/// A bar with `config`, reading the machine `system`.
fn start_with(renderer: &FakeRenderer, system: &FakeSystem, config: BarConfig) -> BuiltinBar {
    let mocha = Theme::Catppuccin(CatppuccinFlavor::Mocha);
    let system = Box::new(system.clone());
    BuiltinBar::new(renderer.boxed(), system, Rc::new(NoPointer), config, mocha)
}

/// A bar with `widgets`, inline tables of `bar.toml`, on its right, placed
/// on two monitors and shown a state at `now`, with the clock 15 seconds
/// past the minute.
fn shown_with(renderer: &FakeRenderer, widgets: &str, now: Instant) -> BuiltinBar {
    let system = FakeSystem::default();
    system.set(|machine| {
        machine.time = LocalTime {
            second: 15,
            ..LocalTime::default()
        }
    });
    let config = toml::from_str(&format!("left = []\nright = [{widgets}]")).unwrap_or_default();
    let mut bar = start_with(renderer, &system, config);
    let places = places(0);
    bar.place(&places);
    bar.show(&state(&places, "catppuccin"), now);
    renderer.take();
    bar
}

fn draws(events: &[BarEvent]) -> usize {
    events.iter().filter(|e| matches!(e, Drawn { .. })).count()
}

/// A state with the monitors of `places`, in the theme `name`.
fn state(places: &[MonitorPlace], name: &str) -> State {
    let monitors = (places.iter())
        .map(|place| Monitor {
            id: place.id.clone(),
            index: place.index as u32,
            active_workspace: 1,
            ..Monitor::default()
        })
        .collect();
    State {
        theme: ApiTheme {
            name: name.into(),
            ..ApiTheme::default()
        },
        monitors,
        ..State::default()
    }
}

fn backgrounds(events: &[BarEvent]) -> Vec<String> {
    (events.iter())
        .filter_map(|event| match event {
            Drawn { background, .. } => Some(background.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn it_makes_a_bar_on_each_monitor_once_placed() {
    // Arrange
    let renderer = FakeRenderer::default();
    let mut bar = start(&renderer);
    let places = places(0);

    // Act
    bar.place(&places);
    bar.place(&places);

    // Assert
    let areas: Vec<Rect> = places.iter().map(|p| p.work_area).collect();
    assert_eq!(renderer.take(), [Created(areas[0]), Created(areas[1])]);
}

#[test]
fn a_change_of_the_work_areas_moves_the_bars() {
    // Arrange
    let renderer = FakeRenderer::default();
    let mut bar = start(&renderer);
    bar.place(&places(0));
    renderer.take();

    // Act
    bar.place(&places(40));

    // Assert
    let events = renderer.take();
    assert!(
        events.contains(&Dropped(places(0)[0].work_area)),
        "{events:?}"
    );
    assert!(
        events.contains(&Created(places(40)[1].work_area)),
        "{events:?}"
    );
}

#[test]
fn the_bars_reach_their_height_below_what_the_system_keeps() {
    // Arrange
    let renderer = FakeRenderer::default();
    let mut bar = start(&renderer);
    let places = places(40);
    bar.place(&places);

    // Act
    let reach = bar.reach(&places);

    // Assert
    let height = BarConfig::default().height;
    assert_eq!(reach, Reach::bar_on(&places, height, &[0, 1]));
    assert_eq!(reach.top(&places[1].id), 40 + height);
}

#[test]
fn a_bar_turned_off_reaches_nothing_past_what_the_system_keeps() {
    // Arrange
    let renderer = FakeRenderer::default();
    let mut bar = start(&renderer);
    let places = places(40);
    bar.place(&places);

    // Act
    bar.reload(toml::from_str("enabled = false").unwrap_or_default());

    // Assert
    assert_eq!(bar.reach(&places).top(&places[0].id), 40);
}

#[test]
fn it_draws_in_the_theme_the_state_names() {
    // Arrange
    let renderer = FakeRenderer::default();
    let mut bar = start(&renderer);
    let places = places(0);
    bar.place(&places);
    bar.show(&state(&places, "catppuccin"), Instant::now());
    let mocha = backgrounds(&renderer.take());

    // Act
    bar.show(&state(&places, "rose-pine"), Instant::now());

    // Assert
    let rose_pine = backgrounds(&renderer.take());
    assert_eq!(rose_pine.len(), 2);
    assert_ne!(mocha, rose_pine);
}

#[test]
fn a_clock_without_seconds_is_due_as_the_minute_turns() {
    // Arrange
    let renderer = FakeRenderer::default();
    let now = Instant::now();

    // Act
    let bar = shown_with(&renderer, r#"{ type = "clock", format = "%H:%M" }"#, now);

    // Assert
    assert_eq!(bar.due(), Some(now + Duration::from_secs(45)));
}

#[test]
fn a_wake_draws_the_bars_again_only_once_the_clock_turns() {
    // Arrange
    let renderer = FakeRenderer::default();
    let now = Instant::now();
    let mut bar = shown_with(&renderer, r#"{ type = "clock", format = "%H:%M" }"#, now);

    // Act
    bar.wake(now + Duration::from_secs(44));
    let early = draws(&renderer.take());
    bar.wake(now + Duration::from_secs(45));
    let turned = draws(&renderer.take());

    // Assert
    assert_eq!((early, turned), (0, 2));
    assert_eq!(bar.due(), Some(now + Duration::from_secs(90)));
}

#[test]
fn a_reading_of_the_machine_is_due_every_second() {
    // Arrange
    let renderer = FakeRenderer::default();
    let now = Instant::now();

    // Act
    let bar = shown_with(&renderer, r#"{ type = "ram" }"#, now);

    // Assert
    assert_eq!(bar.due(), Some(now + READING_INTERVAL));
}

#[test]
fn a_bar_that_shows_nothing_changing_on_its_own_is_never_due() {
    // Arrange
    let renderer = FakeRenderer::default();

    // Act
    let bar = shown_with(&renderer, r#"{ type = "workspaces" }"#, Instant::now());

    // Assert
    assert_eq!(bar.due(), None);
}
