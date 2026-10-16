use std::fs;
use std::path::{Path, PathBuf};

use mosaico_core::system::FileProbe;

use super::{Location, WindowsFiles, location};

/// A fresh directory for `test` under the system temp directory.
fn temp_dir(test: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("mosaico-files-tests")
        .join(format!("{test}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn a_path_is_placed_by_its_prefix() {
    // Arrange
    let cases = [
        (r"C:\apps\mosaico.exe", Location::Drive(b'C')),
        (r"d:\apps\mosaico.exe", Location::Drive(b'D')),
        (r"\\?\C:\apps\mosaico.exe", Location::Drive(b'C')),
        (r"\\server\share\mosaico.exe", Location::Share),
        (r"\\?\UNC\server\share\mosaico.exe", Location::Share),
        (r"\\.\pipe\mosaico", Location::Device),
        (
            r"\\?\Volume{00000000-0000-0000-0000-000000000000}\a.exe",
            Location::Local,
        ),
        (r"apps\mosaico.exe", Location::Local),
    ];

    for (path, expected) in cases {
        // Act
        let placed = location(Path::new(path));

        // Assert
        assert_eq!(placed, expected, "{path}");
    }
}

#[test]
fn a_path_on_a_share_is_answered_without_looking_at_it() {
    // Arrange: an address kept for documentation, which never answers.
    let share = Path::new(r"\\192.0.2.1\share\mosaico.exe");

    // Act
    let is_file = WindowsFiles.is_file(share);
    let same = WindowsFiles.same_file(share, Path::new(r"C:\apps\mosaico.exe"));

    // Assert
    assert_eq!(is_file, None);
    assert!(!same);
}

#[test]
fn a_file_a_directory_and_nothing_are_told_apart() {
    // Arrange
    let dir = temp_dir("kinds");
    let file = dir.join("mosaico.exe");
    fs::write(&file, "").unwrap();

    // Act / Assert
    assert_eq!(WindowsFiles.is_file(&file), Some(true));
    assert_eq!(WindowsFiles.is_file(&dir), Some(false));
    assert_eq!(WindowsFiles.is_file(&dir.join("gone.exe")), Some(false));
}

#[test]
fn other_spellings_of_a_path_name_the_same_file() {
    // Arrange
    let dir = temp_dir("spellings");
    let file = dir.join("mosaico.exe");
    fs::write(&file, "").unwrap();
    fs::create_dir(dir.join("bin")).unwrap();
    let spellings = [
        dir.join("bin").join("..").join("mosaico.exe"),
        PathBuf::from(file.to_string_lossy().to_uppercase()),
        PathBuf::from(format!(r"\\?\{}", file.display())),
    ];

    for spelling in spellings {
        // Act
        let same = WindowsFiles.same_file(&spelling, &file);

        // Assert
        assert!(same, "{spelling:?}");
    }
}

#[test]
fn different_files_hard_links_and_missing_files_are_not_the_same_file() {
    // Arrange
    let dir = temp_dir("different");
    let file = dir.join("mosaico.exe");
    let other = dir.join("other.exe");
    let hard_link = dir.join("hard-link.exe");
    fs::write(&file, "").unwrap();
    fs::write(&other, "").unwrap();
    fs::hard_link(&file, &hard_link).unwrap();

    // Act / Assert
    assert!(!WindowsFiles.same_file(&file, &other));
    assert!(!WindowsFiles.same_file(&file, &hard_link));
    assert!(!WindowsFiles.same_file(&file, &dir.join("gone.exe")));
}
