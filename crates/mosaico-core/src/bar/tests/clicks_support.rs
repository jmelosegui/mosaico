//! The `[mouse]` sections the tests of widget clicks read.

use crate::Action;
use crate::bar::{BarSide, Button, WidgetClicks};
use crate::config::{BarConfig, BarMouseConfig, ClickBinding, WidgetKind};

/// The `[mouse]` section of a `bar.toml` holding `toml`.
pub(super) fn mouse(toml: &str) -> BarMouseConfig {
    let config: BarConfig = toml::from_str(toml).expect("a valid bar.toml");
    config.mouse
}

/// The `[mouse]` section with `entries` as its `[[mouse.click]]`.
pub(super) fn with(entries: &[ClickBinding]) -> BarMouseConfig {
    BarMouseConfig {
        click: entries.to_vec(),
        ..BarMouseConfig::default()
    }
}

/// What a click of `button` runs on a widget of `kind` drawn on the left.
pub(super) fn left(clicks: &WidgetClicks, kind: WidgetKind, button: Button) -> Option<Action> {
    clicks.action(kind, BarSide::Left, button)
}
