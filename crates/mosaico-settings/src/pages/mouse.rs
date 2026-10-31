//! The Mouse page: how the pointer and focus follow each other, and what
//! dragging does, from `config.toml`.

use super::{Entry, Section};

pub const SECTIONS: &[Section] = &[
    (
        "Focus",
        &[
            Entry::Setting("mouse.follows_focus"),
            Entry::Setting("mouse.focus_follows_mouse"),
        ],
    ),
    (
        "Dragging",
        &[
            Entry::Setting("mouse.drag_to_swap"),
            Entry::Setting("mouse.resize_splits"),
        ],
    ),
];
