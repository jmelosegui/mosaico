//! What the daemon tells `mosaico doctor` about the bar API: the provider
//! it runs, and each session with the space it keeps clear.

use serde::{Deserialize, Serialize};

use super::Reserve;

/// The bar API as the daemon serves it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BarStatus {
    /// The provider the daemon runs, by the name `[bar] provider` gives it.
    /// It is the one in the file unless a reload of the file failed.
    pub provider: String,
    /// The sessions open, then those that ended and keep their space a
    /// moment for a bar that starts again, each in the order it opened.
    #[serde(default)]
    pub sessions: Vec<BarSession>,
}

/// One session of the bar API.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BarSession {
    /// The session's number, counted from 1 since the daemon started.
    pub session: u64,
    /// The program its `hello` named, such as `"mosaico subscribe"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client: Option<String>,
    /// The bar its `hello` named, such as `"yasb"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bar: Option<String>,
    /// The process that draws the bar, if known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process: Option<u32>,
    /// Whether the session ended. Its space is then kept until the grace
    /// runs out.
    #[serde(default)]
    pub ended: bool,
    /// Whether the provider lets it reserve space.
    #[serde(default)]
    pub may_reserve: bool,
    /// The space it keeps clear of tiles, one entry per monitor it named.
    #[serde(default)]
    pub reserves: Vec<Reserve>,
}
