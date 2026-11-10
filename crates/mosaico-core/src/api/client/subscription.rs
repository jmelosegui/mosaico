//! What `subscribe` is asked to do, and where it reads and prints.

use std::io::{BufRead, Write};
use std::sync::Arc;
use std::time::Duration;

use crate::system::ParentWatch;

use super::{Hold, MonitorRef, Output};

/// What `subscribe` follows, prints and forwards.
#[derive(Debug, Clone, PartialEq)]
pub struct Subscription {
    /// The bar it speaks for, such as `yasb`.
    pub bar: Option<String>,
    /// The topics, or `None` for all of them.
    pub topics: Option<Vec<String>>,
    /// The only monitor to show, and the one requests act on when they
    /// name none.
    pub monitor: Option<MonitorRef>,
    /// What is printed.
    pub output: Output,
    /// Whether to wait for a daemon that is not running, and to connect
    /// again when it restarts.
    pub wait: bool,
    /// Whether requests are read from the input, one per line.
    pub requests: bool,
    /// With `requests`, how long the input may stay quiet before the
    /// command ends.
    pub idle: Duration,
    /// With `wait`, how often to try to connect.
    pub retry: Duration,
    /// The space held for the bar while the command runs.
    pub reserve: Vec<Hold>,
}

impl Default for Subscription {
    fn default() -> Self {
        Self {
            bar: None,
            topics: None,
            monitor: None,
            output: Output::Messages,
            wait: false,
            requests: false,
            idle: Duration::from_secs(15),
            retry: Duration::from_millis(500),
            reserve: Vec::new(),
        }
    }
}

/// Opens the input on the thread that reads it, so the input itself need
/// not move between threads, as a lock on a process's standard input
/// cannot.
pub type OpenInput = Box<dyn FnOnce() -> Box<dyn BufRead> + Send>;

/// Where `subscribe` reads requests, prints, and learns that the process
/// that started it exited.
pub struct Streams<'a> {
    /// The requests, one per line, opened and read only with `requests`.
    pub input: OpenInput,
    /// Where lines are printed.
    pub out: &'a mut dyn Write,
    /// The process that started this one, if it can be watched.
    pub parent: Option<Arc<dyn ParentWatch>>,
}
