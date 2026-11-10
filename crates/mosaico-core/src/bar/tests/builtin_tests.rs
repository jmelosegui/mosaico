//! mosaico's own bar as a provider: where it makes bars, how far they
//! reach, and what it draws from the bar API's state.

use std::rc::Rc;

use crate::api::{Monitor, State, Theme as ApiTheme};
use crate::bar::fakes::BarEvent::{self, Created, Drawn, Dropped};
use crate::bar::fakes::{FakeRenderer, FakeSystem, NoPointer};
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
    let mocha = Theme::Catppuccin(CatppuccinFlavor::Mocha);
    let system = Box::new(FakeSystem::default());
    let config = BarConfig::default();
    BuiltinBar::new(renderer.boxed(), system, Rc::new(NoPointer), config, mocha)
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
    bar.show(&state(&places, "catppuccin"));
    let mocha = backgrounds(&renderer.take());

    // Act
    bar.show(&state(&places, "rose-pine"));

    // Assert
    let rose_pine = backgrounds(&renderer.take());
    assert_eq!(rose_pine.len(), 2);
    assert_ne!(mocha, rose_pine);
}
