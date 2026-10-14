//! Temporary files for the store's tests.

use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use super::ConfigStore;

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Returns a unique temp path for each call so parallel tests do not
/// collide on the shared system temp directory.
pub(super) fn unique_temp_path() -> PathBuf {
    let mut p = std::env::temp_dir();
    let pid = std::process::id();
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    p.push(format!("mosaico-migration-test-{pid}-{n}.toml"));
    p
}

pub(super) fn cleanup(path: &Path) {
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(path.with_extension("toml.pre-0.9.bak"));
    for n in 1..10 {
        let _ = std::fs::remove_file(path.with_extension(format!("toml.pre-0.9.bak.{n}")));
    }
}

/// A store on a directory of its own, deleted with everything in it
/// when the store is dropped, so a test leaves nothing behind.
pub(super) struct TempStore(ConfigStore);

impl Deref for TempStore {
    type Target = ConfigStore;

    fn deref(&self) -> &ConfigStore {
        &self.0
    }
}

impl Drop for TempStore {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(self.0.dir());
    }
}

/// A store on a fresh, empty directory named after `test`.
pub(super) fn temp_store(test: &str) -> TempStore {
    let dir = std::env::temp_dir()
        .join("mosaico-config-store-tests")
        .join(format!("{test}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    TempStore(ConfigStore::new(dir))
}

/// The names of the files in the store's directory, sorted.
pub(super) fn file_names(store: &ConfigStore) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(store.dir())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

/// What the file at `path` holds.
pub(super) fn contents(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap()
}
