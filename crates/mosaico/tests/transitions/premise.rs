//! What the swap and drag tests need of the desktop before they swap or
//! drag.
//!
//! Notepad opens on whichever monitor it last closed on, and a window
//! joins the workspace on screen there. When the two notepads are not
//! alone and side by side on workspace 8, the swap or the drag is not the
//! one the tests mean, and their failure would look like a bug in what
//! they test.

use mosaico_core::{Rect, WindowId};

/// Fails the test, saying its setup is wrong, unless `state`, as
/// `debug state` prints it, shows workspace 8 on one monitor only,
/// holding the two `windows` and nothing else, and unless `before` has
/// them side by side: at the same y and height, at different x.
pub fn assert_alone_side_by_side(state: &str, windows: [WindowId; 2], before: [Rect; 2]) {
    let blocks = workspace_windows(state, 8);
    let [listed] = blocks.as_slice() else {
        let why = format!("workspace 8 holds windows on {} monitors", blocks.len());
        wrong_setup(&why, state);
    };
    let mut expected: Vec<String> = windows.iter().map(WindowId::to_string).collect();
    let mut listed = listed.clone();
    expected.sort();
    listed.sort();
    if listed != expected {
        let why = format!("workspace 8 holds {listed:?}, not just the notepads");
        wrong_setup(&why, state);
    }
    let [a, b] = before;
    if a.y != b.y || a.height != b.height || a.x == b.x {
        let why = format!("the notepads are not side by side: {before:?}");
        wrong_setup(&why, state);
    }
}

/// Fails the test for `why`, with the desktop `state` it found.
fn wrong_setup(why: &str, state: &str) -> ! {
    panic!("the test's setup is wrong, not what it tests: {why}\n{state}")
}

/// The ids of the windows `state` lists on workspace `n` of each monitor
/// that has windows there.
fn workspace_windows(state: &str, n: usize) -> Vec<Vec<String>> {
    let header = format!("  Workspace {n} (");
    let mut blocks = Vec::new();
    let mut lines = state.lines().peekable();
    while let Some(line) = lines.next() {
        if !line.starts_with(&header) {
            continue;
        }
        let mut ids = Vec::new();
        while let Some(window) = lines.next_if(|l| l.starts_with("    ")) {
            let id = window.split_whitespace().next().unwrap_or_default();
            ids.push(id.to_string());
        }
        blocks.push(ids);
    }
    blocks
}
