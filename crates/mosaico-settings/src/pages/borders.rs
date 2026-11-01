//! The Borders page: the line around windows, from `config.toml`.

use super::{Entry, Section};

pub const SECTIONS: &[Section] = &[
    (
        "Border",
        &[
            Entry::Setting("borders.width"),
            Entry::Setting("borders.corner_style"),
        ],
    ),
    (
        "Colors",
        &[
            Entry::Setting("borders.colors.focused"),
            Entry::Setting("borders.colors.monocle"),
            Entry::Setting("borders.colors.unfocused"),
        ],
    ),
];
