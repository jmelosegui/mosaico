//! Listing the font families installed on the system.

use crate::Result;

/// The font families the user can pick for text mosaico draws.
///
/// # Contract for platform implementations
///
/// - `families` is a query: it changes nothing the user can see.
/// - It lists the family names a font setting takes, the way the
///   platform's own text drawing names them. A family with several faces
///   (regular, bold, italic) is listed once.
/// - The order and the case are the platform's; callers sort and
///   de-duplicate with [`font_choices`].
/// - It fails only when the list cannot be read at all. A platform that
///   cannot list fonts returns [`Error::Unsupported`](crate::Error::Unsupported).
pub trait FontCatalog {
    /// The installed font families.
    fn families(&self) -> Result<Vec<String>>;
}

/// The families to offer for a font setting that is now `current`: those
/// of `catalog` sorted by name ignoring case, each once, without the
/// vertical-writing variants some platforms list beside them, and
/// `current` too when it is set and not installed, so a font the file
/// names is never lost from the list. A catalog that cannot list fonts
/// offers only `current`.
pub fn font_choices(catalog: &dyn FontCatalog, current: &str) -> Vec<String> {
    let mut families: Vec<String> = catalog
        .families()
        .unwrap_or_default()
        .into_iter()
        .map(|name| name.trim().to_owned())
        .filter(|name| !name.is_empty() && !name.starts_with('@'))
        .collect();
    if !current.is_empty() {
        families.push(current.to_owned());
    }
    families.sort_by_cached_key(|name| name.to_lowercase());
    families.dedup_by_key(|name| name.to_lowercase());
    families
}

#[cfg(test)]
#[path = "tests/font_catalog_tests.rs"]
mod tests;
