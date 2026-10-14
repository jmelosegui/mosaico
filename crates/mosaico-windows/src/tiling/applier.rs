//! Applies the tiler's effects through Win32.
//!
//! Most effects are applied as they arrive. Two kinds are held back
//! while the user is still pressing keys, and only the latest of each
//! is applied once input goes quiet:
//!
//! - focus changes, because `SetForegroundWindow` is a cross-process
//!   call that often takes 50-150 ms;
//! - coalesced placements from moves, so a burst of moves repaints each
//!   window once. A window's border and cursor moves wait with it, so
//!   the border never shows at a slot the window has not reached yet.
//!
//! Every applied effect is recorded in [`Expectations`] so the events it
//! causes can be told apart from the user's.

use std::collections::HashMap;
use std::time::Instant;

use mosaico_core::config::{BorderColors, HidingBehaviour};
use mosaico_core::tiler::{BorderStyle, Coalescer, Effect, Expectations, HiddenWindows};
use mosaico_core::window::Window as _;
use mosaico_core::{Rect, WindowId};
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, SetCursorPos, WM_CLOSE};

use crate::border::{Border, Color};
use crate::frame;
use crate::window::Window;

use super::hidden_file::{self, HiddenFile};

/// Border color used when a configured value does not parse. Matches
/// the historical default focused color.
const FALLBACK_COLOR: Color = Color {
    r: 0,
    g: 0xB4,
    b: 0xD8,
};

/// Executes effects and remembers what the OS should report back.
#[derive(Default)]
pub(super) struct Applier {
    /// One overlay per bordered window, created on first use.
    borders: HashMap<WindowId, Border>,
    /// Effects held back until input goes quiet.
    held: Coalescer,
    expectations: Expectations,
    /// Windows hidden by workspace switching, so the next daemon can
    /// show them again if this one is killed.
    hidden: HiddenWindows,
    /// Where `hidden` is kept, if the instance has a data directory.
    hidden_file: Option<HiddenFile>,
}

impl Applier {
    /// An applier that keeps its record of hidden windows in `file`.
    pub(super) fn with_hidden_file(file: Option<HiddenFile>) -> Self {
        Self {
            hidden_file: file,
            ..Self::default()
        }
    }

    /// Shows again the windows a previous daemon hid and never showed,
    /// because it was killed or crashed, then clears its record.
    ///
    /// Run before the first windows are enumerated, so they are adopted
    /// like any other window.
    pub(super) fn restore_previous(&mut self, now: Instant) {
        let Some(file) = &self.hidden_file else {
            return;
        };
        let effects = file.load().restore(hidden_file::identity);
        if !effects.is_empty() {
            mosaico_core::log_info!(
                "showing {} window(s) a previous daemon left hidden",
                effects.len()
            );
        }
        for effect in &effects {
            self.run(effect, now);
        }
        self.save_hidden();
    }

    /// Forgets a window that was destroyed.
    pub(super) fn forget(&mut self, window: WindowId) {
        if self.hidden.forget(window) {
            self.save_hidden();
        }
    }

    fn save_hidden(&self) {
        if let Some(file) = &self.hidden_file {
            file.save(&self.hidden);
        }
    }

    /// Applies effects in order, holding back the ones that may wait.
    pub(super) fn apply(&mut self, effects: Vec<Effect>, now: Instant) {
        for effect in effects {
            if matches!(effect, Effect::Focus { .. }) {
                self.expectations.held(&effect);
            }
            if let Some(effect) = self.held.admit(effect) {
                self.run(&effect, now);
            }
        }
    }

    /// Whether anything is being held back.
    pub(super) fn has_pending(&self) -> bool {
        !self.held.is_empty()
    }

    /// Applies everything held back.
    pub(super) fn flush(&mut self, now: Instant) {
        for effect in self.held.take() {
            self.run(&effect, now);
        }
    }

    /// Why an event is an echo of our own effects, if it is one.
    pub(super) fn echo(
        &mut self,
        event: &mosaico_core::WindowEvent,
        now: Instant,
    ) -> Option<&'static str> {
        self.expectations.echo(event, now)
    }

    fn run(&mut self, effect: &Effect, now: Instant) {
        match effect {
            Effect::Place { window, rect, .. } => place(*window, rect),
            Effect::PrePlace { window, rect } => pre_place(*window, rect),
            Effect::Hide { window, how } => {
                let win = Window::from_id(*window);
                match how {
                    HidingBehaviour::Cloak => win.cloak(),
                    HidingBehaviour::Hide => win.hide(),
                    HidingBehaviour::Minimize => win.minimize(),
                }
            }
            Effect::Show { window, how } => {
                let win = Window::from_id(*window);
                match how {
                    HidingBehaviour::Cloak => win.uncloak(),
                    HidingBehaviour::Hide | HidingBehaviour::Minimize => win.show(),
                }
            }
            Effect::Reveal { window } => {
                let win = Window::from_id(*window);
                win.uncloak();
                win.force_show();
            }
            Effect::Focus {
                window,
                move_cursor,
            } => {
                Window::from_id(*window).set_foreground();
                if *move_cursor {
                    center_cursor(*window);
                }
            }
            Effect::CenterCursor { window } => center_cursor(*window),
            Effect::Close { window } => close(*window),
            Effect::Minimize { window } => Window::from_id(*window).minimize(),
            Effect::Repaint { window } => Window::from_id(*window).invalidate(),
            Effect::Decorate { window, corners } => {
                frame::set_corner_preference(Window::from_id(*window).hwnd(), *corners);
            }
            Effect::Undecorate { window } => {
                frame::reset_corner_preference(Window::from_id(*window).hwnd());
            }
            Effect::Border { window, style } => self.draw_border(*window, style.as_ref()),
            Effect::DropBorder { window } => {
                // Dropping destroys the overlay window.
                self.borders.remove(window);
            }
        }
        self.expectations.applied(effect, now);
        if self.hidden.applied(effect, hidden_file::identity) {
            self.save_hidden();
        }
    }

    fn draw_border(&mut self, window: WindowId, style: Option<&BorderStyle>) {
        let Some(style) = style else {
            if let Some(border) = self.borders.get(&window) {
                border.hide();
            }
            return;
        };
        let border = match self.borders.get(&window) {
            Some(b) => b,
            None => match Border::new() {
                Ok(b) => self.borders.entry(window).or_insert(b),
                Err(e) => {
                    mosaico_core::log_warn!("border.create failed for {}: {}", window, e);
                    return;
                }
            },
        };
        border.show(
            &style.rect,
            parse_color(&style.color),
            style.width,
            style.radius,
            Window::from_id(window).hwnd(),
        );
    }
}

/// Moves a window into its slot, unless it is already there.
///
/// Maximized windows are always moved: positioning one clears its
/// maximized state, which moving it alone does not.
fn place(window: WindowId, rect: &Rect) {
    let win = Window::from_id(window);
    if !win.is_maximized() && win.rect().ok().as_ref() == Some(rect) {
        return;
    }
    if let Err(e) = win.set_rect(rect) {
        mosaico_core::log_warn!("Failed to position window {}: {}", window, e);
    }
    win.invalidate();
}

/// Moves a window that has not been shown yet, without blocking.
///
/// These windows have been created but not shown, and some belong to
/// threads that never pump messages, so a synchronous `SetWindowPos`
/// would never return and would take the daemon down with it.
fn pre_place(window: WindowId, rect: &Rect) {
    if let Err(e) = Window::from_id(window).move_async(rect) {
        mosaico_core::log_debug!("~pre {} move failed: {}", window, e);
    }
}

/// Puts the cursor at the center of the window.
fn center_cursor(window: WindowId) {
    let Ok(rect) = Window::from_id(window).rect() else {
        return;
    };
    // SAFETY: SetCursorPos only takes screen coordinates.
    unsafe {
        let _ = SetCursorPos(rect.x + rect.width / 2, rect.y + rect.height / 2);
    }
}

/// Sends `WM_CLOSE` so the window can shut down gracefully.
fn close(window: WindowId) {
    // SAFETY: PostMessageW only queues a message; an invalid handle makes
    // it fail without side effects.
    unsafe {
        let _ = PostMessageW(
            Some(Window::from_id(window).hwnd()),
            WM_CLOSE,
            WPARAM(0),
            LPARAM(0),
        );
    }
}

fn parse_color(hex: &str) -> Color {
    if hex == BorderColors::NONE {
        return FALLBACK_COLOR;
    }
    Color::from_hex(hex).unwrap_or(FALLBACK_COLOR)
}
