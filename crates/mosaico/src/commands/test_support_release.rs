//! GitHub and the platform's installer, faked, for the commands that
//! look up and install releases.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use mosaico_core::http::HttpClient;
use mosaico_core::update::Installer;
use mosaico_core::{Error, Result};

/// The file name of the release archive in the fake world.
pub const ASSET: &str = "mosaico-test-os.zip";

/// The file name of the executable in that archive.
pub const BINARY: &str = "mosaico-test";

/// GitHub in the fake world. It cannot be reached until a release is
/// published.
#[derive(Default)]
pub struct Releases {
    /// The tag of the latest release.
    latest: RefCell<Option<String>>,
    /// The files attached to the latest release, by name.
    assets: RefCell<HashMap<String, Vec<u8>>>,
}

impl Releases {
    /// Publishes a release tagged `tag` as the latest.
    pub fn publish(&self, tag: &str) {
        *self.latest.borrow_mut() = Some(tag.to_string());
    }

    /// Attaches a file named `name` to the latest release.
    pub fn attach(&self, name: &str, content: Vec<u8>) {
        self.assets.borrow_mut().insert(name.to_string(), content);
    }
}

impl HttpClient for Releases {
    fn get_bytes(&self, host: &str, path: &str, _timeout: Duration) -> Result<Vec<u8>> {
        let url = format!("https://{host}{path}");
        let Some(tag) = self.latest.borrow().clone() else {
            return Err(Error::Network(format!(
                "{url}: could not send the request: no network"
            )));
        };
        if path.ends_with("/releases/latest") {
            return Ok(format!(r#"{{"tag_name":"{tag}"}}"#).into_bytes());
        }
        let download = format!("/jmelosegui/mosaico/releases/download/{tag}/");
        if let Some(name) = path.strip_prefix(&download)
            && let Some(content) = self.assets.borrow().get(name)
        {
            return Ok(content.clone());
        }
        Err(Error::Network(format!(
            "{url} answered with HTTP status 404"
        )))
    }
}

/// The platform's installer in the fake world. It records what it would
/// install instead of writing a file.
#[derive(Default)]
pub struct FakeInstaller {
    /// The executable replaced, and the binary it was replaced with.
    pub replaced: RefCell<Option<(PathBuf, Vec<u8>)>>,
    /// Error replacing fails with, if any.
    pub error: RefCell<Option<String>>,
}

impl Installer for FakeInstaller {
    fn asset_name(&self) -> &str {
        ASSET
    }

    fn binary_name(&self) -> &str {
        BINARY
    }

    fn replace(&self, exe: &Path, binary: &[u8]) -> Result<()> {
        if let Some(error) = self.error.borrow().clone() {
            return Err(Error::platform("write the new executable", error));
        }
        *self.replaced.borrow_mut() = Some((exe.to_path_buf(), binary.to_vec()));
        Ok(())
    }

    fn remove_leftovers(&self, _exe: &Path) -> Result<()> {
        Ok(())
    }
}
