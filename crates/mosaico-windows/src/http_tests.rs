use std::time::Duration;

use mosaico_core::Error;

use super::*;

const URL: &str = "https://github.com/jmelosegui/mosaico/releases/download/v1.0.0/app.zip";

#[test]
fn a_successful_status_lets_the_body_be_read() {
    // Act / Assert
    assert!(check_status(URL, 200).is_ok());
    assert!(check_status(URL, 204).is_ok());
}

#[test]
fn a_status_outside_200_to_299_names_the_url_and_the_status() {
    for status in [302, 404, 500] {
        // Act
        let error = check_status(URL, status).unwrap_err();

        // Assert
        assert!(matches!(error, Error::Network(_)), "{error:?}");
        assert_eq!(
            error.to_string(),
            format!("network request failed: {URL} answered with HTTP status {status}")
        );
    }
}

#[test]
fn a_timeout_under_a_millisecond_does_not_become_wait_forever() {
    // Arrange -- WinHTTP takes a timeout of 0 as "no timeout at all".
    let timeouts = [Duration::ZERO, Duration::from_micros(500)];

    // Act
    let ms = timeouts.map(timeout_ms);

    // Assert
    assert_eq!(ms, [1, 1]);
}

#[test]
fn a_timeout_is_whole_milliseconds_capped_to_what_winhttp_takes() {
    // Act / Assert
    assert_eq!(timeout_ms(Duration::from_secs(5)), 5000);
    assert_eq!(timeout_ms(Duration::from_micros(1500)), 1);
    assert_eq!(timeout_ms(Duration::MAX), i32::MAX);
}

#[test]
fn the_client_can_be_moved_to_and_shared_with_other_threads() {
    fn shareable<T: Send + Sync + 'static>(_: T) {}

    // Act / Assert -- this compiles only while the client stays
    // stateless, which the daemon's background threads rely on.
    shareable(WinHttpClient);
}
