//! The arguments of a command line, quoted and split the way Rust's
//! standard library and the C runtime of current Windows programs read
//! them, which is how mosaico reads its own. `CommandLineToArgvW` reads
//! `""` inside quotes differently.

use std::iter::repeat_n;

/// Appends `arg` as it is, or in quotes when it is empty or has spaces,
/// tabs or quotes, escaping its quotes and the backslashes before them
/// so it splits back unchanged.
pub fn push(line: &mut String, arg: &str) {
    if !arg.is_empty() && !arg.contains([' ', '\t', '"']) {
        line.push_str(arg);
        return;
    }
    line.push('"');
    let mut backslashes = 0;
    for c in arg.chars() {
        if c == '\\' {
            backslashes += 1;
            continue;
        }
        let escapes = if c == '"' {
            backslashes * 2 + 1
        } else {
            backslashes
        };
        line.extend(repeat_n('\\', escapes));
        line.push(c);
        backslashes = 0;
    }
    // Backslashes before the closing quote would escape it.
    line.extend(repeat_n('\\', backslashes * 2));
    line.push('"');
}

/// Splits what follows the program into arguments. Spaces and tabs
/// outside quotes separate them and quotes group them. Backslashes are
/// literal unless a quote follows: then each pair is one backslash and
/// an odd one makes the quote literal. `""` inside quotes is a quote.
pub fn split(rest: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut chars = rest.chars().peekable();
    loop {
        while chars.next_if(|c| matches!(c, ' ' | '\t')).is_some() {}
        if chars.peek().is_none() {
            return args;
        }
        let mut arg = String::new();
        let mut quoted = false;
        while let Some(c) = chars.next_if(|&c| quoted || !matches!(c, ' ' | '\t')) {
            match c {
                '\\' => {
                    let mut backslashes = 1;
                    while chars.next_if_eq(&'\\').is_some() {
                        backslashes += 1;
                    }
                    if chars.peek() == Some(&'"') {
                        arg.extend(repeat_n('\\', backslashes / 2));
                        if backslashes % 2 == 1 {
                            chars.next();
                            arg.push('"');
                        }
                    } else {
                        arg.extend(repeat_n('\\', backslashes));
                    }
                }
                '"' => {
                    if quoted && chars.next_if_eq(&'"').is_some() {
                        arg.push('"');
                    } else {
                        quoted = !quoted;
                    }
                }
                c => arg.push(c),
            }
        }
        args.push(arg);
    }
}
