//! Starting tiling managers on a fake platform, with transitions off or
//! on, letting their windows settle, and the files they keep their record
//! of hidden windows in.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::WindowEvent;
use crate::config::Config;

use super::super::{HiddenFile, Platform, TilingManager};
use super::fake_platform::FakePlatform;

/// Starts a tiling manager on `platform` with `config` and no window
/// rules, keeping its record of hidden windows in `hidden`.
pub(in crate::tiler) fn start(
    platform: &FakePlatform,
    hidden: Option<HiddenFile>,
    config: &Config,
) -> TilingManager<FakePlatform> {
    TilingManager::new(platform.clone(), hidden, config, Vec::new()).expect("the manager starts")
}

/// The default config with transitions turned off, so that every
/// placement is made at once, as the tests of the tiling pipeline
/// itself expect.
pub(in crate::tiler) fn still() -> Config {
    let mut config = Config::default();
    config.animations.enabled = false;
    config
}

/// The default config with transitions on, so that the placements of a
/// step that may animate do.
pub(in crate::tiler) fn animated() -> Config {
    let mut config = Config::default();
    config.animations.enabled = true;
    config
}

/// Takes frames as the daemon does until no window moves, a display
/// refresh apart. Then the applications get to what is queued for them,
/// and the platform reports the windows that moved, as Windows does. A
/// window still to land after that is waited for, until the frame step
/// that ends the wait by time. With transitions off, there is nothing to
/// wait for.
pub(in crate::tiler) fn settle(platform: &FakePlatform, manager: &mut TilingManager<FakePlatform>) {
    for _ in 0..64 {
        if manager.wants_frames() {
            platform.advance(Duration::from_millis(16));
        } else {
            for window in platform.catch_up_prompt() {
                manager.handle_event(&WindowEvent::LocationChanged { window });
            }
            let Some(due) = manager.frame_due(Some(platform.now())) else {
                return;
            };
            platform.advance(due.saturating_duration_since(platform.now()));
        }
        manager.frame();
    }
    panic!("windows still moving after 64 frame steps");
}

/// Windows the manager tiles on the active workspace of monitor `mon`.
pub(in crate::tiler) fn tiled(manager: &TilingManager<FakePlatform>, mon: usize) -> Vec<u64> {
    manager.tiler().monitors()[mon]
        .active()
        .windows()
        .iter()
        .map(|w| w.raw())
        .collect()
}

/// A record of hidden windows in a directory of a test's own, removed
/// with the directory when the test ends.
pub(super) struct TempRecord {
    dir: PathBuf,
    path: PathBuf,
}

impl TempRecord {
    /// A record for `test`, with nothing in it yet.
    pub(super) fn new(test: &str) -> Self {
        let dir = std::env::temp_dir()
            .join("mosaico-manager-tests")
            .join(format!("{test}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let path = dir.join("hidden-windows.json");
        Self { dir, path }
    }

    /// Where the record is kept.
    pub(super) fn path(&self) -> &Path {
        &self.path
    }

    /// The record, for a manager to keep or for a test to read.
    pub(super) fn file(&self) -> HiddenFile {
        HiddenFile::new(&self.path)
    }
}

impl Drop for TempRecord {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
        // The parent is shared with the other tests, which run at the
        // same time, so it is left in place.
    }
}
