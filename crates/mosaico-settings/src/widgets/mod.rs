//! The controls of the settings window, styled like the ArtCraft apps.

mod color;
mod color_button;
mod color_hex;
mod color_popup;
mod fields;
mod icon_dropdown;
mod marks;
mod row;
mod segmented;
mod slider;
mod tabs;
mod toggle;

pub use color::{ColorPicker, color};
pub use fields::{button, dimmed_dropdown, dropdown, number, number_of_width, text};
pub use icon_dropdown::{Choice, icon_dropdown};
#[expect(unused_imports, reason = "the page redesigns use these")]
pub use marks::{badge, chip, keycaps};
pub use marks::{checkbox, ghost_button, icon_button};
pub use row::{RowState, card, divider, heading, row, section};
pub use segmented::segmented;
pub use slider::slider;
pub use tabs::tabs;
pub use toggle::toggle;
