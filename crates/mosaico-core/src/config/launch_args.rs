//! The arguments of a launch binding as one line of text, the way a
//! settings field shows them and the user types them.

/// Splits `text` into arguments at spaces. Double quotes keep spaces in
/// an argument and are dropped: `-p "Ubuntu Dev"` is `-p` and
/// `Ubuntu Dev`, and `""` is an empty argument.
pub fn split_args(text: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    let mut started = false;
    for ch in text.chars() {
        match ch {
            '"' => {
                quoted = !quoted;
                started = true;
            }
            ch if ch.is_whitespace() && !quoted => {
                if started {
                    args.push(std::mem::take(&mut current));
                    started = false;
                }
            }
            ch => {
                current.push(ch);
                started = true;
            }
        }
    }
    if started {
        args.push(current);
    }
    args
}

/// Joins `args` into the line [`split_args`] reads back, quoting the
/// arguments that are empty or hold spaces.
pub fn join_args(args: &[String]) -> String {
    let shown: Vec<String> = args
        .iter()
        .map(|arg| {
            if arg.is_empty() || arg.contains(char::is_whitespace) {
                format!("\"{arg}\"")
            } else {
                arg.clone()
            }
        })
        .collect();
    shown.join(" ")
}

#[cfg(test)]
#[path = "launch_args_tests.rs"]
mod tests;
