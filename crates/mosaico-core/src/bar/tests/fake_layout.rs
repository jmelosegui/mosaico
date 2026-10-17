//! Where a fake bar draws its items: every item at a fixed width, laid
//! out as a platform lays them out, so the tests know where to point.
//!
//! The left side starts at [`EDGE`] and the right side ends [`EDGE`] in
//! from the bar's right edge, with [`GAP`] between items. A pill is
//! [`PILL`] wide, a separator [`SEPARATOR`], the window icon [`ICON`] and
//! a workspace label [`LABEL`], with [`LABEL_GAP`] between labels and
//! [`INSET`] between the labels and their container's edges.

use crate::Rect;

use super::super::{BarContent, BarItem, BarSide, ItemRegion};

pub(in crate::bar) const EDGE: i32 = 8;
pub(in crate::bar) const GAP: i32 = 10;
pub(in crate::bar) const PILL: i32 = 100;
pub(in crate::bar) const SEPARATOR: i32 = 12;
pub(in crate::bar) const ICON: i32 = 40;
pub(in crate::bar) const LABEL: i32 = 20;
pub(in crate::bar) const LABEL_GAP: i32 = 4;
pub(in crate::bar) const INSET: i32 = 2;

/// The regions of `content` on a bar `width` wide and `height` tall, in
/// the order a platform draws them: the left side, then the right.
pub(super) fn regions(content: &BarContent, width: i32, height: i32) -> Vec<ItemRegion> {
    let mut regions = Vec::new();
    let mut x = EDGE;
    for (index, item) in content.left.iter().enumerate() {
        let wide = item_width(item);
        let rect = Rect::new(x, 0, wide, height);
        regions.extend(content.region(BarSide::Left, index, rect));
        if let BarItem::Workspaces { labels, .. } = item {
            for (k, _) in labels.iter().enumerate() {
                let left = x + INSET + k as i32 * (LABEL + LABEL_GAP);
                regions.push(content.label_region(k, Rect::new(left, 0, LABEL, height)));
            }
        }
        x += wide + GAP;
    }
    let mut right = width - EDGE;
    for (index, item) in content.right.iter().enumerate() {
        let wide = item_width(item);
        right -= wide;
        let rect = Rect::new(right, 0, wide, height);
        regions.extend(content.region(BarSide::Right, index, rect));
        right -= GAP;
    }
    regions
}

/// How wide the fake bar draws `item`.
fn item_width(item: &BarItem) -> i32 {
    match item {
        BarItem::Workspaces { labels, .. } => {
            let count = labels.len() as i32;
            INSET * 2 + count * LABEL + (count - 1).max(0) * LABEL_GAP
        }
        BarItem::WindowIcon(Some(_)) => ICON,
        BarItem::WindowIcon(None) => 0,
        BarItem::Pill { .. } => PILL,
        BarItem::Separator(_) => SEPARATOR,
    }
}
