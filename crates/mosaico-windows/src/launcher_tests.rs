use windows::Win32::Foundation::{HLOCAL, LocalFree};
use windows::Win32::UI::Shell::CommandLineToArgvW;

use super::*;

/// Splits `line` the way a program started with it reads its arguments.
fn split(line: &str) -> Vec<String> {
    // The first word is read as the program name, under other rules.
    let line = wide(&format!("program {line}"));
    let mut count = 0;
    // SAFETY: `line` is nul-terminated and outlives the call.
    let argv = unsafe { CommandLineToArgvW(PCWSTR(line.as_ptr()), &mut count) };
    assert!(!argv.is_null(), "CommandLineToArgvW failed");
    let count = usize::try_from(count).unwrap_or_default();
    // SAFETY: `argv` points to `count` nul-terminated strings.
    let args = unsafe { std::slice::from_raw_parts(argv, count) }
        .iter()
        .skip(1)
        // SAFETY: Each entry is a nul-terminated string in `argv`.
        .map(|arg| unsafe { arg.to_string() }.unwrap_or_default())
        .collect();
    // SAFETY: `argv` was allocated by CommandLineToArgvW for the caller.
    unsafe { LocalFree(Some(HLOCAL(argv.cast()))) };
    args
}

fn strings(args: &[&str]) -> Vec<String> {
    args.iter().map(|arg| arg.to_string()).collect()
}

fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<OsString> {
    let pairs: Vec<(String, OsString)> = pairs
        .iter()
        .map(|(k, v)| (k.to_string(), OsString::from(v)))
        .collect();
    move |key| {
        pairs
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v.clone())
    }
}

#[test]
fn every_argument_reaches_the_program_as_written() {
    // Arrange
    let args = strings(&[
        "--new-window",
        r"C:\Program Files\notes.txt",
        "",
        r#"say "hi""#,
        r"C:\folder\",
        r#"a\"b"#,
        r"\\server\share\ end",
        "tab\there",
    ]);

    // Act
    let line = quote_args(&args);

    // Assert
    assert_eq!(split(&line), args, "{line}");
}

#[test]
fn plain_arguments_are_not_quoted() {
    // Arrange
    let args = strings(&["--profile", r"C:\tools\x.ini"]);

    // Act
    let line = quote_args(&args);

    // Assert
    assert_eq!(line, r"--profile C:\tools\x.ini");
}

#[test]
fn no_arguments_make_an_empty_command_line() {
    // Act
    let line = quote_args(&[]);

    // Assert
    assert_eq!(line, "");
}

#[test]
fn set_variables_are_expanded() {
    // Arrange
    let var = env(&[("USERPROFILE", r"C:\Users\me"), ("APP", "x")]);

    // Act
    let text = expand_vars(r"%UserProfile%\notes-%APP%.txt", var);

    // Assert
    assert_eq!(text, r"C:\Users\me\notes-x.txt");
}

#[test]
fn unset_variables_and_lone_percent_signs_stay_as_written() {
    // Arrange
    let var = env(&[("B", "two")]);

    // Act
    let text = expand_vars("100% %MISSING% %% %A%B% end%", var);

    // Assert
    assert_eq!(text, "100% %MISSING% %% %Atwo end%");
}

#[test]
fn a_command_that_does_not_exist_fails_without_a_message_box() {
    // Arrange
    let launcher = ShellLauncher::current();

    // Act
    let launched = launcher.launch("mosaico-launcher-no-such-program.exe", &[]);

    // Assert
    assert!(
        matches!(
            launched,
            Err(Error::Platform {
                operation: "start an application",
                ..
            })
        ),
        "{launched:?}"
    );
}
