//! The parts of the state a session follows.

use super::{Snapshot, State};

/// A part of the state a bar can follow on its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Topic {
    /// The daemon: its version, the bar picked, pausing, the update notice
    /// and the workspace settings. Every session follows it.
    Daemon,
    /// The colors bars are drawn in.
    Theme,
    /// The monitors and their workspaces.
    Monitors,
    /// Where the user works.
    Focus,
}

impl Topic {
    /// Every topic, in the order they are listed on the wire.
    pub const ALL: [Topic; 4] = [Topic::Daemon, Topic::Theme, Topic::Monitors, Topic::Focus];

    /// The topic's name on the wire.
    pub fn name(self) -> &'static str {
        match self {
            Topic::Daemon => "daemon",
            Topic::Theme => "theme",
            Topic::Monitors => "monitors",
            Topic::Focus => "focus",
        }
    }

    /// The topic `name` names, if this build knows it.
    pub fn named(name: &str) -> Option<Topic> {
        Topic::ALL.into_iter().find(|topic| topic.name() == name)
    }
}

/// The topics a session follows, `daemon` always among them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Topics([bool; 4]);

impl Default for Topics {
    /// Every topic, what a session follows when its `hello` names none.
    fn default() -> Self {
        Self([true; 4])
    }
}

impl Topics {
    /// The topics `names` asks for, those this build knows, with `daemon`;
    /// every topic when `names` is `None`. Names it does not know are left
    /// out rather than refused, so a bar built for a later release still
    /// gets the topics this one has.
    pub fn requested(names: Option<&[String]>) -> Self {
        let Some(names) = names else {
            return Self::default();
        };
        let mut topics = Self([false; 4]);
        topics.0[Topic::Daemon as usize] = true;
        for topic in names.iter().filter_map(|name| Topic::named(name)) {
            topics.0[topic as usize] = true;
        }
        topics
    }

    /// Whether `topic` is followed.
    pub fn contains(self, topic: Topic) -> bool {
        self.0[topic as usize]
    }

    /// The names of the topics followed, in wire order.
    pub fn names(self) -> Vec<String> {
        (Topic::ALL.into_iter())
            .filter(|topic| self.contains(*topic))
            .map(|topic| topic.name().to_string())
            .collect()
    }

    /// `state` with the topics not followed left at their defaults, so two
    /// such states differ only in what is followed.
    pub fn filter(self, state: &State) -> State {
        State {
            daemon: self.pick(Topic::Daemon, &state.daemon),
            theme: self.pick(Topic::Theme, &state.theme),
            monitors: self.pick(Topic::Monitors, &state.monitors),
            focus: self.pick(Topic::Focus, &state.focus),
        }
    }

    /// The topics followed of `state`, in full.
    pub fn snapshot(self, state: &State) -> Snapshot {
        let pick = |topic| self.contains(topic);
        Snapshot {
            daemon: pick(Topic::Daemon).then(|| state.daemon.clone()),
            theme: pick(Topic::Theme).then(|| state.theme.clone()),
            monitors: pick(Topic::Monitors).then(|| state.monitors.clone()),
            focus: pick(Topic::Focus).then(|| state.focus.clone()),
        }
    }

    fn pick<T: Clone + Default>(self, topic: Topic, value: &T) -> T {
        if self.contains(topic) {
            value.clone()
        } else {
            T::default()
        }
    }
}
