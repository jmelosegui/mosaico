//! `cargo xtask msi [--version X.Y.Z] [--no-build]`: builds the Windows
//! installer, `target/release/mosaico-windows-amd64.msi`, from
//! `packaging/windows/mosaico.wxs` and the release binaries.
//!
//! It needs WiX 5 and its util extension, which the release workflow
//! installs the same way:
//!
//! ```text
//! dotnet tool install --global wix --version 5.0.2
//! wix extension add --global WixToolset.Util.wixext/5.0.2
//! ```
//!
//! The package version comes from `crates/mosaico/Cargo.toml` unless
//! `--version` gives one, which is how two packages are built to try an
//! upgrade locally.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::cargo;

/// The WiX version the package is built with. WiX 6 and later ask every
/// build to accept a maintenance fee EULA, so the build stays on 5.
pub const WIX_VERSION: &str = "5.0.2";

/// The name of the package, as the release publishes it.
pub const PACKAGE: &str = "mosaico-windows-amd64.msi";

/// What the command line asked for.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Options {
    /// The version to give the package instead of the crate's.
    pub version: Option<String>,
    /// Package the release binaries already built instead of building them.
    pub no_build: bool,
}

impl Options {
    /// Reads the arguments after `msi`, or says what is wrong with them.
    pub fn parse(args: &[&str]) -> Result<Self, String> {
        let mut options = Self::default();
        let mut args = args.iter();
        while let Some(arg) = args.next() {
            match *arg {
                "--no-build" => options.no_build = true,
                "--version" => match args.next() {
                    Some(version) => options.version = Some((*version).to_string()),
                    None => return Err("--version needs a version, as in 1.2.3".into()),
                },
                other => return Err(format!("unknown argument {other}")),
            }
        }
        Ok(options)
    }
}

/// Builds the package. Returns whether it was built.
pub fn run(root: &Path, options: &Options) -> bool {
    if !options.no_build {
        let built = cargo(root)
            .args([
                "build",
                "--release",
                "--quiet",
                "-p",
                "mosaico",
                "-p",
                "mosaico-settings",
            ])
            .status();
        if !built.is_ok_and(|status| status.success()) {
            eprintln!("could not build the release binaries");
            return false;
        }
    }

    let version = match &options.version {
        Some(version) => msi_version(version),
        None => crate_version(root).and_then(|version| msi_version(&version)),
    };
    let version = match version {
        Ok(version) => version,
        Err(e) => {
            eprintln!("{e}");
            return false;
        }
    };

    let release = release_dir(root);
    let output = release.join(PACKAGE);
    let status = wix()
        .arg("build")
        .arg(root.join("packaging").join("windows").join("mosaico.wxs"))
        .args(["-arch", "x64", "-ext", "WixToolset.Util.wixext"])
        .arg("-d")
        .arg(format!("Version={version}"))
        .arg("-d")
        .arg(format!("BinDir={}", release.display()))
        .arg("-d")
        .arg(format!("AssetsDir={}", root.join("assets").display()))
        .arg("-o")
        .arg(&output)
        .status();
    match status {
        Ok(status) if status.success() => {
            println!("{} (version {version})", output.display());
            true
        }
        Ok(_) => false,
        Err(e) => {
            eprintln!(
                "could not run wix: {e}. Install it with `dotnet tool install --global wix \
                 --version {WIX_VERSION}` and `wix extension add --global \
                 WixToolset.Util.wixext/{WIX_VERSION}`."
            );
            false
        }
    }
}

/// The version of the `mosaico` crate.
fn crate_version(root: &Path) -> Result<String, String> {
    let manifest = root.join("crates").join("mosaico").join("Cargo.toml");
    let text = fs::read_to_string(&manifest)
        .map_err(|e| format!("could not read {}: {e}", manifest.display()))?;
    text.lines()
        .find_map(|line| line.strip_prefix("version = \""))
        .and_then(|rest| rest.strip_suffix('"'))
        .map(str::to_string)
        .ok_or_else(|| format!("no version in {}", manifest.display()))
}

/// The package version for the crate version `version`.
///
/// Windows Installer compares only three numbers, the first two below
/// 256 and the third below 65536, and has no prerelease part. A
/// prerelease or build suffix is dropped, so `1.2.3-rc.1` is packaged as
/// `1.2.3`.
pub fn msi_version(version: &str) -> Result<String, String> {
    let core = version.split(['-', '+']).next().unwrap_or_default();
    let parts: Vec<&str> = core.split('.').collect();
    let limits = [255, 255, 65535];
    if parts.len() != limits.len() {
        return Err(format!("{version} is not a version like 1.2.3"));
    }
    for (part, limit) in parts.iter().zip(limits) {
        match part.parse::<u32>() {
            Ok(n) if n <= limit => {}
            _ => return Err(format!("{version} cannot be an installer version")),
        }
    }
    Ok(core.to_string())
}

/// Where `cargo build --release` puts the binaries, honouring
/// `CARGO_TARGET_DIR`.
fn release_dir(root: &Path) -> PathBuf {
    env::var_os("CARGO_TARGET_DIR")
        .map_or_else(|| root.join("target"), PathBuf::from)
        .join("release")
}

/// The `wix` command: the one on the PATH, or where `dotnet tool install
/// --global` puts it when its folder is not on the PATH yet.
fn wix() -> Command {
    let installed = env::var_os("USERPROFILE")
        .map(|home| {
            Path::new(&home)
                .join(".dotnet")
                .join("tools")
                .join("wix.exe")
        })
        .filter(|wix| wix.is_file());
    Command::new(installed.unwrap_or_else(|| PathBuf::from("wix")))
}

#[cfg(test)]
#[path = "msi_tests.rs"]
mod tests;
