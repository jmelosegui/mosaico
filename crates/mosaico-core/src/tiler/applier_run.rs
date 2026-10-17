//! Carrying out an effect the applier lets through, and recording it.

use std::time::Instant;

use crate::motion::{Ending, Stop};

use super::super::{Effect, Platform, apply_decoration_effect, apply_window_effect};
use super::Applier;
use super::shows::left_by;

impl Applier {
    /// Carries out `effect` through the platform, and records it.
    pub(super) fn run(&mut self, effect: &Effect, platform: &mut dyn Platform, now: Instant) {
        match *effect {
            // A move that does not wait may land before the window's
            // placement comes, and a placement that finds its window in
            // place is skipped, repaint and all. Nothing has repainted a
            // window moved that way while on screen, so its next placement
            // is made in full, where it would have been made anyway.
            Effect::PrePlace { window, .. } if platform.desktop().is_visible(window) => {
                self.moved_on_screen.insert(window);
                apply_window_effect(effect, platform.desktop(), platform.control());
            }
            Effect::Place { window, rect, .. } if self.moved_on_screen.remove(&window) => {
                self.place_now(window, rect, platform, now);
            }
            Effect::Place { window, .. } => {
                self.forget_stopped(window);
                self.run_place(effect, platform, now);
            }
            // What ends a motion, effect by effect, is in `applier_ends.rs`,
            // and what is kept of a change on its way in `applier_posted.rs`.
            Effect::Hide { window, .. } | Effect::Minimize { window } => {
                let stop = match effect {
                    Effect::Minimize { .. } => Stop::Minimized,
                    _ => Stop::Hidden,
                };
                self.stop_motion(window, Ending::Stopped(stop), now);
                apply_window_effect(effect, platform.desktop(), platform.control());
                self.show_state(window, left_by(effect), platform);
            }
            Effect::Reveal { window } | Effect::Show { window, .. } => {
                if let Effect::Reveal { .. } = effect {
                    self.finish_motion(window, Ending::Revealed, platform, now);
                }
                let minimized = platform.desktop().is_minimized(window);
                self.expectations.showing(window, minimized);
                apply_window_effect(effect, platform.desktop(), platform.control());
                self.show_state(window, left_by(effect), platform);
            }
            // While the user drags a window the pointer is theirs: it goes
            // where it was asked once the drag ends (`applier_drag.rs`).
            Effect::Focus {
                window,
                move_cursor: true,
            }
            | Effect::CenterCursor { window }
                if self.holds_pointer() =>
            {
                self.owe_pointer(window);
                if let Effect::Focus { .. } = effect {
                    let move_cursor = false;
                    let focus = Effect::Focus {
                        window,
                        move_cursor,
                    };
                    apply_window_effect(&focus, platform.desktop(), platform.control());
                }
            }
            Effect::Focus {
                window,
                move_cursor: true,
            }
            | Effect::CenterCursor { window }
                if self.aim(window, platform.control()).is_some() =>
            {
                self.point_at_target(effect, window, platform.control());
            }
            Effect::PrePlace { .. }
            | Effect::Focus { .. }
            | Effect::CenterCursor { .. }
            | Effect::Close { .. }
            | Effect::Repaint { .. }
            | Effect::PutBehind { .. } => {
                apply_window_effect(effect, platform.desktop(), platform.control())
            }
            // Admitting it dropped what was held for the window. One let go
            // of while it moves ends in its slot, without its border.
            Effect::Untile { window } => {
                self.forget_stopped(window);
                self.let_go(window, platform, now);
            }
            // The borders of moving windows are in `applier_borders.rs`. One a
            // display change stopped keeps its style the same way, to be drawn
            // once it is placed after the rebuild.
            Effect::Border { window, ref style }
                if self.is_moving_window(window) || self.is_stopped(window) =>
            {
                self.border_of_moving(window, style, platform.decorations());
            }
            // One still to land where a posted placement takes it has the
            // style drawn once it lands, around where it was placed
            // (`applier_landing.rs`). A border to be hidden is hidden at once.
            Effect::Border { window, ref style } if self.is_landing(window) => {
                self.defer_border(window, style.clone());
                if style.is_none() {
                    apply_decoration_effect(effect, platform.decorations());
                }
            }
            Effect::Border { .. } | Effect::DropBorder { .. } => {
                if let Effect::DropBorder { window } = *effect {
                    self.forget_kept_border(window);
                    self.forget_deferred_border(window);
                }
                apply_decoration_effect(effect, platform.decorations());
            }
            Effect::Decorate { .. } | Effect::Undecorate { .. } | Effect::RedrawBorders => {
                apply_decoration_effect(effect, platform.decorations())
            }
        }
        self.expectations.applied(effect, now);
        if self
            .hidden
            .applied(effect, |w| platform.desktop().identity(w))
        {
            self.save_hidden();
        }
    }
}
