use super::*;

fn file(crate_name: &str, path: &str, text: &str) -> SourceFile {
    SourceFile {
        crate_name: crate_name.into(),
        path: path.into(),
        text: text.into(),
    }
}

#[test]
fn tests_are_counted_by_their_attribute() {
    // Act
    let counts = count_file("src/a.rs", "#[test]\nfn a() {}\n    #[test]\nfn b() {}\n");

    // Assert
    assert_eq!(counts.tests, 2);
}

#[test]
fn only_lines_beyond_the_limit_count() {
    // Arrange
    let at_limit = "x\n".repeat(LINE_LIMIT);
    let over = "x\n".repeat(LINE_LIMIT + 7);

    // Act / Assert
    assert_eq!(count_file("src/a.rs", &at_limit).lines_over_limit, 0);
    assert_eq!(count_file("src/a.rs", &over).lines_over_limit, 7);
}

#[test]
fn unsafe_counts_in_code_but_not_in_comments_or_longer_names() {
    // Arrange
    let text = "unsafe { f() } // unsafe\nunsafe fn g() {}\n#![forbid(unsafe_code)]\n";

    // Act
    let counts = count_file("src/a.rs", text);

    // Assert
    assert_eq!(counts.unsafe_keywords, 2);
}

#[test]
fn unwraps_count_only_outside_test_code() {
    // Arrange
    let text = "let a = b.unwrap();\nlet c = d.expect(\"x\");\n#[cfg(test)]\nmod tests {\n    e.unwrap();\n}\n";

    // Act
    let library = count_file("src/a.rs", text);
    let test_file = count_file("src/tiler/tests/a.rs", "a.unwrap();\n");
    let sibling = count_file("src/a_tests.rs", "a.unwrap();\n");
    let fake = count_file("src/fake_desktop.rs", "a.unwrap();\n");
    let module = count_file("src/config/tests.rs", "a.unwrap();\n");

    // Assert
    assert_eq!(library.unwraps, 2);
    assert_eq!(
        (
            test_file.unwraps,
            sibling.unwraps,
            fake.unwraps,
            module.unwraps
        ),
        (0, 0, 0, 0)
    );
}

#[test]
fn a_test_module_declared_elsewhere_leaves_the_rest_of_the_file_counted() {
    // Arrange
    let text = "#[cfg(test)]\n#[path = \"a_tests.rs\"]\nmod tests;\n\nfn f() { x.unwrap(); }\n";

    // Act
    let counts = count_file("src/a.rs", text);

    // Assert
    assert_eq!(counts.unwraps, 1);
}

#[test]
fn a_test_only_function_is_test_code_up_to_its_closing_brace() {
    // Arrange
    let text = "#[cfg(test)]\npub(crate) fn letter() -> Key {\n    x.expect(\"a letter\")\n}\n\nfn f() { y.unwrap(); }\n";

    // Act
    let counts = count_file("src/key.rs", text);

    // Assert
    assert_eq!(counts.unwraps, 1);
}

#[test]
fn os_names_count_in_comments_but_not_inside_other_names() {
    // Arrange
    let text =
        "/// Takes an HWND.\nuse windows::Win32;\nuse mosaico_windows::x;\nlet hwnd_like = 1;\n";

    // Act
    let counts = count_file("src/a.rs", text);

    // Assert
    assert_eq!(counts.os_names, 2);
}

#[test]
fn the_report_sums_each_crate_and_only_checks_the_crates_a_rule_covers() {
    // Arrange
    let files = [
        file(
            "mosaico-core",
            "src/a.rs",
            "#[test]\nfn a() { HWND; x.unwrap(); }\n",
        ),
        file("mosaico-core", "src/b.rs", "#[test]\nfn b() {}\n"),
        file(
            "mosaico",
            "src/main.rs",
            "fn main() { x.expect(\"y\"); HWND; }\n",
        ),
    ];

    // Act
    let report = report(&files);

    // Assert
    let core = &report.values()["mosaico-core"];
    assert_eq!(core["tests"], 2);
    assert_eq!(core["os_names"], 1);
    assert_eq!(core["unwraps_outside_tests"], 1);
    let cli = &report.values()["mosaico"];
    assert!(!cli.contains_key("os_names") && !cli.contains_key("unwraps_outside_tests"));
    assert_eq!(
        report.get("mosaico-core", "tests").map(|m| m.better),
        Some(Better::Higher)
    );
}
