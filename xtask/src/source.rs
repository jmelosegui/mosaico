//! Reading the source files of the workspace crates.

use std::fs;
use std::io;
use std::path::Path;

/// A Rust source file of a workspace crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    /// The directory name of the crate under `crates/`.
    pub crate_name: String,
    /// The path within the crate, with `/` separators.
    pub path: String,
    /// The file's text.
    pub text: String,
}

/// Every `.rs` file of every crate under `root/crates`, sorted by crate
/// and path so counts do not depend on directory order.
pub fn read_crates(root: &Path) -> io::Result<Vec<SourceFile>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(root.join("crates"))? {
        let dir = entry?.path();
        if !dir.is_dir() {
            continue;
        }
        let crate_name = dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        collect(&dir, &dir, &crate_name, &mut files)?;
    }
    files.sort_by(|a, b| (&a.crate_name, &a.path).cmp(&(&b.crate_name, &b.path)));
    Ok(files)
}

fn collect(base: &Path, dir: &Path, crate_name: &str, out: &mut Vec<SourceFile>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n != "target") {
                collect(base, &path, crate_name, out)?;
            }
        } else if path.extension().is_some_and(|e| e == "rs") {
            let relative = path.strip_prefix(base).unwrap_or(&path);
            out.push(SourceFile {
                crate_name: crate_name.to_owned(),
                path: relative.to_string_lossy().replace('\\', "/"),
                text: fs::read_to_string(&path)?,
            });
        }
    }
    Ok(())
}
