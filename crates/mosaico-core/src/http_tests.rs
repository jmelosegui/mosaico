use std::time::Duration;

use crate::Error;

use super::fake::FakeHttp;
use super::{HttpClient, url};

const TIMEOUT: Duration = Duration::from_secs(5);

#[test]
fn a_request_is_named_by_its_https_url() {
    // Act / Assert
    assert_eq!(
        url("example.org", "/a/b.zip"),
        "https://example.org/a/b.zip"
    );
}

#[test]
fn a_status_outside_200_to_299_names_the_url_and_the_status() {
    // Act
    let error = Error::http_status("https://example.org/a.zip", 404);

    // Assert
    assert!(matches!(error, Error::Network(_)), "{error:?}");
    assert_eq!(
        error.to_string(),
        "network request failed: https://example.org/a.zip answered with HTTP status 404"
    );
}

#[test]
fn a_failed_step_names_the_url_the_step_and_the_cause() {
    // Act
    let error = Error::http_request("https://example.org/a.zip", "send the request", "timed out");

    // Assert
    assert!(matches!(error, Error::Network(_)), "{error:?}");
    assert_eq!(
        error.to_string(),
        "network request failed: https://example.org/a.zip: could not send the request: timed out"
    );
}

#[test]
fn text_is_the_body_read_as_utf8() {
    // Arrange
    let http = FakeHttp::default().answer("example.org", "/notes", "caf\u{e9}");

    // Act
    let text = http.get_text("example.org", "/notes", TIMEOUT).unwrap();

    // Assert
    assert_eq!(text, "caf\u{e9}");
    assert_eq!(
        http.requests(),
        [("https://example.org/notes".to_string(), TIMEOUT)]
    );
}

#[test]
fn a_body_that_is_not_text_is_a_network_error() {
    // Arrange
    let http = FakeHttp::default().answer("example.org", "/logo", [0xff, 0xfe, 0x00]);

    // Act
    let error = http.get_text("example.org", "/logo", TIMEOUT).unwrap_err();

    // Assert
    assert!(matches!(error, Error::Network(_)), "{error:?}");
    assert!(
        error.to_string().contains("https://example.org/logo"),
        "{error}"
    );
}

#[test]
fn a_failed_request_fails_the_text_with_the_same_error() {
    // Arrange
    let http = FakeHttp::default().status("example.org", "/notes", 404);

    // Act
    let error = http.get_text("example.org", "/notes", TIMEOUT).unwrap_err();

    // Assert
    assert_eq!(
        error.to_string(),
        "network request failed: https://example.org/notes answered with HTTP status 404"
    );
}
