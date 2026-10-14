//! Starting tiling managers on a fake platform, and the files they keep
//! their record of hidden windows in.

use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;

use super::super::{HiddenFile, TilingManager};
use super::fake_platform::FakePlatform;

/// Starts a tiling manager on `platform` with `config` and no window
/// rules, keeping its record of hidden windows in `hidden`.
pub(super) fn start(
    platform: &FakePlatform,
    hidden: Option<HiddenFile>,
    config: &Config,
) -> TilingManager<FakePlatform> {
    TilingManager::new(platform.clone(), hidden, config, Vec::new()).expect("the manager starts")
}

/// Windows the manager tiles on the active workspace of monitor `mon`.
pub(super) fn tiled(manager: &TilingManager<FakePlatform>, mon: usize) -> Vec<u64> {
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
