//! The requests a bar keeps tiles clear of its space with.

use serde::{Deserialize, Serialize};

use crate::StableMonitorId;
use crate::tiler::Edges;

/// The monitors a reservation is for: one, by id, or every one connected
/// now and later.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
pub enum MonitorTarget {
    /// Every monitor, `"all"` on the wire.
    #[default]
    All,
    /// The monitor with this id.
    One(StableMonitorId),
}

impl MonitorTarget {
    /// Whether it covers monitor `id`.
    pub fn covers(&self, id: &StableMonitorId) -> bool {
        match self {
            Self::All => true,
            Self::One(one) => one == id,
        }
    }
}

impl From<String> for MonitorTarget {
    fn from(text: String) -> Self {
        if text == "all" {
            Self::All
        } else {
            Self::One(StableMonitorId::new(text))
        }
    }
}

impl From<MonitorTarget> for String {
    fn from(target: MonitorTarget) -> Self {
        match target {
            MonitorTarget::All => "all".to_string(),
            MonitorTarget::One(id) => id.as_str().to_string(),
        }
    }
}

/// How far a bar reaches into a monitor from each edge, in the desktop's
/// pixels, which tiles keep clear of while its session lasts. Edges left
/// out reach nothing; all of them at 0 ends the reservation.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub struct Reserve {
    /// The monitor, by id, or `"all"`.
    #[cfg_attr(test, schemars(with = "String"))]
    pub monitor: MonitorTarget,
    /// Down from the top edge.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub top: u32,
    /// Up from the bottom edge.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub bottom: u32,
    /// In from the left edge.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub left: u32,
    /// In from the right edge.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub right: u32,
}

impl Reserve {
    /// What it asks for, on each edge.
    pub fn edges(&self) -> Edges {
        let px = |edge: u32| i32::try_from(edge).unwrap_or(i32::MAX);
        Edges::new(px(self.top), px(self.bottom), px(self.left), px(self.right))
    }
}

/// Ends a session's reservations.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
#[serde(default)]
pub struct Release {
    /// The one made for this monitor, or `"all"`, or every one the session
    /// holds when `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, schemars(with = "Option<String>"))]
    pub monitor: Option<MonitorTarget>,
}

fn is_zero(edge: &u32) -> bool {
    *edge == 0
}
