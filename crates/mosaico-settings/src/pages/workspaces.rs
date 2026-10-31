//! The Workspaces page: how workspaces switch and what each one tiles
//! with, from `config.toml`.

use super::{Entry, Section};

pub const SECTIONS: &[Section] = &[
    (
        "Switching",
        &[
            Entry::Setting("workspaces.mode"),
            Entry::Setting("workspaces.wrap"),
        ],
    ),
    (
        "Layouts",
        &[
            Entry::Setting("workspaces.layouts.1"),
            Entry::Setting("workspaces.layouts.2"),
            Entry::Setting("workspaces.layouts.3"),
            Entry::Setting("workspaces.layouts.4"),
            Entry::Setting("workspaces.layouts.5"),
            Entry::Setting("workspaces.layouts.6"),
            Entry::Setting("workspaces.layouts.7"),
            Entry::Setting("workspaces.layouts.8"),
        ],
    ),
];
