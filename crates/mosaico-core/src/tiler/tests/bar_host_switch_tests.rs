//! Picking another bar while the daemon runs, and a provider that wants
//! no state.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Instant;

use crate::api;
use crate::bar::{BarHost, BarProvider};
use crate::config::{BarChoice, Theme};

use super::bar_host_tests::{Recorder, Told, host, host_on, picked};
use super::fake_platform::{Call, FakePlatform};
use super::manager_support::{start, still};

#[test]
fn another_bar_is_placed_only_once_the_old_bars_are_off_screen() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let mut manager = start(&platform, None, &still());
    let recorder = Recorder::default();
    let mut bars = host(&recorder);
    bars.start(&mut manager);
    recorder.take();

    // Act -- the built-in bar is made again from the same recorder.
    let off = bars.choose(&mut manager, &picked(BarChoice::Off), Instant::now());
    let stopped = recorder.take();
    let on = bars.choose(&mut manager, &picked(BarChoice::Mosaico), Instant::now());

    // Assert
    assert!(off && on);
    assert_eq!(stopped, [Told::Stopped]);
    let told = recorder.take();
    assert!(
        matches!(told[..], [Told::Placed(_), Told::Reach, Told::Shown(_)]),
        "{told:?}"
    );
}

#[test]
fn picking_the_bar_that_runs_changes_nothing() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let mut manager = start(&platform, None, &still());
    let recorder = Recorder::default();
    let mut bars = host(&recorder);
    bars.start(&mut manager);
    recorder.take();

    // Act
    let changed = bars.choose(&mut manager, &picked(BarChoice::Mosaico), Instant::now());

    // Assert
    assert!(!changed);
    assert_eq!(recorder.take(), []);
}

#[test]
fn the_built_in_bar_is_made_only_once_it_is_picked() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let mut manager = start(&platform, None, &still());
    let made = Rc::new(Cell::new(0));
    let count = Rc::clone(&made);
    let make = Box::new(move |_: Theme| {
        count.set(count.get() + 1);
        Box::new(Recorder::default()) as Box<dyn BarProvider>
    });
    let mut bars = BarHost::new(
        &picked(BarChoice::External),
        make,
        api::Daemon::default(),
        mocha(),
    );
    bars.start(&mut manager);
    let before = made.get();

    // Act
    bars.choose(&mut manager, &picked(BarChoice::Mosaico), Instant::now());

    // Assert
    assert_eq!((before, made.get()), (0, 1));
    assert_eq!(bars.choice(), BarChoice::Mosaico);
}

#[test]
fn a_switch_between_bars_that_keep_the_same_space_asks_nothing_of_the_platform() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    let mut manager = start(&platform, None, &still());
    let mut bars = host_on(&Recorder::default(), BarChoice::External);
    bars.start(&mut manager);
    platform.take_calls();

    // Act
    bars.choose(&mut manager, &picked(BarChoice::Off), Instant::now());

    // Assert -- not even a border is drawn again.
    let calls = platform.take_calls();
    assert!(calls.is_empty(), "{calls:?}");
}

#[test]
fn no_bar_gives_the_tiles_the_built_in_bars_space_at_once() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    let mut manager = start(&platform, None, &still());
    let mut bars = host(&Recorder::default());
    bars.start(&mut manager);
    platform.take_calls();

    // Act
    bars.choose(&mut manager, &picked(BarChoice::Off), Instant::now());

    // Assert
    assert_eq!(manager.tiler().monitors()[0].work_area().y, 0);
    let calls = platform.take_calls();
    let placed = calls.iter().filter(|c| matches!(c, Call::Place(..)));
    assert_eq!(placed.count(), 2, "{calls:?}");
}

#[test]
fn no_state_is_built_for_a_bar_that_wants_none_until_it_does() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let mut manager = start(&platform, None, &still());
    let recorder = Recorder::default();
    recorder.1.set(true);
    let mut bars = host(&recorder);

    // Act
    bars.start(&mut manager);
    bars.tick(&mut manager, Instant::now());
    let built = manager.last_bar_state().monitors.len();
    recorder.1.set(false);
    bars.show(&mut manager);

    // Assert -- once it wants the state, it is shown the state it missed.
    assert_eq!(built, 0);
    let shown = recorder.shown();
    assert_eq!(shown.len(), 1, "{shown:?}");
    assert_eq!(shown[0].monitors.len(), 1);
}

fn mocha() -> Theme {
    Theme::Catppuccin(crate::config::CatppuccinFlavor::Mocha)
}
