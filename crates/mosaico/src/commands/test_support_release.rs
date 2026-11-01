//! GitHub and the platform's installer, faked, for the commands that
//! look up and install releases.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use mosaico_core::http::{self, HttpClient};
use mosaico_core::update::Installer;
use mosaico_core::{Error, Result};

/// The file name of the release archive in the fake world.
pub const ASSET: &str = "mosaico-test-os.zip";

/// The file name of the mosaico executable in that archive.
pub const BINARY: &str = "mosaico-test";

/// The file name of the settings app in that archive and beside the
/// mosaico executable.
pub const SETTINGS: &str = "mosaico-test-settings";

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
        let url = http::url(host, path);
        let Some(tag) = self.latest.borrow().clone() else {
            return Err(Error::http_request(&url, "send the request", "no network"));
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
        Err(Error::http_status(&url, 404))
    }
}

/// The platform's installer in the fake world. It records what it would
/// install instead of writing a file.
#[derive(Default)]
pub struct FakeInstaller {
    /// The executables replaced, in order, and what they were replaced
    /// with.
    pub replaced: RefCell<Vec<(PathBuf, Vec<u8>)>>,
    /// Error replacing fails with, if any, and the file name it fails
    /// on; every file when that is `None`.
    pub error: RefCell<Option<(String, Option<&'static str>)>>,
    /// Whether this platform cannot update itself.
    pub unsupported: Cell<bool>,
}

impl FakeInstaller {
    /// Fails as a platform that cannot update itself does, if it is one.
    fn supported(&self) -> Result<()> {
        if self.unsupported.get() {
            return Err(Error::Unsupported("self-update"));
        }
        Ok(())
    }
}

impl Installer for FakeInstaller {
    fn asset_name(&self) -> Result<&str> {
        self.supported()?;
        Ok(ASSET)
    }

    fn binary_name(&self) -> &str {
        BINARY
    }

    fn settings_name(&self) -> &str {
        SETTINGS
    }

    fn replace(&self, exe: &Path, binary: &[u8]) -> Result<()> {
        self.supported()?;
        if let Some((error, on)) = self.error.borrow().clone()
            && on.is_none_or(|name| exe.file_name() == Some(name.as_ref()))
        {
            return Err(Error::platform("write the new executable", error));
        }
        self.replaced
            .borrow_mut()
            .push((exe.to_path_buf(), binary.to_vec()));
        Ok(())
    }

    fn remove_leftovers(&self, _exe: &Path) -> Result<()> {
        Ok(())
    }
}
