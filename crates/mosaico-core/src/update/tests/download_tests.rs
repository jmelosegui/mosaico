use std::io::{Cursor, Write};
use std::path::Path;
use std::time::Duration;

use zip::write::SimpleFileOptions;

use crate::Error;
use crate::http::fake::FakeHttp;

use super::*;

/// Where the archive of release `v1.2.3` is downloaded from.
const ARCHIVE: &str = "/jmelosegui/mosaico/releases/download/v1.2.3/mosaico-test-os.zip";

/// A platform whose release archive is `mosaico-test-os.zip`, holding
/// the executable `mosaico-test`.
struct TestPlatform;

impl Installer for TestPlatform {
    fn asset_name(&self) -> &str {
        "mosaico-test-os.zip"
    }

    fn binary_name(&self) -> &str {
        "mosaico-test"
    }

    fn replace(&self, _exe: &Path, _binary: &[u8]) -> crate::Result<()> {
        unreachable!("downloading installs nothing")
    }

    fn remove_leftovers(&self, _exe: &Path) -> crate::Result<()> {
        unreachable!("downloading installs nothing")
    }
}

/// A zip archive holding `files`, as names and contents.
fn zip_of(files: &[(&str, &[u8])]) -> Vec<u8> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, content) in files {
        zip.start_file(*name, SimpleFileOptions::default()).unwrap();
        zip.write_all(content).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

/// GitHub serving `archive` as this platform's archive of `v1.2.3`.
fn serving(archive: Vec<u8>) -> FakeHttp {
    FakeHttp::default().answer("github.com", ARCHIVE, archive)
}

#[test]
fn the_binary_comes_from_the_archive_of_this_platform() {
    // Arrange
    let http = serving(zip_of(&[
        ("README.md", b"read me"),
        ("mosaico-test", b"binary"),
    ]));

    // Act
    let binary = download_binary(&http, &TestPlatform, "v1.2.3").unwrap();

    // Assert
    assert_eq!(binary, b"binary");
    assert_eq!(
        http.requests(),
        [(
            format!("https://github.com{ARCHIVE}"),
            Duration::from_secs(60)
        )]
    );
}

#[test]
fn the_binary_is_found_in_a_folder_and_in_any_letter_case() {
    // Arrange
    let http = serving(zip_of(&[("mosaico-v1.2.3/MOSAICO-TEST", b"binary")]));

    // Act
    let binary = download_binary(&http, &TestPlatform, "v1.2.3").unwrap();

    // Assert
    assert_eq!(binary, b"binary");
}

#[test]
fn a_file_whose_name_only_ends_like_the_binary_is_not_it() {
    // Arrange
    let http = serving(zip_of(&[("not-mosaico-test", b"other")]));

    // Act
    let error = download_binary(&http, &TestPlatform, "v1.2.3").unwrap_err();

    // Assert
    assert_eq!(error.to_string(), "mosaico-test not found in zip archive");
}

#[test]
fn a_release_without_this_platforms_archive_is_a_network_error() {
    // Arrange
    let http = FakeHttp::default().status("github.com", ARCHIVE, 404);

    // Act
    let error = download_binary(&http, &TestPlatform, "v1.2.3").unwrap_err();

    // Assert
    assert!(matches!(error, Error::Network(_)), "{error:?}");
    assert!(error.to_string().ends_with("HTTP status 404"), "{error}");
}

#[test]
fn an_unreachable_host_is_a_network_error() {
    // Arrange
    let http = FakeHttp::default();

    // Act
    let error = download_binary(&http, &TestPlatform, "v1.2.3").unwrap_err();

    // Assert
    assert!(matches!(error, Error::Network(_)), "{error:?}");
}

#[test]
fn an_archive_without_the_binary_is_an_error() {
    // Arrange
    let http = serving(zip_of(&[("README.md", b"read me")]));

    // Act
    let error = download_binary(&http, &TestPlatform, "v1.2.3").unwrap_err();

    // Assert
    assert!(matches!(error, Error::Io(_)), "{error:?}");
    assert_eq!(error.to_string(), "mosaico-test not found in zip archive");
}

#[test]
fn a_download_that_is_not_an_archive_is_an_error() {
    // Arrange -- an error page served as if it were the file.
    let http = serving(b"Not Found".to_vec());

    // Act
    let error = download_binary(&http, &TestPlatform, "v1.2.3").unwrap_err();

    // Assert
    assert!(matches!(error, Error::Io(_)), "{error:?}");
    assert!(
        error.to_string().starts_with("invalid zip archive: "),
        "{error}"
    );
}
