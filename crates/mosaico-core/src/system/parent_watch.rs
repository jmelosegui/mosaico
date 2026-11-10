//! Hearing when the process that started this one exits.

/// The process that started this one, watched so a helper it started ends
/// with it rather than lingering.
///
/// # Contract for platform implementations
///
/// - The watch is taken on the parent as it was when the watch was made:
///   a later process that reuses its id is never mistaken for it.
/// - `pid` is the parent's process id, or `None` when it cannot be told,
///   such as when it exited before the watch was made.
/// - `wait` blocks until the parent exits, and returns at once if it had
///   exited before the watch was made. When the parent cannot be watched
///   at all, it never returns, so the helper simply keeps running.
/// - Both may be called from any thread, and `wait` from several at once.
pub trait ParentWatch: Send + Sync {
    /// The parent's process id, if it can be told.
    fn pid(&self) -> Option<u32>;

    /// Blocks until the parent exits.
    fn wait(&self);
}
