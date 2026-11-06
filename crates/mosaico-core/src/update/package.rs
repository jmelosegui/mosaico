//! Telling whether the platform's installer package owns the running
//! mosaico, which then must be updated through that installer.

use std::path::Path;

use super::Installer;
use crate::system::FileProbe;

/// Whether `exe`, the mosaico executable that is running, is the one the
/// platform's installer package installed.
///
/// It is when the package recorded a folder and `exe` is in it, however
/// either path is spelled. A copy elsewhere, such as one installed by
/// hand or a development build, is not, even while the package is
/// installed too.
pub fn is_packaged(installer: &dyn Installer, files: &dyn FileProbe, exe: &Path) -> bool {
    let (Some(folder), Some(name)) = (installer.package_folder(), exe.file_name()) else {
        return false;
    };
    let installed = folder.join(name);
    installed == exe || files.same_file(&installed, exe)
}

#[cfg(test)]
#[path = "tests/package_tests.rs"]
mod tests;
