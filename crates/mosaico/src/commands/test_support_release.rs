//! GitHub, faked, for the commands that look up releases.

use std::cell::RefCell;
use std::time::Duration;

use mosaico_core::http::HttpClient;
use mosaico_core::{Error, Result};

/// GitHub in the fake world. It cannot be reached until a release is
/// published.
#[derive(Default)]
pub struct Releases {
    /// The tag of the latest release.
    latest: RefCell<Option<String>>,
}

impl Releases {
    /// Publishes a release tagged `tag` as the latest.
    pub fn publish(&self, tag: &str) {
        *self.latest.borrow_mut() = Some(tag.to_string());
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
        Err(Error::Network(format!(
            "{url} answered with HTTP status 404"
        )))
    }
}
