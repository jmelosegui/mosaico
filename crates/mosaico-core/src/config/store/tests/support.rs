//! Temporary files for the store's tests.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

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
