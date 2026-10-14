//! An [`HttpClient`] that answers from a table, for tests.

use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Duration;

use super::{HttpClient, url};
use crate::{Error, Result};

/// What the fake server answers for one URL.
enum Answer {
    /// A successful answer with this body.
    Body(Vec<u8>),
    /// An answer with this status outside 200-299.
    Status(u16),
}

/// Answers each URL from a table and records the requests. A URL with
/// no entry is on a host that cannot be reached.
#[derive(Default)]
pub(crate) struct FakeHttp {
    answers: HashMap<String, Answer>,
    requests: RefCell<Vec<(String, Duration)>>,
}

impl FakeHttp {
    /// Answers `path` on `host` with `body`.
    pub(crate) fn answer(mut self, host: &str, path: &str, body: impl Into<Vec<u8>>) -> Self {
        self.answers
            .insert(url(host, path), Answer::Body(body.into()));
        self
    }

    /// Answers `path` on `host` with `status`, such as 404.
    pub(crate) fn status(mut self, host: &str, path: &str, status: u16) -> Self {
        self.answers.insert(url(host, path), Answer::Status(status));
        self
    }

    /// The requests made so far, as the URL and the timeout.
    pub(crate) fn requests(&self) -> Vec<(String, Duration)> {
        self.requests.borrow().clone()
    }
}

impl HttpClient for FakeHttp {
    fn get_bytes(&self, host: &str, path: &str, timeout: Duration) -> Result<Vec<u8>> {
        let url = url(host, path);
        self.requests.borrow_mut().push((url.clone(), timeout));
        match self.answers.get(&url) {
            Some(Answer::Body(body)) => Ok(body.clone()),
            Some(Answer::Status(status)) => Err(Error::http_status(&url, *status)),
            None => Err(Error::http_request(
                &url,
                "send the request",
                format!("{host} cannot be reached"),
            )),
        }
    }
}
