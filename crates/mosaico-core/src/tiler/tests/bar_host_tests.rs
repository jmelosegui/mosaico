//! The daemon's side of the bar contract on a fake platform: the order in
//! which a provider is placed, asked how far it reaches and shown the
//! state.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Instant;

use crate::api;
use crate::bar::{BarHost, BarProvider};
use crate::config::{BarChoice, BarSection, CatppuccinFlavor, Theme};
use crate::tiler::MonitorPlace;

use super::fake_platform::FakePlatform;
use super::manager_support::{start, still};
use super::*;

const HEIGHT: i32 = 30;

#[derive(Debug, Clone, PartialEq)]
pub(super) enum Told {
    Placed(Vec<Rect>),
    Reach,
    Shown(Box<api::State>),
    Refreshed,
    Stopped,
}

/// A provider that notes what it is told, with a bar `HEIGHT` high on
/// every monitor. It wants no state while its flag is set.
#[derive(Default, Clone)]
pub(super) struct Recorder(Rc<RefCell<Vec<Told>>>, pub(super) Rc<Cell<bool>>);

impl Recorder {
    pub(super) fn take(&self) -> Vec<Told> {
        std::mem::take(&mut self.0.borrow_mut())
    }

    pub(super) fn shown(&self) -> Vec<api::State> {
        (self.take().into_iter())
            .filter_map(|told| match told {
                Told::Shown(state) => Some(*state),
                _ => None,
            })
            .collect()
    }
}

impl BarProvider for Recorder {
    fn name(&self) -> &str {
        "recorder"
    }

    fn place(&mut self, places: &[MonitorPlace]) {
        let areas = places.iter().map(|place| place.work_area).collect();
        self.0.borrow_mut().push(Told::Placed(areas));
    }

    fn reach(&self, places: &[MonitorPlace]) -> Reach {
        self.0.borrow_mut().push(Told::Reach);
        Reach::bar_on(places, HEIGHT, &[])
    }

    fn show(&mut self, state: &api::State) {
        self.0
            .borrow_mut()
            .push(Told::Shown(Box::new(state.clone())));
    }

    fn refresh(&mut self) {
        self.0.borrow_mut().push(Told::Refreshed);
    }

    fn wants_state(&self) -> bool {
        !self.1.get()
    }

    fn stop(&mut self) {
        self.0.borrow_mut().push(Told::Stopped);
    }
}

pub(super) fn host(recorder: &Recorder) -> BarHost {
    host_on(recorder, BarChoice::Mosaico)
}

/// The `[bar]` table that picks `choice` and keeps no space.
pub(super) fn picked(choice: BarChoice) -> BarSection {
    BarSection {
        provider: choice.name().to_string(),
        ..BarSection::default()
    }
}

/// A host on `choice`, whose built-in bar, once picked, is `recorder`.
pub(super) fn host_on(recorder: &Recorder, choice: BarChoice) -> BarHost {
    let mocha = Theme::Catppuccin(CatppuccinFlavor::Mocha);
    let recorder = recorder.clone();
    let make = Box::new(move |_| Box::new(recorder.clone()) as Box<dyn BarProvider>);
    BarHost::new(&picked(choice), make, api::Daemon::default(), mocha)
}

#[test]
fn a_started_bar_is_placed_then_asked_its_reach_then_shown() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let mut manager = start(&platform, None, &still());
    let recorder = Recorder::default();
    let mut bars = host(&recorder);
    let screen = Rect::new(0, 0, MON_W, MON_H);

    // Act
    bars.start(&mut manager);

    // Assert -- the tiles keep clear of the bar, and the state names the
    // provider and the theme.
    let told = recorder.take();
    assert_eq!(told[..2], [Told::Placed(vec![screen]), Told::Reach]);
    let Told::Shown(state) = &told[2] else {
        panic!("{told:?}");
    };
    assert_eq!(state.daemon.provider, "recorder");
    assert_eq!(state.theme.name, "catppuccin");
    assert_eq!(manager.tiler().monitors()[0].work_area().y, HEIGHT);
}

#[test]
fn the_state_is_shown_again_only_once_it_changed() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let mut manager = start(&platform, None, &still());
    let recorder = Recorder::default();
    let mut bars = host(&recorder);
    bars.start(&mut manager);
    recorder.take();

    // Act
    bars.show(&mut manager);
    bars.set_paused(true);
    bars.show(&mut manager);

    // Assert
    let shown = recorder.shown();
    assert_eq!(shown.len(), 1, "{shown:?}");
    assert!(shown[0].daemon.paused);
}

#[test]
fn an_empty_update_notice_is_none() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let mut manager = start(&platform, None, &still());
    let recorder = Recorder::default();
    let mut bars = host(&recorder);
    bars.start(&mut manager);
    recorder.take();

    // Act
    bars.set_update("v9.0.0 available");
    bars.show(&mut manager);
    bars.set_update("");
    bars.show(&mut manager);

    // Assert
    let updates: Vec<_> = recorder
        .shown()
        .into_iter()
        .map(|s| s.daemon.update)
        .collect();
    assert_eq!(updates, [Some("v9.0.0 available".to_string()), None]);
}

#[test]
fn a_display_change_moves_the_bars_before_the_tiles() {
    // Arrange
    let platform = FakePlatform::new(2, &[(1, 0), (2, 1)]);
    let mut manager = start(&platform, None, &still());
    let recorder = Recorder::default();
    let mut bars = host(&recorder);
    bars.start(&mut manager);
    recorder.take();

    // Act -- the second monitor leaves.
    bars.display_changed(&mut manager, displays(1));
    bars.show(&mut manager);

    // Assert
    let told = recorder.take();
    assert!(
        matches!(&told[0], Told::Placed(areas) if areas.len() == 1),
        "{told:?}"
    );
    assert_eq!(told[1], Told::Reach);
    let Told::Shown(state) = &told[2] else {
        panic!("{told:?}");
    };
    assert_eq!(state.monitors.len(), 1);
    assert_eq!(manager.tiler().monitors()[0].work_area().y, HEIGHT);
}

#[test]
fn bars_placed_again_are_shown_the_state_though_it_did_not_change() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let mut manager = start(&platform, None, &still());
    let recorder = Recorder::default();
    let mut bars = host(&recorder);
    bars.start(&mut manager);
    recorder.take();

    // Act
    let read = bars.reset_work_areas(&mut manager);
    bars.show(&mut manager);

    // Assert
    assert!(read.is_ok());
    let told = recorder.take();
    assert!(
        matches!(told[..], [Told::Placed(_), Told::Reach, Told::Shown(_)]),
        "{told:?}"
    );
}

#[test]
fn stopping_takes_the_bars_off_screen() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let mut manager = start(&platform, None, &still());
    let recorder = Recorder::default();
    let mut bars = host(&recorder);
    bars.start(&mut manager);
    recorder.take();

    // Act
    bars.stop();

    // Assert
    assert_eq!(recorder.take(), [Told::Stopped]);
}

#[test]
fn a_tick_draws_the_bars_once() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let mut manager = start(&platform, None, &still());
    let recorder = Recorder::default();
    let mut bars = host(&recorder);
    bars.start(&mut manager);
    recorder.take();

    // Act -- one tick with nothing new, one with an update notice.
    bars.tick(&mut manager, Instant::now());
    bars.set_update("v9.0.0 available");
    bars.tick(&mut manager, Instant::now());

    // Assert
    let told = recorder.take();
    assert!(
        matches!(told[..], [Told::Refreshed, Told::Shown(_)]),
        "{told:?}"
    );
}
