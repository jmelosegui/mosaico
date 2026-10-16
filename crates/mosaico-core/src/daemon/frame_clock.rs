//! The display frames the daemon loop moves windows on.

/// Wakes the daemon loop on display frames while it wants them, so the
/// tiler can move windows a step further through their transitions on
/// each.
///
/// The platform's loop asks for frames while anything moves, and hands
/// each frame message it takes to the [`Scheduler`](super::Scheduler) as
/// [`Message::Frame`](super::Message::Frame). A frame carries no time: the
/// frame step takes it from the platform when it runs.
///
/// # Contract for platform implementations
///
/// - While frames are wanted, a frame message reaches the loop about once
///   per refresh of the display, or about 60 times a second where the
///   platform cannot follow the display. While they are not, none is
///   sent, and the clock costs nothing: it does not wake up.
/// - At most one frame message waits in the loop's queue: the next one is
///   sent once the loop has taken the last one out, so a loop that is busy
///   finds one frame waiting, not a backlog.
/// - [`want_frames`](Self::want_frames) never blocks, and asking twice is
///   the same as asking once. A frame sent just before frames were turned
///   off may still arrive.
pub trait FrameClock {
    /// Starts the frames, or stops them.
    fn want_frames(&self, on: bool);
}
