//! Lines logged before the logger is set up.
//!
//! The logger is global and can only be set up once, so this file holds
//! a single test and gets a process of its own, where it starts empty.

use std::path::PathBuf;
use std::sync::{Arc, Barrier};
use std::thread;

use mosaico_core::log::{self, LogConfig};
use mosaico_core::{log_debug, log_info, log_warn};

/// Threads that log while the logger is being set up.
const RACERS: usize = 4;

/// Lines each of them logs. With the three logged before, they stay
/// under the 100 lines the logger keeps until it is set up, so none of
/// them may go missing.
const LINES_PER_RACER: usize = 20;

/// A fresh directory for this test's log.
fn log_dir() -> PathBuf {
    let dir = std::env::temp_dir()
        .join("mosaico-log-tests")
        .join(format!("early-lines-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

/// Where the one line ending in `[level] message` is, failing unless
/// there is exactly one.
fn position(lines: &[&str], level: &str, message: &str) -> usize {
    let suffix = format!("[{level}] {message}");
    let found: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.ends_with(&suffix))
        .map(|(i, _)| i)
        .collect();
    assert_eq!(
        found.len(),
        1,
        "{suffix:?} should be in the log once:\n{}",
        lines.join("\n")
    );
    found[0]
}

#[test]
fn lines_logged_before_init_are_written_once_after_it_at_its_level() {
    // Arrange -- lines from before the logger exists, and threads that
    // keep logging while it is set up.
    let dir = log_dir();
    log_debug!("early debug");
    log_info!("early info");
    log_warn!("early warn");
    let go = Arc::new(Barrier::new(RACERS + 1));
    let racers: Vec<_> = (0..RACERS)
        .map(|racer| {
            let go = Arc::clone(&go);
            thread::spawn(move || {
                go.wait();
                for line in 0..LINES_PER_RACER {
                    log_info!("racer {racer} line {line}");
                }
            })
        })
        .collect();
    let config = LogConfig {
        enabled: true,
        level: "info".into(),
        max_file_mb: 10,
    };

    // Act
    go.wait();
    log::init_in(&config, &dir);
    for racer in racers {
        racer.join().expect("a racing thread panicked");
    }
    log_info!("late info");

    // Assert
    let text = std::fs::read_to_string(dir.join("mosaico.log")).expect("no log was written");
    let lines: Vec<&str> = text.lines().collect();

    // The first line of the logger, then what was logged before it, in
    // order, then what came after.
    let init = position(&lines, "INFO", "Logger initialized (level=info, max_mb=10)");
    let early_info = position(&lines, "INFO", "early info");
    let early_warn = position(&lines, "WARN", "early warn");
    let late = position(&lines, "INFO", "late info");
    assert!(init < early_info, "{text}");
    assert!(early_info < early_warn, "{text}");
    assert!(early_warn < late, "{text}");

    // Below the configured level, so it is left out.
    assert!(!text.contains("early debug"), "{text}");

    // Whether a racing line was kept for the logger or written straight
    // to the file, it is there, and only once.
    for racer in 0..RACERS {
        for line in 0..LINES_PER_RACER {
            position(&lines, "INFO", &format!("racer {racer} line {line}"));
        }
    }
}
