use crate::config::store::test_support::temp_store;
use crate::config::{BarChoice, ProviderCheck};

#[test]
fn a_config_that_does_not_load_keeps_the_bar_it_picks() {
    // Arrange
    let store = temp_store("bar-provider-kept");
    let content = "[layout]\ngap = = 1\n\n[bar]\nprovider = \"none\"\n";
    std::fs::write(store.config_path(), content).unwrap();

    // Act
    let config = store.load_config();

    // Assert
    assert_eq!(config.bar.choice(), BarChoice::Off);
    assert_eq!(store.bar_provider(), ProviderCheck::Known(BarChoice::Off));
}

#[test]
fn a_bar_table_that_cannot_be_read_turns_the_built_in_bar_off() {
    // Arrange
    let store = temp_store("bar-provider-unreadable");
    std::fs::write(store.config_path(), "[bar]\nprovider = = \"x\"\n").unwrap();

    // Act
    let config = store.load_config();

    // Assert
    assert_eq!(config.bar.choice(), BarChoice::External);
    assert_eq!(store.bar_provider(), ProviderCheck::Unreadable);
}

#[test]
fn no_config_file_picks_the_built_in_bar() {
    // Arrange
    let store = temp_store("bar-provider-missing");

    // Act
    let check = store.bar_provider();

    // Assert
    assert_eq!(check, ProviderCheck::Known(BarChoice::Mosaico));
    assert_eq!(store.load_config().bar.choice(), BarChoice::Mosaico);
}
