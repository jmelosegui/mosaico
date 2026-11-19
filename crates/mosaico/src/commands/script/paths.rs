//! Where scripts live: `scripts\` for the scripts, `lib\` for the modules
//! they `require`, both in the config folder.

use std::path::{Path, PathBuf};

/// The folders of an instance's scripts.
pub struct Folders {
    /// The config folder, which holds the other two.
    pub root: PathBuf,
    /// The scripts `mosaico script run` finds by name.
    pub scripts: PathBuf,
    /// The modules scripts `require`.
    pub lib: PathBuf,
}

impl Folders {
    /// The folders inside the config folder `root`.
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            scripts: root.join("scripts"),
            lib: root.join("lib"),
        }
    }

    /// The script `target` names: a path when it has a folder in it or
    /// ends in `.lua`, otherwise a script in `scripts\` by its name.
    pub fn resolve(&self, target: &str) -> PathBuf {
        let is_path = target.contains(['/', '\\']) || target.ends_with(".lua");
        if is_path {
            PathBuf::from(target)
        } else {
            self.script(target)
        }
    }

    /// The file of the script called `name` in `scripts\`.
    pub fn script(&self, name: &str) -> PathBuf {
        self.scripts.join(format!("{name}.lua"))
    }
}

/// Why `name` cannot name a script, if it cannot: names are file names
/// without the extension, made of letters, digits, `-` and `_`.
pub fn invalid_name(name: &str) -> Option<String> {
    let fine = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    (!fine).then(|| {
        format!(
            "`{name}` is not a script name; use letters, digits, `-` and `_`, as in meeting-mode"
        )
    })
}

/// The name of the script at `path`: its file name without `.lua`.
pub fn name_of(path: &Path) -> String {
    path.file_stem()
        .map_or_else(|| "script".into(), |s| s.to_string_lossy().into_owned())
}
