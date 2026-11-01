//! Downloading a release and taking the executables out of its archive.

use std::io::{self, Cursor, Read};
use std::time::Duration;

use super::Installer;
use crate::Result;
use crate::http::HttpClient;

/// The host release files are downloaded from.
const DOWNLOAD_HOST: &str = "github.com";

/// How long each step of the download may take.
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(60);

/// The executables of a release, as their contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binaries {
    /// The mosaico executable, [`binary_name`](Installer::binary_name).
    pub mosaico: Vec<u8>,
    /// The settings app, [`settings_name`](Installer::settings_name).
    pub settings: Vec<u8>,
}

/// Downloads the release tagged `tag` for the platform of `installer`,
/// and returns the executables from its archive.
///
/// The archive is a zip, which is what the release workflow builds for
/// every platform. Each executable is the entry with its name, at the top
/// of the archive or in a folder, in any letter case.
///
/// # Errors
///
/// Fails with [`Error::Unsupported`](crate::Error::Unsupported), before
/// downloading anything, when the platform cannot update itself, with
/// [`Error::Network`](crate::Error::Network) when the archive cannot be
/// downloaded, for example because the release has none for this
/// platform, and with [`Error::Io`](crate::Error::Io) when it is not a
/// zip archive or lacks either executable.
pub fn download_binaries(
    http: &dyn HttpClient,
    installer: &dyn Installer,
    tag: &str,
) -> Result<Binaries> {
    let asset = installer.asset_name()?;
    let path = format!("/jmelosegui/mosaico/releases/download/{tag}/{asset}");
    let archive = http.get_bytes(DOWNLOAD_HOST, &path, DOWNLOAD_TIMEOUT)?;
    Ok(Binaries {
        mosaico: extract(&archive, installer.binary_name())?,
        settings: extract(&archive, installer.settings_name())?,
    })
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
            // The size is what the archive says, which a corrupt one can
            // get wrong by any amount, so it only checks what was read.
            let declared = file.size();
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)
                .map_err(|e| invalid(format!("cannot extract {binary}: {e}")))?;
            if buf.len() as u64 != declared {
                return Err(invalid(format!(
                    "invalid zip archive: {binary} is said to be {declared} bytes but holds {}",
                    buf.len()
                ))
                .into());
            }
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
