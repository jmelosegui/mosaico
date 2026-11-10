//! A monitor named on the command line.

use std::str::FromStr;

use serde_json::Value;

use super::JsonState;

/// A monitor as a user names it: by id, such as `m-41c6e2`, or by its
/// index from the left, such as `0`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonitorRef {
    /// The monitor with this id.
    Id(String),
    /// The monitor at this index from the left.
    Index(u64),
}

impl MonitorRef {
    /// Whether `monitor`, one of the state's monitors, is the one named.
    pub fn matches(&self, monitor: &Value) -> bool {
        match self {
            Self::Id(id) => monitor["id"].as_str() == Some(id),
            Self::Index(index) => monitor["index"].as_u64() == Some(*index),
        }
    }

    /// The id of the monitor named: the id given, or the id of the monitor
    /// at the index in `state`, if it has one.
    pub fn resolve(&self, state: &JsonState) -> Option<String> {
        match self {
            Self::Id(id) => Some(id.clone()),
            index => state.monitor_id(index),
        }
    }
}

impl FromStr for MonitorRef {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        if text.is_empty() {
            return Err("a monitor is named by its id or its index".to_string());
        }
        Ok(match text.parse::<u64>() {
            Ok(index) => Self::Index(index),
            Err(_) => Self::Id(text.to_string()),
        })
    }
}
