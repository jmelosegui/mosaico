//! Random states for property tests, from a seed, with no dependency.

use std::collections::BTreeMap;

use crate::{StableMonitorId, WindowId};

use super::{Daemon, Focus, FocusedWindow, Monitor, Rect, State, Theme, Workspace};

/// A small seeded generator (xorshift64*), so a failing case can be
/// replayed from its seed.
pub struct Gen(u64);

impl Gen {
    pub fn new(seed: u64) -> Self {
        Self(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// A number below `n`.
    pub fn below(&mut self, n: u64) -> u64 {
        self.next() % n.max(1)
    }

    fn flip(&mut self) -> bool {
        self.below(2) == 0
    }

    fn pick<'a>(&mut self, items: &[&'a str]) -> &'a str {
        items[self.below(items.len() as u64) as usize]
    }

    fn rect(&mut self) -> Rect {
        Rect {
            x: self.below(3) as i32 * 1920 - 1920,
            y: self.below(2) as i32 * 40,
            width: 1920,
            height: 1080 - self.below(2) as i32 * 40,
        }
    }

    /// A state with up to four of the monitors `m-0` to `m-5`.
    pub fn state(&mut self) -> State {
        let mut ids: Vec<u64> = (0..6).filter(|_| self.below(3) == 0).take(4).collect();
        if ids.is_empty() && self.flip() {
            ids.push(self.below(6));
        }
        let monitors: Vec<Monitor> = (ids.iter().enumerate())
            .map(|(index, id)| self.monitor(*id, index as u32))
            .collect();
        let focused = monitors.get(self.below(4) as usize).map(|m| m.id.clone());
        State {
            daemon: Daemon {
                version: self.pick(&["0.14.0", "0.15.0"]).into(),
                api: 1,
                instance: self.flip().then(|| "test".into()),
                provider: self.pick(&["mosaico", "external", "none"]).into(),
                paused: self.flip(),
                update: self.flip().then(|| "v0.15.0 available".into()),
                workspace_mode: self.pick(&["per-monitor", "global"]).into(),
                wrap: self.flip(),
            },
            theme: Theme {
                name: "catppuccin".into(),
                flavor: self.pick(&["mocha", "latte"]).into(),
                colors: BTreeMap::from([(
                    "blue".into(),
                    self.pick(&["#89b4fa", "#1e66f5"]).into(),
                )]),
            },
            focus: Focus {
                monitor: focused,
                workspace: self.flip().then(|| 1 + self.below(8) as u32),
                window: self.flip().then(|| FocusedWindow {
                    id: WindowId::from_raw(self.below(5)),
                    title: self
                        .pick(&["", "notes", "a \"quoted\" <b>title</b>"])
                        .into(),
                    program: self.pick(&["", "code.exe"]).into(),
                }),
            },
            monitors,
        }
    }

    fn monitor(&mut self, id: u64, index: u32) -> Monitor {
        let workspaces = (1..=1 + self.below(3) as u32)
            .map(|n| self.workspace(n))
            .collect();
        Monitor {
            id: StableMonitorId::new(format!("m-{id}")),
            index,
            name: self.pick(&["", "DELL U2720Q"]).into(),
            device: format!("DISPLAY{id}"),
            scale: [1.0, 1.25, 1.5][self.below(3) as usize],
            bounds: self.rect(),
            work_area: self.rect(),
            tiling_area: self.rect(),
            maximized: self.flip(),
            active_workspace: 1 + self.below(3) as u32,
            workspaces,
        }
    }

    fn workspace(&mut self, number: u32) -> Workspace {
        let windows: Vec<WindowId> = (0..self.below(4))
            .map(|_| WindowId::from_raw(self.below(9)))
            .collect();
        Workspace {
            number,
            name: number.to_string(),
            layout: self.pick(&["bsp", "vertical-stack", "three-column"]).into(),
            monocle: self.flip(),
            focused: windows.first().copied().filter(|_| self.flip()),
            windows,
        }
    }
}
