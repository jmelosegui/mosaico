//! Notepad windows on the live desktop, for the window transition and the
//! drag and drop tests, and where they are.
//!
//! Windows are found and read through the platform's own implementations
//! of the core contracts, so where a window is reads the same visible
//! bounds the daemon places windows at.

#![allow(dead_code)]

use std::process::{Child, Command};
use std::thread;
use std::time::{Duration, Instant};

use mosaico_core::tiler::{Enumerator, Platform};
use mosaico_core::{Rect, WindowId};
use mosaico_windows::enumerate::WindowsEnumerator;
use mosaico_windows::tiling::Win32Platform;

/// The windows of a notepad process smaller than this are not the ones
/// that get tiled, such as its tab strip.
const MIN_SIZE: i32 = 100;

/// How often a window is looked at while it is waited for.
const POLL: Duration = Duration::from_millis(50);

/// The notepads a test opened. Dropping them closes them the way a user
/// does, so notepad saves its session empty and brings none of them back
/// in the next test.
pub struct Notepads {
    platform: Win32Platform,
    windows: Vec<WindowId>,
    processes: Vec<Child>,
}

impl Notepads {
    /// No notepad yet. Notepads left open, by an earlier test or by hand,
    /// are closed first, so new ones open as windows of their own.
    pub fn new() -> Self {
        let platform = Win32Platform::new();
        close(&platform, &notepad_windows());
        Self {
            platform,
            windows: Vec::new(),
            processes: Vec::new(),
        }
    }

    /// Opens a notepad and returns its window once it is shown.
    pub fn open(&mut self) -> WindowId {
        let before = notepad_windows();
        let process = Command::new("notepad.exe").spawn();
        self.processes
            .push(process.expect("failed to launch notepad.exe"));
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let new = notepad_windows().into_iter().find(|w| !before.contains(w));
            if let Some(window) = new {
                self.windows.push(window);
                return window;
            }
            assert!(Instant::now() < deadline, "no notepad window within 10 s");
            thread::sleep(POLL);
        }
    }

    /// Where `window` is now, in visible bounds.
    pub fn rect(&self, window: WindowId) -> Rect {
        let rect = self.platform.desktop().rect(window);
        rect.unwrap_or_else(|| panic!("notepad window {window} is gone"))
    }

    /// Where `windows` are once they have stayed there for half a second,
    /// as they do once the tiler has placed them.
    pub fn still(&self, windows: [WindowId; 2]) -> [Rect; 2] {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut last = windows.map(|w| self.rect(w));
        let mut since = Instant::now();
        while since.elapsed() < Duration::from_millis(500) {
            assert!(Instant::now() < deadline, "notepads still moving: {last:?}");
            thread::sleep(POLL);
            let now = windows.map(|w| self.rect(w));
            if now != last {
                (last, since) = (now, Instant::now());
            }
        }
        last
    }

    /// Every rect `windows` are seen at for `span`, looked at every few
    /// milliseconds.
    pub fn sample(&self, windows: [WindowId; 2], span: Duration) -> [Vec<Rect>; 2] {
        let mut seen = [Vec::new(), Vec::new()];
        let end = Instant::now() + span;
        while Instant::now() < end {
            for (rects, window) in seen.iter_mut().zip(windows) {
                rects.push(self.rect(window));
            }
            thread::sleep(Duration::from_millis(3));
        }
        seen
    }
}

impl Drop for Notepads {
    fn drop(&mut self) {
        close(&self.platform, &self.windows);
        for process in &mut self.processes {
            let _ = process.kill();
            let _ = process.wait();
        }
    }
}

/// Asks each of `windows` to close, and waits up to five seconds for them
/// to go.
fn close(platform: &Win32Platform, windows: &[WindowId]) {
    for &window in windows {
        let _ = platform.control().close(window);
    }
    let deadline = Instant::now() + Duration::from_secs(5);
    let shown = |w: &WindowId| platform.desktop().is_visible(*w);
    while windows.iter().any(shown) && Instant::now() < deadline {
        thread::sleep(POLL);
    }
}

/// The notepad windows on the desktop that are big enough to be tiled.
fn notepad_windows() -> Vec<WindowId> {
    let windows = WindowsEnumerator.windows().unwrap_or_default();
    windows
        .into_iter()
        .filter(|&window| {
            let details = WindowsEnumerator.details(window);
            let big = |r: Rect| r.width >= MIN_SIZE && r.height >= MIN_SIZE;
            details.facts.class == "Notepad" && details.rect.is_some_and(big)
        })
        .collect()
}
