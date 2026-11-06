use std::path::{Path, PathBuf};

use super::*;

/// The mosaico executable of a package installed in `folder`, if one is.
struct Packaged {
    folder: Option<&'static str>,
}

impl Installer for Packaged {
    fn asset_name(&self) -> crate::Result<&str> {
        Ok("mosaico-test-os.zip")
    }

    fn binary_name(&self) -> &str {
        "mosaico-test"
    }

    fn settings_name(&self) -> &str {
        "settings-test"
    }

    fn replace(&self, _exe: &Path, _binary: &[u8]) -> crate::Result<()> {
        unreachable!("telling how mosaico was installed installs nothing")
    }

    fn remove_leftovers(&self, _exe: &Path) -> crate::Result<()> {
        unreachable!("telling how mosaico was installed installs nothing")
    }

    fn package_folder(&self) -> Option<PathBuf> {
        self.folder.map(PathBuf::from)
    }
}

/// Files where `alias` and `target` name the same file, as a link and
/// the file it points to do.
struct Linked {
    alias: &'static str,
    target: &'static str,
}

impl FileProbe for Linked {
    fn is_file(&self, _path: &Path) -> Option<bool> {
        Some(true)
    }

    fn same_file(&self, a: &Path, b: &Path) -> bool {
        let pair = |x: &Path, y: &Path| x == Path::new(self.alias) && y == Path::new(self.target);
        a == b || pair(a, b) || pair(b, a)
    }
}

/// No two paths name one file.
const NO_LINKS: Linked = Linked {
    alias: "",
    target: "",
};

#[test]
fn the_executable_in_the_package_folder_is_packaged() {
    // Arrange
    let installer = Packaged {
        folder: Some("/programs/mosaico"),
    };

    // Act
    let packaged = is_packaged(
        &installer,
        &NO_LINKS,
        Path::new("/programs/mosaico/mosaico-test"),
    );

    // Assert
    assert!(packaged);
}

#[test]
fn a_link_to_the_packaged_executable_is_packaged() {
    // Arrange
    let installer = Packaged {
        folder: Some("/programs/mosaico"),
    };
    let files = Linked {
        alias: "/short/mosaico-test",
        target: "/programs/mosaico/mosaico-test",
    };

    // Act
    let packaged = is_packaged(&installer, &files, Path::new("/short/mosaico-test"));

    // Assert
    assert!(packaged);
}

#[test]
fn a_copy_elsewhere_is_not_packaged_while_the_package_is_installed() {
    // Arrange
    let installer = Packaged {
        folder: Some("/programs/mosaico"),
    };

    // Act
    let packaged = is_packaged(
        &installer,
        &NO_LINKS,
        Path::new("/home/mosaico/mosaico-test"),
    );

    // Assert
    assert!(!packaged);
}

#[test]
fn nothing_is_packaged_without_a_package() {
    // Arrange
    let installer = Packaged { folder: None };

    // Act
    let packaged = is_packaged(
        &installer,
        &NO_LINKS,
        Path::new("/programs/mosaico/mosaico-test"),
    );

    // Assert
    assert!(!packaged);
}
