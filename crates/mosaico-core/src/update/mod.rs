//! Finding newer releases of mosaico and installing them.
//!
//! Releases are published on GitHub. Looking them up and downloading
//! them goes through an [`HttpClient`](crate::http::HttpClient), so it
//! is tested without a network. How a release is packaged and installed
//! differs per platform, which says so through an
//! [`Installer`](crate::update::Installer).

mod download;
mod install;
mod package;
mod release;
mod restart;

pub use download::{Binaries, download_binaries, download_package};
pub use install::Installer;
pub use package::is_packaged;
pub use release::{is_newer_tag, latest_tag, newer_release};
pub use restart::RestartMarker;
