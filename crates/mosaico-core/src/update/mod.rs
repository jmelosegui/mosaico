//! Finding newer releases of mosaico and installing them.
//!
//! Releases are published on GitHub. Looking them up and downloading
//! them goes through an [`HttpClient`](crate::http::HttpClient), so it
//! is tested without a network. How a release is packaged and installed
//! differs per platform, which says so through an
//! [`Installer`](crate::update::Installer).

mod download;
mod install;
mod release;

pub use download::{Binaries, download_binaries};
pub use install::Installer;
pub use release::{is_newer_tag, latest_tag, newer_release};
