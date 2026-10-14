use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use crate::Error;
use crate::http::fake::FakeHttp;

use super::*;

/// Where the rules of the `test-os` rule set are downloaded from.
const RULES: &str = "/jmelosegui/mosaico-rules/main/test-os/rules.toml";

/// What the cache holds before a download.
const CACHED: &str = "[[rule]]\nmatch_class = \"Cached\"\nmanage = false\n";

/// Two valid rules.
const VALID: &str = "\
[[rule]]
match_class = \"Chrome_WidgetWin_1\"
manage = false

[[rule]]
match_title = \"Picture in picture\"
manage = false
";

/// A cache in a fresh directory for `test` that holds [`CACHED`].
fn cache(test: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("mosaico-community-rules-tests")
        .join(format!("{test}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let cache = dir.join("rules.toml");
    fs::write(&cache, CACHED).unwrap();
    cache
}

/// GitHub serving `body` as the rules of the `test-os` rule set.
fn serving(body: &str) -> FakeHttp {
    FakeHttp::default().answer("raw.githubusercontent.com", RULES, body)
}

#[test]
fn valid_rules_are_cached_and_returned() {
    // Arrange
    let cache = cache("valid");
    let http = serving(VALID);

    // Act
    let rules = download(&http, "test-os", &cache).unwrap();

    // Assert
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].match_class.as_deref(), Some("Chrome_WidgetWin_1"));
    assert_eq!(fs::read_to_string(&cache).unwrap(), VALID);
}

#[test]
fn the_rules_come_from_the_folder_of_the_rule_set() {
    // Arrange
    let cache = cache("folder");
    let http = serving(VALID);

    // Act
    download(&http, "test-os", &cache).unwrap();

    // Assert
    assert_eq!(
        http.requests(),
        [(
            format!("https://raw.githubusercontent.com{RULES}"),
            Duration::from_secs(5)
        )]
    );
}

#[test]
fn an_unreachable_host_keeps_the_cached_rules() {
    // Arrange
    let cache = cache("unreachable");
    let http = FakeHttp::default();

    // Act
    let error = download(&http, "test-os", &cache).unwrap_err();

    // Assert
    assert!(matches!(error, Error::Network(_)), "{error:?}");
    assert_eq!(fs::read_to_string(&cache).unwrap(), CACHED);
}

#[test]
fn invalid_toml_does_not_overwrite_the_cache() {
    // Arrange
    let cache = cache("invalid");
    let http = serving("[[rule]\nmatch_class = ");

    // Act
    let error = download(&http, "test-os", &cache).unwrap_err();

    // Assert
    assert!(matches!(error, Error::Config { .. }), "{error:?}");
    assert_eq!(fs::read_to_string(&cache).unwrap(), CACHED);
}

#[test]
fn rules_that_do_not_fit_the_schema_do_not_overwrite_the_cache() {
    // Arrange -- valid TOML, but `manage` must be a boolean.
    let cache = cache("schema");
    let http = serving("[[rule]]\nmatch_class = \"App\"\nmanage = \"no\"\n");

    // Act
    let error = download(&http, "test-os", &cache).unwrap_err();

    // Assert
    assert!(matches!(error, Error::Config { .. }), "{error:?}");
    assert_eq!(fs::read_to_string(&cache).unwrap(), CACHED);
}

#[test]
fn a_file_without_rules_does_not_overwrite_the_cache() {
    // Arrange
    let cache = cache("empty");
    let http = serving("# No rules yet.\n");

    // Act
    let error = download(&http, "test-os", &cache).unwrap_err();

    // Assert
    assert!(matches!(error, Error::Config { .. }), "{error:?}");
    assert_eq!(fs::read_to_string(&cache).unwrap(), CACHED);
}

#[test]
fn a_cache_that_cannot_be_written_is_an_error() {
    // Arrange -- the cache's directory does not exist.
    let cache = cache("unwritable")
        .with_file_name("missing")
        .join("rules.toml");
    let http = serving(VALID);

    // Act
    let error = download(&http, "test-os", &cache).unwrap_err();

    // Assert
    assert!(matches!(error, Error::Io(_)), "{error:?}");
    assert!(!cache.exists());
}
