use std::time::Duration;

use crate::Error;
use crate::http::fake::FakeHttp;

use super::*;

/// GitHub answering the lookup of the latest release with `json`.
fn github(json: &str) -> FakeHttp {
    FakeHttp::default().answer(
        "api.github.com",
        "/repos/jmelosegui/mosaico/releases/latest",
        json,
    )
}

#[test]
fn the_latest_tag_is_read_from_the_latest_release() {
    // Arrange
    let http = github(r#"{"tag_name":"v0.2.0","name":"Release 0.2.0"}"#);

    // Act
    let tag = latest_tag(&http).unwrap();

    // Assert
    assert_eq!(tag, "v0.2.0");
    assert_eq!(
        http.requests(),
        [(
            "https://api.github.com/repos/jmelosegui/mosaico/releases/latest".to_string(),
            Duration::from_secs(5)
        )]
    );
}

#[test]
fn an_unreachable_host_is_a_network_error() {
    // Arrange
    let http = FakeHttp::default();

    // Act
    let error = latest_tag(&http).unwrap_err();

    // Assert
    assert!(matches!(error, Error::Network(_)), "{error:?}");
}

#[test]
fn a_rate_limited_lookup_is_a_network_error() {
    // Arrange -- GitHub answers 403 once the rate limit is used up.
    let http = FakeHttp::default().status(
        "api.github.com",
        "/repos/jmelosegui/mosaico/releases/latest",
        403,
    );

    // Act
    let error = latest_tag(&http).unwrap_err();

    // Assert
    assert!(matches!(error, Error::Network(_)), "{error:?}");
    assert!(
        error.to_string().ends_with("answered with HTTP status 403"),
        "{error}"
    );
}

#[test]
fn an_answer_without_a_tag_is_a_network_error() {
    // Arrange -- a successful answer that is not a release, as a proxy
    // that answers with a page of its own gives.
    let http = github(r#"{"message":"Not a release"}"#);

    // Act
    let error = latest_tag(&http).unwrap_err();

    // Assert
    assert!(matches!(error, Error::Network(_)), "{error:?}");
    assert!(
        error.to_string().contains("named no release tag"),
        "{error}"
    );
}

#[test]
fn a_newer_release_is_reported_with_its_tag() {
    // Arrange
    let http = github(r#"{"tag_name":"v0.2.0"}"#);

    // Act
    let newer = newer_release(&http, "0.1.0").unwrap();

    // Assert
    assert_eq!(newer, Some("v0.2.0".to_string()));
}

#[test]
fn the_running_version_is_not_newer() {
    // Arrange
    let http = github(r#"{"tag_name":"v0.1.0"}"#);

    // Act
    let newer = newer_release(&http, "0.1.0").unwrap();

    // Assert
    assert_eq!(newer, None);
}

#[test]
fn an_older_release_is_not_newer() {
    // Arrange
    let http = github(r#"{"tag_name":"v0.1.0"}"#);

    // Act
    let newer = newer_release(&http, "0.2.0").unwrap();

    // Assert
    assert_eq!(newer, None);
}

#[test]
fn a_failed_lookup_is_not_taken_for_being_up_to_date() {
    // Arrange
    let http = FakeHttp::default();

    // Act
    let newer = newer_release(&http, "0.1.0");

    // Assert
    assert!(newer.is_err(), "{newer:?}");
}

#[test]
fn a_tag_is_compared_with_or_without_its_v() {
    // Act / Assert
    assert!(is_newer_tag("v1.0.0", "0.9.1"));
    assert!(is_newer_tag("1.0.0", "0.9.1"));
    assert!(!is_newer_tag("v0.9.1", "0.9.1"));
}
