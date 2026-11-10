//! The state as a client was sent it, kept as JSON.

use serde_json::{Map, Value};

use super::MonitorRef;

/// The state as a client was sent it: a snapshot with every update since
/// applied, as [`State::apply`](crate::api::State::apply) applies them.
///
/// It is kept as JSON rather than as [`State`](crate::api::State), so the
/// fields a newer daemon adds are kept and passed on too.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct JsonState(Map<String, Value>);

impl JsonState {
    /// Starts again from `snapshot`, which carries every topic followed.
    pub fn reset(&mut self, snapshot: &Value) {
        self.0 = snapshot.as_object().cloned().unwrap_or_default();
    }

    /// Takes in `update`: replaces each topic it carries, and each monitor
    /// by id, drops the monitors it removes, and keeps the monitors in
    /// `index` order.
    pub fn apply(&mut self, update: &Value) {
        let Some(update) = update.as_object() else {
            return;
        };
        let gone = update
            .get("removed")
            .and_then(|removed| removed["monitors"].as_array());
        if let (Some(Value::Array(monitors)), Some(gone)) = (self.0.get_mut("monitors"), gone) {
            monitors.retain(|m| !gone.contains(&m["id"]));
        }
        for (key, value) in update {
            match key.as_str() {
                "removed" => {}
                "monitors" => self.merge_monitors(value),
                _ => {
                    self.0.insert(key.clone(), value.clone());
                }
            }
        }
    }

    fn merge_monitors(&mut self, changed: &Value) {
        let slot = (self.0)
            .entry("monitors")
            .or_insert_with(|| Value::Array(Vec::new()));
        let (Value::Array(monitors), Some(changed)) = (slot, changed.as_array()) else {
            return;
        };
        for monitor in changed {
            monitors.retain(|m| m["id"] != monitor["id"]);
            let index = |m: &Value| m["index"].as_u64().unwrap_or(u64::MAX);
            // After the last monitor at or before its index, as a stable
            // sort by index would put it; kept by hand, as the generic sort
            // costs more code than the few monitors there are.
            let at = monitors.partition_point(|m| index(m) <= index(monitor));
            monitors.insert(at, monitor.clone());
        }
    }

    /// The state, with only the `topics` and the monitor `monitor` names
    /// when given. A session always follows `daemon`, so it is dropped
    /// here unless asked for.
    pub fn view(&self, topics: Option<&[String]>, monitor: Option<&MonitorRef>) -> Value {
        let mut state = self.0.clone();
        if let Some(topics) = topics {
            state.retain(|topic, _| topics.contains(topic));
        }
        let mut state = Value::Object(state);
        if let (Some(monitor), Some(Value::Array(monitors))) = (monitor, state.get_mut("monitors"))
        {
            monitors.retain(|m| monitor.matches(m));
        }
        state
    }

    /// The id of the monitor `monitor` names, if the state has it.
    pub fn monitor_id(&self, monitor: &MonitorRef) -> Option<String> {
        let monitors = self.0.get("monitors")?.as_array()?;
        let found = monitors.iter().find(|m| monitor.matches(m))?;
        found["id"].as_str().map(str::to_string)
    }
}
