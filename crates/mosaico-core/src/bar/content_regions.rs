//! What each item of a bar is to the pointer, where a platform draws it.

use crate::Rect;

use super::super::{BarSide, BarTarget, ItemRegion, WidgetRef};
use super::BarContent;

impl BarContent {
    /// The region of the item at `index` of `side`, drawn at `rect` in bar
    /// coordinates: the widget the item shows, which takes no clicks yet.
    /// `None` for a separator, which the pointer passes over, and for an
    /// index past the items of that side.
    pub fn region(&self, side: BarSide, index: usize, rect: Rect) -> Option<ItemRegion> {
        let widgets = match side {
            BarSide::Left => &self.left_widgets,
            BarSide::Right => &self.right_widgets,
        };
        let index = (*widgets.get(index)?)?;
        Some(ItemRegion {
            target: BarTarget::Widget(WidgetRef { side, index }),
            clickable: false,
            rect,
        })
    }

    /// The region of the label of workspace `workspace`, from 0, drawn at
    /// `rect` in bar coordinates. A click on it goes to that workspace,
    /// unless the bar does not take clicks.
    pub fn label_region(&self, workspace: usize, rect: Rect) -> ItemRegion {
        ItemRegion {
            target: BarTarget::Workspace(workspace),
            clickable: self.labels_clickable,
            rect,
        }
    }
}

#[cfg(test)]
#[path = "tests/content_region_support.rs"]
mod support;

#[cfg(test)]
#[path = "tests/content_region_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/content_click_tests.rs"]
mod click_tests;
