use super::*;

#[test]
fn a_provider_is_compared_without_regard_to_case() {
    // Arrange
    let section = BarSection {
        provider: " External ".to_string(),
    };

    // Act
    let check = section.check();

    // Assert
    assert_eq!(check, ProviderCheck::Known(BarChoice::External));
    assert_eq!(check.warning(), None);
}

#[test]
fn an_unknown_provider_is_read_as_external_and_said_so() {
    // Arrange
    let section = BarSection {
        provider: "zeebar".to_string(),
    };

    // Act
    let check = section.check();

    // Assert
    assert_eq!(check.choice(), BarChoice::External);
    let warning = check.warning().unwrap_or_default();
    assert!(warning.contains("\"zeebar\""), "{warning}");
}

#[test]
fn every_choice_reads_back_from_its_name() {
    for choice in BarChoice::ALL {
        assert_eq!(BarChoice::parse(choice.name()), Some(choice));
    }
    assert_eq!(BarChoice::Off.name(), "none");
}

#[test]
fn a_file_without_the_table_has_the_built_in_bar() {
    // Arrange
    let content = "[layout]\ngap = 10\n";

    // Act
    let section = read_bar_section(content);

    // Assert
    assert_eq!(section, Some(BarSection::default()));
}

#[test]
fn the_table_is_read_when_another_table_does_not_fit_the_config() {
    // Arrange
    let content = "[layout]\ngap = \"wide\"\n\n[bar]\nprovider = \"none\"\n";

    // Act
    let section = read_bar_section(content);

    // Assert
    assert_eq!(section.map(|s| s.choice()), Some(BarChoice::Off));
}

#[test]
fn the_table_is_read_alone_when_the_file_is_not_toml() {
    // Arrange
    let content = "[layout]\ngap = = 10\n\n[bar]\nprovider = \"external\"\n\n[bar.reserve]\ntop = 40\n\n[tray]\nenabled = true\n";

    // Act
    let section = read_bar_section(content);

    // Assert
    assert_eq!(section.map(|s| s.choice()), Some(BarChoice::External));
}

#[test]
fn a_broken_file_without_the_table_has_the_built_in_bar() {
    // Arrange
    let content = "[layout]\ngap = = 10\n";

    // Act
    let section = read_bar_section(content);

    // Assert
    assert_eq!(section, Some(BarSection::default()));
}

#[test]
fn a_table_that_cannot_be_read_gives_none() {
    for content in [
        "[bar]\nprovider = = \"none\"\n",
        "[bar]\nprovider = 3\n",
        "bar.provider = \"none\"\n[layout\n",
    ] {
        assert_eq!(read_bar_section(content), None, "{content}");
    }
}

#[test]
fn an_unreadable_table_is_external_and_said_so() {
    // Act
    let check = ProviderCheck::Unreadable;

    // Assert
    assert_eq!(check.choice(), BarChoice::External);
    assert_eq!(BarSection::unreadable().choice(), BarChoice::External);
    assert!(check.warning().is_some());
}
