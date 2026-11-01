use crate::Error;

use super::*;

struct Fonts(Result<Vec<&'static str>>);

impl FontCatalog for Fonts {
    fn families(&self) -> Result<Vec<String>> {
        match &self.0 {
            Ok(names) => Ok(names.iter().map(|n| (*n).to_owned()).collect()),
            Err(_) => Err(Error::Unsupported("listing fonts")),
        }
    }
}

#[test]
fn families_are_sorted_ignoring_case_and_listed_once() {
    // Arrange
    let catalog = Fonts(Ok(vec![
        "Segoe UI",
        "Arial",
        "Segoe UI",
        "  ",
        "@Yu Gothic",
    ]));

    // Act
    let choices = font_choices(&catalog, "");

    // Assert
    assert_eq!(choices, ["Arial", "Segoe UI"]);
}

#[test]
fn the_font_in_use_is_offered_even_when_it_is_not_installed() {
    // Arrange
    let catalog = Fonts(Ok(vec!["Arial"]));

    // Act
    let choices = font_choices(&catalog, "My Nerd Font");

    // Assert
    assert_eq!(choices, ["Arial", "My Nerd Font"]);
}

#[test]
fn a_catalog_that_cannot_list_offers_only_the_font_in_use() {
    // Arrange
    let catalog = Fonts(Err(Error::Unsupported("listing fonts")));

    // Act and assert
    assert_eq!(font_choices(&catalog, "Consolas"), ["Consolas"]);
    assert!(font_choices(&catalog, "").is_empty());
}
