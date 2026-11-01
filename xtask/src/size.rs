//! `cargo xtask size`: the release binary must stay under a ceiling.
//! The settings app's size is reported beside it, without a ceiling: it
//! ships as its own executable and only loads when the window opens.
//!
//! Unlike the metrics baselines this is a limit, not an exact count: the
//! size moves with every Rust toolchain, which CI takes from `stable`.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::cargo;

/// The largest the release binary may be, in bytes.
pub const MAX_BYTES: u64 = 2_000_000;

/// Builds the release binaries the way the release workflow does, checks
/// `mosaico` against [`MAX_BYTES`] and reports the settings app's size.
/// Returns whether `mosaico` fits.
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
    report_settings_app(root);
    let binary = release_binary(root, "mosaico");
    let bytes = match fs::metadata(&binary) {
        Ok(metadata) => metadata.len(),
        Err(e) => {
            eprintln!("could not read {}: {e}", binary.display());
            return false;
        }
    };
    let name = binary.file_name().map_or_default(|n| n.to_string_lossy());
    match verdict(bytes, MAX_BYTES) {
        Ok(line) => {
            println!("{name}: {line}");
            true
        }
        Err(line) => {
            eprintln!("{name}: {line}");
            false
        }
    }
}

/// Prints the settings app's size, which has no ceiling. Says so when it
/// was not built rather than failing.
fn report_settings_app(root: &Path) {
    let binary = release_binary(root, "mosaico-settings");
    let name = binary.file_name().map_or_default(|n| n.to_string_lossy());
    match fs::metadata(&binary) {
        Ok(metadata) => println!("{name}: {}", reported(metadata.len())),
        Err(e) => println!("{name}: not measured, could not read it: {e}"),
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

/// The size of a binary that has no ceiling, for information.
pub fn reported(bytes: u64) -> String {
    format!("{} bytes, for information (no ceiling)", grouped(bytes))
}

/// Says how `bytes` compares with `limit`: `Ok` when it fits, `Err` when
/// it does not.
pub fn verdict(bytes: u64, limit: u64) -> Result<String, String> {
    let size = format!("{} bytes", grouped(bytes));
    let ceiling = format!("the {} byte ceiling", grouped(limit));
    if bytes <= limit {
        Ok(format!(
            "{size}, {} under {ceiling}",
            grouped(limit - bytes)
        ))
    } else {
        Err(format!(
            "{size}, {} over {ceiling}. Find what grew with `cargo bloat --release -p mosaico --crates`.",
            grouped(bytes - limit)
        ))
    }
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
    fn a_binary_under_the_ceiling_passes_and_says_by_how_much() {
        // Act
        let line = verdict(1_409_024, 2_000_000);

        // Assert
        assert_eq!(
            line,
            Ok("1,409,024 bytes, 590,976 under the 2,000,000 byte ceiling".into())
        );
    }

    #[test]
    fn a_binary_at_the_ceiling_still_fits() {
        // Act / Assert
        assert!(verdict(2_000_000, 2_000_000).is_ok());
    }

    #[test]
    fn a_binary_over_the_ceiling_fails_and_says_by_how_much() {
        // Act
        let line = verdict(2_000_512, 2_000_000).unwrap_err();

        // Assert
        assert!(line.starts_with("2,000,512 bytes, 512 over"), "{line}");
    }

    #[test]
    fn the_settings_app_is_reported_without_a_ceiling() {
        // Act / Assert
        assert_eq!(
            reported(6_215_680),
            "6,215,680 bytes, for information (no ceiling)"
        );
    }

    #[test]
    fn small_numbers_have_no_separator() {
        // Act / Assert
        assert_eq!(grouped(0), "0");
        assert_eq!(grouped(999), "999");
        assert_eq!(grouped(1000), "1,000");
    }
}
