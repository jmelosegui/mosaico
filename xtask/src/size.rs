//! `cargo xtask size`: reports the size of the release binaries, so CI
//! logs show how they grow. Neither has a ceiling: `mosaico` had one
//! until it embedded a script runtime, which was worth more than staying
//! under it.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::cargo;

/// Builds the release binaries the way the release workflow does and
/// prints the size of `mosaico` and the settings app. Returns whether
/// the build succeeded and `mosaico` could be measured.
pub fn run(root: &Path) -> bool {
    let built = cargo(root).args(["build", "--release", "--quiet"]).status();
    match built {
        Ok(status) if status.success() => {}
        Ok(_) => return false,
        Err(e) => {
            eprintln!("could not build the release binary: {e}");
            return false;
        }
    }
    let measured = report(root, "mosaico");
    report(root, "mosaico-settings");
    measured
}

/// Prints the size of the release binary `name`, or why it could not be
/// read. Returns whether it was measured.
fn report(root: &Path, name: &str) -> bool {
    let binary = release_binary(root, name);
    let file = binary.file_name().map_or_default(|n| n.to_string_lossy());
    match fs::metadata(&binary) {
        Ok(metadata) => {
            println!("{file}: {}", reported(metadata.len()));
            true
        }
        Err(e) => {
            println!("{file}: not measured, could not read it: {e}");
            false
        }
    }
}

/// Where `cargo build --release` puts the binary `name`, honouring
/// `CARGO_TARGET_DIR`.
fn release_binary(root: &Path, name: &str) -> PathBuf {
    let target = env::var_os("CARGO_TARGET_DIR").map_or_else(|| root.join("target"), PathBuf::from);
    target
        .join("release")
        .join(format!("{name}{}", env::consts::EXE_SUFFIX))
}

/// The size of a binary, for information.
pub fn reported(bytes: u64) -> String {
    format!("{} bytes", grouped(bytes))
}

/// `n` with a comma between each group of three digits.
fn grouped(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, digit) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_size_is_reported_with_grouped_digits() {
        // Act / Assert
        assert_eq!(reported(6_215_680), "6,215,680 bytes");
    }

    #[test]
    fn a_size_over_the_old_ceiling_is_only_reported() {
        // Act / Assert
        assert_eq!(reported(2_000_512), "2,000,512 bytes");
    }

    #[test]
    fn small_numbers_have_no_separator() {
        // Act / Assert
        assert_eq!(grouped(0), "0");
        assert_eq!(grouped(999), "999");
        assert_eq!(grouped(1000), "1,000");
    }
}
