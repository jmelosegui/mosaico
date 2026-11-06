use super::*;

fn strings(args: &[&str]) -> Vec<String> {
    args.iter().map(ToString::to_string).collect()
}

#[test]
fn arguments_are_split_at_spaces() {
    assert_eq!(split_args("  -p   dev "), strings(&["-p", "dev"]));
}

#[test]
fn quotes_keep_spaces_in_an_argument() {
    assert_eq!(
        split_args(r#"-p "Ubuntu Dev" x"y z""#),
        strings(&["-p", "Ubuntu Dev", "xy z"])
    );
}

#[test]
fn empty_quotes_are_an_empty_argument() {
    assert_eq!(split_args(r#"a "" b"#), strings(&["a", "", "b"]));
}

#[test]
fn joined_arguments_split_back_to_the_same() {
    // Arrange
    let args = strings(&["-p", "Ubuntu Dev", "", "--x"]);

    // Act
    let line = join_args(&args);

    // Assert
    assert_eq!(line, r#"-p "Ubuntu Dev" "" --x"#);
    assert_eq!(split_args(&line), args);
}
