//! What a client prints of the daemon's messages.

use serde_json::{Value, json};

use super::{FieldPath, JsonState, MonitorRef, field_text};

/// What `subscribe` prints.
#[derive(Debug, Clone, PartialEq)]
pub enum Output {
    /// Every message, one per line, as the daemon sent it.
    Messages,
    /// The whole state of the topics followed, as
    /// `{"rev":59,"state":{...}}`, on subscribing and whenever it changes,
    /// then every other message but `welcome`.
    Full,
    /// One value of the state, as text, on subscribing and whenever it
    /// changes.
    Field(FieldPath),
}

/// Turns the daemon's messages into the lines a client prints, keeping
/// the state they carry.
pub(super) struct Printer {
    output: Output,
    monitor: Option<MonitorRef>,
    /// Whether answers to requests are printed whatever the output.
    replies: bool,
    /// The topics asked for, the only ones printed of the whole state.
    topics: Option<Vec<String>>,
    state: JsonState,
    /// The last state or value printed, so an unchanged one is not again.
    printed: Option<String>,
}

impl Printer {
    pub fn new(output: Output, monitor: Option<MonitorRef>, replies: bool) -> Self {
        Self {
            output,
            monitor,
            replies,
            topics: None,
            state: JsonState::default(),
            printed: None,
        }
    }

    /// Prints only `topics` of the whole state from now on, or every
    /// topic for `None`.
    pub fn follow(&mut self, topics: Option<Vec<String>>) {
        self.topics = topics;
    }

    /// The state as it stands.
    pub fn state(&self) -> &JsonState {
        &self.state
    }

    /// The session was lost: the next state is printed even if unchanged,
    /// since what came between said the daemon went away.
    pub fn lost(&mut self) {
        if self.output == Output::Full {
            self.printed = None;
        }
    }

    /// The line to print for `message`, if any.
    pub fn take(&mut self, message: &Value) -> Option<String> {
        let mut message = message.clone();
        if let Some(snapshot) = message.get_mut("snapshot") {
            self.state.reset(snapshot);
            self.keep_monitor(snapshot, &[]);
        } else if let Some(update) = message.get_mut("update") {
            let before = self.state.view(None, None);
            self.state.apply(update);
            self.keep_monitor(update, before["monitors"].as_array().map_or(&[], |m| m));
        } else {
            return self.other(message);
        }
        match &self.output {
            Output::Messages => Some(message.to_string()),
            Output::Full => {
                let state = (self.state).view(self.topics.as_deref(), self.monitor.as_ref());
                let line = json!({"rev": message["rev"], "state": state});
                self.changed(state.to_string()).then(|| line.to_string())
            }
            Output::Field(path) => {
                let state = self.state.view(None, self.monitor.as_ref());
                let text = field_text(path.lookup(&state));
                self.changed(text.clone()).then_some(text)
            }
        }
    }

    fn other(&self, message: Value) -> Option<String> {
        let print = match self.output {
            Output::Messages => true,
            Output::Full => message.get("welcome").is_none(),
            Output::Field(_) => self.replies && message.get("id").is_some(),
        };
        print.then(|| message.to_string())
    }

    /// Whether `printed` differs from what was printed last, which it
    /// becomes.
    fn changed(&mut self, printed: String) -> bool {
        let changed = self.printed.as_ref() != Some(&printed);
        self.printed = Some(printed);
        changed
    }

    /// Leaves in `carried`, a snapshot or an update, only the monitor
    /// named, if one is; `before` is the monitors before an update, for
    /// the ones it removes.
    fn keep_monitor(&self, carried: &mut Value, before: &[Value]) {
        let Some(monitor) = &self.monitor else {
            return;
        };
        if let Some(Value::Array(monitors)) = carried.get_mut("monitors") {
            monitors.retain(|m| monitor.matches(m));
        }
        let named = |id: &Value| before.iter().any(|m| m["id"] == *id && monitor.matches(m));
        if let Some(Value::Array(gone)) = carried.pointer_mut("/removed/monitors") {
            gone.retain(named);
        }
    }
}
