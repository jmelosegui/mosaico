//! Carrying out an effect the applier lets through, and recording it.

use std::time::Instant;

use super::super::{Effect, Platform, apply_decoration_effect, apply_window_effect};
use super::Applier;

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
                self.place_now(window, rect, platform);
            }
            Effect::Place { window, .. } => {
                self.forget_stopped(window);
                self.run_place(effect, platform, now);
            }
            // What ends a motion, effect by effect, is in `applier_ends.rs`.
            Effect::Hide { window, .. } | Effect::Minimize { window } => {
                self.stop_motion(window);
                apply_window_effect(effect, platform.desktop(), platform.control());
            }
            Effect::Reveal { window } => {
                self.finish_motion(window, platform);
                apply_window_effect(effect, platform.desktop(), platform.control());
            }
            Effect::Focus {
                window,
                move_cursor: true,
            }
            | Effect::CenterCursor { window }
                if self.is_moving_window(window) =>
            {
                self.point_at_target(effect, window, platform.control());
            }
            Effect::PrePlace { .. }
            | Effect::Show { .. }
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
                self.let_go(window, platform);
            }
            // The borders of moving windows are in `applier_borders.rs`. One a
            // display change stopped keeps its style the same way, to be drawn
            // once it is placed after the rebuild.
            Effect::Border { window, ref style }
                if self.is_moving_window(window) || self.is_stopped(window) =>
            {
                self.border_of_moving(window, style, platform.decorations());
            }
            Effect::DropBorder { window } => {
                self.forget_kept_border(window);
                apply_decoration_effect(effect, platform.decorations());
            }
            Effect::Decorate { .. }
            | Effect::Undecorate { .. }
            | Effect::Border { .. }
            | Effect::RedrawBorders => apply_decoration_effect(effect, platform.decorations()),
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
