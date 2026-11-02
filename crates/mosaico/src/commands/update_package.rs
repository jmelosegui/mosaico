//! Updating a mosaico the platform's installer package installed.
//!
//! The package owns those files, so replacing them by hand would leave
//! the installer recording an older version than the one installed, and
//! the next upgrade or uninstall confused. Instead, `mosaico update`
//! downloads the release's package and starts it. The package's own
//! hooks (see `installer_hook.rs`) stop the daemon, replace the files and
//! start the new daemon again if it was running. This process exits
//! right away, so the file it runs from is free by then.

use mosaico_core::update;

use super::context::{Context, Exit, complain, say};

/// Downloads `tag`'s installer package and starts it, reinstalling when
/// `reinstall` is set, as for `update --force` on the latest version.
pub fn execute(cx: &mut Context, tag: &str, reinstall: bool) -> Exit {
    say!(cx, "Downloading the installer for {tag}...");
    let package = match update::download_package(cx.http, cx.installer, tag) {
        Ok(package) => package,
        Err(e) => {
            complain!(cx, "Error: {e}");
            return Exit::Failure;
        }
    };

    if let Err(e) = cx.installer.run_package(&package, reinstall) {
        complain!(cx, "Error: {e}");
        return Exit::Failure;
    }
    say!(
        cx,
        "The installer is running: it stops mosaico, installs {tag} and starts mosaico again if it was running."
    );
    Exit::Success
}

#[cfg(test)]
#[path = "tests/update_package_tests.rs"]
mod tests;
