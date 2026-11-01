use std::path::{Path, PathBuf};

use mosaico_core::system::LaunchCommand;

use super::split;

fn command(program: &str, args: &[&str]) -> LaunchCommand {
    LaunchCommand {
        program: PathBuf::from(program),
        args: args.iter().map(|arg| arg.to_string()).collect(),
    }
}

/// A file check that finds nothing.
fn nothing(_: &Path) -> bool {
    false
}

/// A file check that finds only `files`.
fn only(files: &[&str]) -> impl Fn(&Path) -> bool {
    let files: Vec<PathBuf> = files.iter().map(PathBuf::from).collect();
    move |path: &Path| files.iter().any(|file| file == path)
}

#[test]
fn a_quoted_program_ends_at_its_closing_quote() {
    // Act
    let parsed = split(
        r#"  "C:\Program Files\mosaico\mosaico.exe"   start"#,
        nothing,
    );

    // Assert
    assert_eq!(
        parsed,
        command(r"C:\Program Files\mosaico\mosaico.exe", &["start"])
    );
}

#[test]
fn an_unquoted_program_with_spaces_is_found_the_way_logon_finds_it() {
    // Arrange
    let is_file = only(&[r"C:\Program Files\Logi\LogiBolt\LogiBolt.exe"]);

    // Act
    let parsed = split(
        r"C:\Program Files\Logi\LogiBolt\LogiBolt.exe --startup",
        is_file,
    );

    // Assert
    assert_eq!(
        parsed,
        command(
            r"C:\Program Files\Logi\LogiBolt\LogiBolt.exe",
            &["--startup"]
        )
    );
}

#[test]
fn an_unquoted_name_without_an_extension_is_found_with_exe_added() {
    // Arrange
    let is_file = only(&[r"C:\Tools\my app\mosaico.exe"]);

    // Act
    let parsed = split(r"C:\Tools\my app\mosaico start", is_file);

    // Assert
    assert_eq!(parsed, command(r"C:\Tools\my app\mosaico.exe", &["start"]));
}

#[test]
fn the_shortest_run_of_words_that_names_a_file_wins() {
    // Arrange
    let is_file = only(&[r"C:\Program.exe", r"C:\Program Files\mosaico\mosaico.exe"]);

    // Act
    let parsed = split(r"C:\Program Files\mosaico\mosaico.exe start", is_file);

    // Assert
    assert_eq!(
        parsed,
        command(r"C:\Program.exe", &[r"Files\mosaico\mosaico.exe", "start"])
    );
}

#[test]
fn an_unquoted_program_that_names_no_file_ends_at_the_first_space() {
    // Act
    let parsed = split(r"C:\Program Files\mosaico\mosaico.exe start", nothing);

    // Assert
    assert_eq!(
        parsed,
        command(r"C:\Program", &[r"Files\mosaico\mosaico.exe", "start"])
    );
}

#[test]
fn a_relative_program_is_not_looked_up() {
    // Act
    let parsed = split("mosaico start", |path: &Path| panic!("looked up {path:?}"));

    // Assert
    assert_eq!(parsed, command("mosaico", &["start"]));
}

#[test]
fn arguments_split_the_way_mosaico_reads_its_own() {
    // Act
    let parsed = split(
        r#""C:\mosaico.exe" C:\a\b "C:\c d\\" a\\\"b "x""y" ab"c d"e"#,
        nothing,
    );

    // Assert
    assert_eq!(
        parsed,
        command(
            r"C:\mosaico.exe",
            &[r"C:\a\b", r"C:\c d\", r#"a\"b"#, r#"x"y"#, "abc de"]
        )
    );
}

#[test]
fn a_line_without_a_program_gives_an_empty_program() {
    for line in ["", " \t ", r#""""#, r#""" start"#] {
        // Act
        let parsed = split(line, nothing);

        // Assert
        assert!(parsed.program.as_os_str().is_empty(), "{line:?}");
    }
}
