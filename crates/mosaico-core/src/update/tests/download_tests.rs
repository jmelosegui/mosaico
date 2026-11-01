use std::io::{Cursor, Write};
use std::path::Path;
use std::time::Duration;

use zip::write::SimpleFileOptions;

use crate::Error;
use crate::http::fake::FakeHttp;

use super::*;
use TestPlatform::{CannotUpdate, Updates};

/// Where the archive of release `v1.2.3` is downloaded from.
const ARCHIVE: &str = "/jmelosegui/mosaico/releases/download/v1.2.3/mosaico-test-os.zip";

/// A platform whose release archive is `mosaico-test-os.zip`, holding
/// the executables `mosaico-test` and `settings-test`, or one that cannot
/// update itself.
enum TestPlatform {
    Updates,
    CannotUpdate,
}

impl Installer for TestPlatform {
    fn asset_name(&self) -> crate::Result<&str> {
        match self {
            Updates => Ok("mosaico-test-os.zip"),
            CannotUpdate => Err(Error::Unsupported("self-update")),
        }
    }

    fn binary_name(&self) -> &str {
        "mosaico-test"
    }

    fn settings_name(&self) -> &str {
        "settings-test"
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

/// A zip archive holding `content` as `name`, whose central directory
/// says the file is `declared` bytes long.
fn zip_declaring(declared: u64, name: &str, content: &[u8]) -> Vec<u8> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .large_file(true);
    zip.start_file(name, options).unwrap();
    zip.write_all(content).unwrap();
    let mut bytes = zip.finish().unwrap().into_inner();

    // In the central directory entry, the zip64 field that comes after
    // the 46-byte header and the name starts, past its id and length,
    // with the size of the file.
    let entry = bytes.windows(4).rposition(|w| w == b"PK\x01\x02").unwrap();
    let size = entry + 46 + name.len() + 4;
    bytes[size..size + 8].copy_from_slice(&declared.to_le_bytes());
    bytes
}

/// GitHub serving `archive` as this platform's archive of `v1.2.3`.
fn serving(archive: Vec<u8>) -> FakeHttp {
    FakeHttp::default().answer("github.com", ARCHIVE, archive)
}

#[test]
fn the_binaries_come_from_the_archive_of_this_platform() {
    // Arrange
    let http = serving(zip_of(&[
        ("README.md", b"read me"),
        ("mosaico-test", b"binary"),
        ("settings-test", b"settings"),
    ]));

    // Act
    let binaries = download_binaries(&http, &Updates, "v1.2.3").unwrap();

    // Assert
    assert_eq!(binaries.mosaico, b"binary");
    assert_eq!(binaries.settings, b"settings");
    assert_eq!(
        http.requests(),
        [(
            format!("https://github.com{ARCHIVE}"),
            Duration::from_secs(60)
        )]
    );
}

#[test]
fn the_binaries_are_found_in_a_folder_and_in_any_letter_case() {
    // Arrange
    let http = serving(zip_of(&[
        ("mosaico-v1.2.3/MOSAICO-TEST", b"binary"),
        ("mosaico-v1.2.3/Settings-Test", b"settings"),
    ]));

    // Act
    let binaries = download_binaries(&http, &Updates, "v1.2.3").unwrap();

    // Assert
    assert_eq!(binaries.mosaico, b"binary");
    assert_eq!(binaries.settings, b"settings");
}

#[test]
fn a_file_whose_name_only_ends_like_the_binary_is_not_it() {
    // Arrange
    let http = serving(zip_of(&[("not-mosaico-test", b"other")]));

    // Act
    let error = download_binaries(&http, &Updates, "v1.2.3").unwrap_err();

    // Assert
    assert_eq!(error.to_string(), "mosaico-test not found in zip archive");
}

#[test]
fn a_release_without_this_platforms_archive_is_a_network_error() {
    // Arrange
    let http = FakeHttp::default().status("github.com", ARCHIVE, 404);

    // Act
    let error = download_binaries(&http, &Updates, "v1.2.3").unwrap_err();

    // Assert
    assert!(matches!(error, Error::Network(_)), "{error:?}");
    assert!(error.to_string().ends_with("HTTP status 404"), "{error}");
}

#[test]
fn an_unreachable_host_is_a_network_error() {
    // Arrange
    let http = FakeHttp::default();

    // Act
    let error = download_binaries(&http, &Updates, "v1.2.3").unwrap_err();

    // Assert
    assert!(matches!(error, Error::Network(_)), "{error:?}");
}

#[test]
fn an_archive_without_the_binary_is_an_error() {
    // Arrange
    let http = serving(zip_of(&[("README.md", b"read me")]));

    // Act
    let error = download_binaries(&http, &Updates, "v1.2.3").unwrap_err();

    // Assert
    assert!(matches!(error, Error::Io(_)), "{error:?}");
    assert_eq!(error.to_string(), "mosaico-test not found in zip archive");
}

#[test]
fn an_archive_without_the_settings_app_is_an_error() {
    // Arrange -- as archives were before the settings app shipped.
    let http = serving(zip_of(&[("mosaico-test", b"binary")]));

    // Act
    let error = download_binaries(&http, &Updates, "v1.2.3").unwrap_err();

    // Assert
    assert!(matches!(error, Error::Io(_)), "{error:?}");
    assert_eq!(error.to_string(), "settings-test not found in zip archive");
}

#[test]
fn a_download_that_is_not_an_archive_is_an_error() {
    // Arrange -- an error page served as if it were the file.
    let http = serving(b"Not Found".to_vec());

    // Act
    let error = download_binaries(&http, &Updates, "v1.2.3").unwrap_err();

    // Assert
    assert!(matches!(error, Error::Io(_)), "{error:?}");
    assert!(
        error.to_string().starts_with("invalid zip archive: "),
        "{error}"
    );
}

#[test]
fn an_archive_that_declares_a_size_it_does_not_hold_is_invalid() {
    // Arrange -- the archive says the binary is 2^63 bytes; it holds 6.
    let http = serving(zip_declaring(1 << 63, "mosaico-test", b"binary"));

    // Act
    let error = download_binaries(&http, &Updates, "v1.2.3").unwrap_err();

    // Assert
    assert!(matches!(error, Error::Io(_)), "{error:?}");
    assert!(
        error.to_string().starts_with("invalid zip archive: "),
        "{error}"
    );
}

#[test]
fn a_platform_that_cannot_update_itself_downloads_nothing() {
    // Arrange
    let http = serving(zip_of(&[("mosaico-test", b"binary")]));

    // Act
    let error = download_binaries(&http, &CannotUpdate, "v1.2.3").unwrap_err();

    // Assert
    assert!(matches!(error, Error::Unsupported(_)), "{error:?}");
    assert!(http.requests().is_empty(), "{:?}", http.requests());
}
