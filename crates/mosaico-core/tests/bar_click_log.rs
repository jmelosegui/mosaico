//! What the bar manager logs about the `[[mouse.click]]` entries it leaves
//! out: when it is made, and when it is given a new config.
//!
//! The logger is global and can only be set up once, so this file holds
//! a single test and gets a process of its own, where it starts empty.

use std::path::PathBuf;
use std::rc::Rc;

use mosaico_core::bar::{
    BarContent, BarId, BarManager, BarRenderer, BarSurface, CpuTimes, ItemRegion, LocalTime,
    NowPlaying, PointerEvent, PointerSink, SystemInfo,
};
use mosaico_core::config::{BarConfig, ClickBinding, Theme};
use mosaico_core::log::{self, LogConfig};
use mosaico_core::{Rect, Result};

/// Bars that draw nothing.
struct Blank;

impl BarRenderer for Blank {
    fn create(
        &mut self,
        _id: BarId,
        _area: Rect,
        _sink: Rc<dyn PointerSink>,
    ) -> Result<Box<dyn BarSurface>> {
        Ok(Box::new(Blank))
    }
}

impl BarSurface for Blank {
    fn draw(&mut self, _config: &BarConfig, _content: &BarContent) -> Result<Vec<ItemRegion>> {
        Ok(Vec::new())
    }

    fn hide(&mut self) -> Result<()> {
        Ok(())
    }
}

/// A machine with nothing to report.
struct Idle;

impl SystemInfo for Idle {
    fn cpu_times(&self) -> Result<CpuTimes> {
        Ok(CpuTimes::default())
    }

    fn memory_usage(&self) -> Result<u32> {
        Ok(0)
    }

    fn now_playing(&self) -> Result<Option<NowPlaying>> {
        Ok(None)
    }

    fn local_time(&self) -> LocalTime {
        LocalTime::default()
    }
}

/// A sink that drops what the bars report.
struct Deaf;

impl PointerSink for Deaf {
    fn send(&self, _bar: BarId, _event: PointerEvent) {}
}

/// The default bar with `entry` as its one `[[mouse.click]]` entry.
fn with(entry: ClickBinding) -> BarConfig {
    let mut config = BarConfig::default();
    config.mouse.click = vec![entry];
    config
}

/// A fresh directory for this test's log.
fn log_dir() -> PathBuf {
    let dir = std::env::temp_dir()
        .join("mosaico-log-tests")
        .join(format!("bar-clicks-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

#[test]
fn the_bar_manager_logs_each_entry_it_leaves_out_when_made_and_on_reload() {
    // Arrange
    let dir = log_dir();
    let warnings_on = LogConfig {
        enabled: true,
        level: "warn".into(),
        max_file_mb: 10,
    };
    log::init_in(&warnings_on, &dir);
    let made = with(ClickBinding::new("radio", "left", "retile"));
    let monitors = vec![Rect::new(0, 0, 1920, 1040)];

    // Act -- the last config removes a binding, which is no mistake.
    let mut manager = BarManager::new(
        Box::new(Blank),
        Box::new(Idle),
        Rc::new(Deaf),
        made,
        monitors,
        Theme::default(),
    );
    manager.reload(with(ClickBinding::new("clock", "left", "close-focused")));
    manager.reload(with(ClickBinding::new("clock", "left", ClickBinding::NONE)));

    // Assert
    let text = std::fs::read_to_string(dir.join("mosaico.log")).expect("no log was written");
    let warnings: Vec<&str> = text
        .lines()
        .filter(|line| line.contains("[WARN]"))
        .collect();
    let expected = [
        "[WARN] bar.toml: [[mouse.click]] with widget = \"radio\", button = \"left\" and \
         action = \"retile\" is ignored: no widget has that type",
        "[WARN] bar.toml: [[mouse.click]] with widget = \"clock\", button = \"left\" and \
         action = \"close-focused\" is ignored: the action is about the focused window",
    ];
    assert_eq!(warnings.len(), expected.len(), "{text}");
    for (warning, expected) in warnings.iter().zip(expected) {
        assert!(warning.contains(expected), "{warning}");
    }
}
