//! Downloading a release and taking the executable out of its archive.

use std::io::{self, Cursor, Read};
use std::time::Duration;

use super::Installer;
use crate::Result;
use crate::http::HttpClient;

/// The host release files are downloaded from.
const DOWNLOAD_HOST: &str = "github.com";

/// How long each step of the download may take.
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(60);

/// Downloads the release tagged `tag` for the platform of `installer`,
/// and returns the mosaico executable from its archive.
///
/// The archive is a zip, which is what the release workflow builds for
/// every platform. The executable is the entry named
/// [`binary_name`](Installer::binary_name), at the top of the archive or
/// in a folder, in any letter case.
///
/// # Errors
///
/// Fails with [`Error::Network`](crate::Error::Network) when the archive
/// cannot be downloaded, for example because the release has none for
/// this platform, and with [`Error::Io`](crate::Error::Io) when it is
/// not a zip archive or does not hold the executable.
pub fn download_binary(
    http: &dyn HttpClient,
    installer: &dyn Installer,
    tag: &str,
) -> Result<Vec<u8>> {
    let path = format!(
        "/jmelosegui/mosaico/releases/download/{tag}/{}",
        installer.asset_name()
    );
    let archive = http.get_bytes(DOWNLOAD_HOST, &path, DOWNLOAD_TIMEOUT)?;
    extract(&archive, installer.binary_name())
}

/// Takes the file named `binary` out of the zip `archive`.
fn extract(archive: &[u8], binary: &str) -> Result<Vec<u8>> {
    let mut archive = zip::ZipArchive::new(Cursor::new(archive))
        .map_err(|e| invalid(format!("invalid zip archive: {e}")))?;

    let wanted = binary.to_lowercase();
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| invalid(format!("cannot read zip entry: {e}")))?;

        let name = file.name().to_lowercase();
        if name == wanted || name.ends_with(&format!("/{wanted}")) {
            let mut buf = Vec::with_capacity(file.size() as usize);
            file.read_to_end(&mut buf)
                .map_err(|e| invalid(format!("cannot extract {binary}: {e}")))?;
            return Ok(buf);
        }
    }

    let missing = format!("{binary} not found in zip archive");
    Err(io::Error::new(io::ErrorKind::NotFound, missing).into())
}

/// An archive that cannot be read, as an I/O error saying `message`.
fn invalid(message: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[cfg(test)]
#[path = "tests/download_tests.rs"]
mod tests;
