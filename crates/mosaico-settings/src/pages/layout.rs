//! The Layout page: how windows are tiled and hidden, from `config.toml`.

use super::{Entry, Section};

pub const SECTIONS: &[Section] = &[
    (
        "Tiling",
        &[
            Entry::Setting("layout.default"),
            Entry::Setting("layout.gap"),
            Entry::Setting("layout.ratio"),
        ],
    ),
    ("Other workspaces", &[Entry::Setting("layout.hiding")]),
];
