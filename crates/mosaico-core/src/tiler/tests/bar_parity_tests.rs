//! The built-in bar draws the same from the bar API's JSON as from the
//! state the tiling manager builds, so it shows nothing an external bar
//! could not.

use std::rc::Rc;

use crate::api;
use crate::bar::fakes::{BarEvent, FakeRenderer, FakeSystem, NoPointer};
use crate::bar::{BarProvider, BuiltinBar};
use crate::config::{BarConfig, CatppuccinFlavor, Theme};

use super::fake_platform::FakePlatform;
use super::manager_support::{start, still};
use super::*;

/// What a built-in bar placed on `manager`'s monitors draws for `state`.
fn drawn(manager: &TilingManager<FakePlatform>, state: &api::State) -> Vec<BarEvent> {
    let renderer = FakeRenderer::default();
    let mocha = Theme::Catppuccin(CatppuccinFlavor::Mocha);
    let system = Box::new(FakeSystem::default());
    let config = BarConfig::default();
    let mut bar = BuiltinBar::new(renderer.boxed(), system, Rc::new(NoPointer), config, mocha);
    bar.place(&manager.places());
    renderer.take();
    bar.show(state, std::time::Instant::now());
    renderer.take()
}

#[test]
fn the_built_in_bar_draws_the_same_from_the_json_as_from_the_state() {
    // Arrange -- two monitors, one in monocle, the hotkeys paused, a newer
    // version on offer and a theme of its own.
    let platform = FakePlatform::new(2, &[(1, 0), (2, 0), (3, 1)]);
    let mut manager = start(&platform, None, &still());
    manager.handle_action(&Action::ToggleMonocle);
    let daemon = api::Daemon {
        paused: true,
        update: Some("v9.0.0 available".into()),
        ..api::Daemon::default()
    };
    let theme = api::Theme::of(Theme::Catppuccin(CatppuccinFlavor::Latte));
    let built = manager
        .bar_state(&daemon, &theme)
        .cloned()
        .unwrap_or_default();

    // Act
    let json = serde_json::to_string(&built).unwrap_or_default();
    let decoded: api::State = serde_json::from_str(&json).unwrap_or_default();

    // Assert
    let from_state = drawn(&manager, &built);
    assert_eq!(from_state.len(), 2, "{from_state:?}");
    assert_eq!(drawn(&manager, &decoded), from_state);
}
