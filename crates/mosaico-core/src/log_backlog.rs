//! Lines logged before the logger is set up.

use super::Level;

/// Most lines kept. A process that never sets up the logger, such as
/// the CLI, must not collect them without bound.
const LIMIT: usize = 100;

/// Lines logged before [`init`](super::init), with their level, so that
/// it can write the ones its level lets through.
pub(super) struct Backlog {
    lines: Vec<(Level, String)>,
}

impl Backlog {
    /// An empty backlog.
    pub(super) const fn new() -> Self {
        Self { lines: Vec::new() }
    }

    /// Keeps the line `line` formats, unless the backlog is full, in
    /// which case it is not formatted at all.
    pub(super) fn keep(&mut self, level: Level, line: impl FnOnce() -> String) {
        if self.lines.len() < LIMIT {
            self.lines.push((level, line()));
        }
    }

    /// Hands over the lines at or above `min_level`, oldest first, and
    /// empties the backlog.
    pub(super) fn take(&mut self, min_level: Level) -> Vec<String> {
        std::mem::take(&mut self.lines)
            .into_iter()
            .filter(|(level, _)| *level >= min_level)
            .map(|(_, line)| line)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_below_the_configured_level_are_left_out() {
        // Arrange
        let mut backlog = Backlog::new();
        backlog.keep(Level::Debug, || "debug\n".into());
        backlog.keep(Level::Warn, || "warn\n".into());
        backlog.keep(Level::Info, || "info\n".into());

        // Act
        let lines = backlog.take(Level::Info);

        // Assert -- in the order they were logged.
        assert_eq!(lines, ["warn\n", "info\n"]);
        assert!(backlog.take(Level::Debug).is_empty(), "taking empties it");
    }

    #[test]
    fn a_full_backlog_keeps_the_oldest_lines() {
        // Arrange
        let mut backlog = Backlog::new();

        // Act
        for n in 0..LIMIT + 5 {
            backlog.keep(Level::Info, || format!("line {n}\n"));
        }
        let lines = backlog.take(Level::Debug);

        // Assert
        assert_eq!(lines.len(), LIMIT);
        assert_eq!(lines[0], "line 0\n");
        assert_eq!(lines[LIMIT - 1], format!("line {}\n", LIMIT - 1));
    }

    #[test]
    fn a_full_backlog_does_not_format_the_line() {
        // Arrange
        let mut backlog = Backlog::new();
        for _ in 0..LIMIT {
            backlog.keep(Level::Debug, String::new);
        }

        // Act / Assert
        backlog.keep(Level::Debug, || panic!("formatted a line it drops"));
    }
}
