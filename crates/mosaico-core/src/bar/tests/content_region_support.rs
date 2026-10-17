//! Content built from a bar config, and the regions a platform would
//! give its items.

use crate::Rect;
use crate::bar::fakes::FakeSystem;
use crate::bar::{BarContent, BarSide, BarState, BarTarget, ItemRegion, SystemReadings, WidgetRef};
use crate::config::BarConfig;

/// A bar config written the way `bar.toml` is.
pub(super) fn bar(toml: &str) -> BarConfig {
    toml::from_str(toml).expect("a valid bar.toml")
}

/// What a bar with `config` shows on a monitor in the default state.
pub(super) fn content(config: &BarConfig) -> BarContent {
    let readings = SystemReadings::read(&FakeSystem::default(), config, None);
    BarContent::build(config, &BarState::default(), &readings)
}

/// Where the tests say an item was drawn.
pub(super) const AT: Rect = Rect {
    x: 10,
    y: 0,
    width: 50,
    height: 30,
};

/// The region of the widget at `index` of `side`, drawn at [`AT`].
pub(super) fn widget(side: BarSide, index: usize) -> Option<ItemRegion> {
    Some(ItemRegion {
        target: BarTarget::Widget(WidgetRef { side, index }),
        clickable: false,
        rect: AT,
    })
}
