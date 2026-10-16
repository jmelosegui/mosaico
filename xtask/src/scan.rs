//! Scanning source text: which files are test code, and matching names.

/// Whether the whole file is test code: a test, a fake or test support.
pub(crate) fn is_test_file(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path);
    path.starts_with("tests/")
        || path.contains("/tests/")
        || name == "tests.rs"
        || name.ends_with("_tests.rs")
        || name.starts_with("fake_")
        || name.contains("test_support")
}

/// Mentions of `name` that do not continue another identifier.
pub(crate) fn occurrences(text: &str, name: &str) -> u64 {
    text.match_indices(name)
        .filter(|&(i, _)| !text[..i].chars().next_back().is_some_and(is_ident))
        .count() as u64
}

/// Occurrences of `word` as a whole identifier.
pub(crate) fn words(text: &str, word: &str) -> u64 {
    text.match_indices(word)
        .filter(|&(i, _)| {
            let before = text[..i].chars().next_back();
            let after = text[i + word.len()..].chars().next();
            !before.is_some_and(is_ident) && !after.is_some_and(is_ident)
        })
        .count() as u64
}

/// Braces `code` opens, less those it closes.
pub(crate) fn braces(code: &str) -> i64 {
    code.matches('{').count() as i64 - code.matches('}').count() as i64
}

fn is_ident(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}
