use std::path::PathBuf;

use super::*;

/// A folder holding empty files with these names, removed when dropped.
struct Folder(PathBuf);

impl Folder {
    fn with(name: &str, files: &[&str]) -> Self {
        let dir = std::env::temp_dir().join(format!("mosaico-launcher-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create the folder");
        for file in files {
            std::fs::write(dir.join(file), b"").expect("write a file");
        }
        Self(dir)
    }
}

impl Drop for Folder {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn env(pairs: Vec<(&'static str, OsString)>) -> impl Fn(&str) -> Option<OsString> {
    move |key| {
        pairs
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v.clone())
    }
}

fn nothing_registered(_: &str) -> bool {
    false
}

/// Whether `command` is found with `PATH` set to `dir` and nothing else.
fn found_on(dir: &Path, command: &str) -> bool {
    let var = env(vec![("PATH", dir.as_os_str().to_owned())]);
    let lookup = Lookup {
        var: &var,
        registered: &nothing_registered,
        home: None,
    };
    can_start(command, &lookup)
}

#[test]
fn a_name_on_path_is_found_with_or_without_its_extension() {
    // Arrange
    let folder = Folder::with("on-path", &["tool.exe", "script.cmd"]);

    // Act
    let found =
        ["tool.exe", "TOOL", "script", "tool.com", "missing"].map(|c| found_on(&folder.0, c));

    // Assert
    assert_eq!(found, [true, true, true, false, false]);
}

#[test]
fn pathext_decides_which_extensions_are_tried() {
    // Arrange
    let folder = Folder::with("pathext", &["tool.ps1"]);
    let var = env(vec![
        ("PATH", folder.0.as_os_str().to_owned()),
        ("PATHEXT", ".EXE;.PS1".into()),
    ]);
    let lookup = Lookup {
        var: &var,
        registered: &nothing_registered,
        home: None,
    };

    // Act
    let found = can_start("tool", &lookup);

    // Assert
    assert!(found);
}

#[test]
fn a_registered_name_is_found_off_path() {
    // Arrange
    let var = env(vec![]);
    let registered = |name: &str| name.eq_ignore_ascii_case("msedge.exe");
    let lookup = Lookup {
        var: &var,
        registered: &registered,
        home: None,
    };

    // Act
    let found = ["msedge.exe", "msedge", "chrome"].map(|c| can_start(c, &lookup));

    // Assert
    assert_eq!(found, [true, true, false]);
}

#[test]
fn paths_are_found_only_when_something_is_there() {
    // Arrange
    let folder = Folder::with("paths", &["app.exe"]);
    let var = env(vec![]);
    let lookup = Lookup {
        var: &var,
        registered: &nothing_registered,
        home: Some(&folder.0),
    };
    let full = folder.0.join("app.exe");
    let missing = folder.0.join("gone.exe");

    // Act
    let found = [
        full.to_str().unwrap_or_default(),
        missing.to_str().unwrap_or_default(),
        r".\app.exe",
        r"sub\app.exe",
    ]
    .map(|c| can_start(c, &lookup));

    // Assert
    assert_eq!(found, [true, false, true, false]);
}

#[test]
fn uris_are_left_to_their_handlers_and_drive_letters_are_not_uris() {
    // Arrange
    let var = env(vec![]);
    let lookup = Lookup {
        var: &var,
        registered: &nothing_registered,
        home: None,
    };

    // Act
    let found = [
        "ms-settings:display",
        r"shell:AppsFolder\Microsoft.WindowsCalculator_8wekyb3d8bbwe!App",
        "https://example.com",
        r"Z:\nowhere\app.exe",
        "",
    ]
    .map(|c| can_start(c, &lookup));

    // Assert
    assert_eq!(found, [true, true, true, false, false]);
}

#[test]
fn edge_is_registered_on_a_machine_that_has_it() {
    // Arrange
    let edge = Path::new(r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe");
    if !edge.exists() {
        return;
    }

    // Act
    let found = registered("msedge.exe");

    // Assert
    assert!(found);
}
