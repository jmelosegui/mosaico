//! The single command line a Windows program receives in place of an
//! argument list.
//!
//! Windows hands a new process one string, and the program's C runtime
//! splits it back into arguments: whitespace separates them, double
//! quotes group them, and backslashes only escape when a run of them
//! ends in a quote. [`command_line`] joins arguments so that splitting
//! gives them back unchanged.

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

/// `program` followed by `args`, quoted as the C runtime splits them,
/// as UTF-16 ending in a nul, ready for `CreateProcessW`.
///
/// The program is quoted whole when it holds a space or a tab: a path
/// cannot hold a quote, and its backslashes are never escapes.
pub(crate) fn command_line(program: &OsStr, args: &[&OsStr]) -> Vec<u16> {
    let mut line = Vec::new();
    let program: Vec<u16> = program.encode_wide().collect();
    let spaced = program.is_empty() || program.iter().any(|&c| c == SPACE || c == TAB);
    if spaced {
        line.push(QUOTE);
    }
    line.extend_from_slice(&program);
    if spaced {
        line.push(QUOTE);
    }
    for arg in args {
        line.push(SPACE);
        push_arg(&mut line, arg);
    }
    line.push(0);
    line
}

const SPACE: u16 = b' ' as u16;
const TAB: u16 = b'\t' as u16;
const QUOTE: u16 = b'"' as u16;
const BACKSLASH: u16 = b'\\' as u16;

/// Appends `arg`, quoted when it is empty or holds whitespace or a quote.
/// Inside quotes, a quote is escaped with a backslash, and the run of
/// backslashes before a quote, or before the closing quote, is doubled.
fn push_arg(line: &mut Vec<u16>, arg: &OsStr) {
    let arg: Vec<u16> = arg.encode_wide().collect();
    let plain = !arg.is_empty()
        && !arg
            .iter()
            .any(|&c| c == SPACE || c == TAB || c == QUOTE || c == b'\n' as u16);
    if plain {
        line.extend_from_slice(&arg);
        return;
    }
    line.push(QUOTE);
    let mut backslashes = 0;
    for &c in &arg {
        if c == BACKSLASH {
            backslashes += 1;
            continue;
        }
        let doubled = if c == QUOTE {
            2 * backslashes + 1
        } else {
            backslashes
        };
        line.extend(std::iter::repeat_n(BACKSLASH, doubled));
        line.push(c);
        backslashes = 0;
    }
    line.extend(std::iter::repeat_n(BACKSLASH, 2 * backslashes));
    line.push(QUOTE);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(program: &str, args: &[&str]) -> String {
        let args: Vec<&OsStr> = args.iter().map(OsStr::new).collect();
        let mut wide = command_line(OsStr::new(program), &args);
        assert_eq!(wide.pop(), Some(0), "ends in a nul");
        String::from_utf16(&wide).unwrap()
    }

    #[test]
    fn plain_arguments_are_joined_by_spaces() {
        assert_eq!(
            line(r"C:\mosaico\mosaico.exe", &["daemon"]),
            r"C:\mosaico\mosaico.exe daemon"
        );
    }

    #[test]
    fn a_program_with_a_space_is_quoted_without_escaping_its_backslashes() {
        assert_eq!(
            line(r"C:\Program Files\mosaico\", &[]),
            r#""C:\Program Files\mosaico\""#
        );
    }

    #[test]
    fn arguments_with_whitespace_or_nothing_are_quoted() {
        assert_eq!(
            line("msiexec", &[r"C:\Temp dir\a.msi", "", "/l*v"]),
            r#"msiexec "C:\Temp dir\a.msi" "" /l*v"#
        );
    }

    #[test]
    fn quotes_and_the_backslashes_before_them_are_escaped() {
        assert_eq!(line("p", &[r#"say "hi""#]), r#"p "say \"hi\"""#);
        assert_eq!(line("p", &[r#"a\"b"#]), r#"p "a\\\"b""#);
    }

    #[test]
    fn trailing_backslashes_of_a_quoted_argument_are_doubled() {
        assert_eq!(line("p", &[r"C:\a dir\"]), r#"p "C:\a dir\\""#);
        assert_eq!(line("p", &[r"C:\plain\"]), r"p C:\plain\");
    }
}
