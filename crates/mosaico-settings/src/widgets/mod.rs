//! The controls of the settings window, styled like the ArtCraft apps.

mod color;
mod fields;
#[expect(dead_code, reason = "the page redesigns use these")]
mod marks;
mod row;
mod segmented;
mod slider;
#[expect(dead_code, reason = "the page redesigns use these")]
mod tabs;
mod toggle;

pub use color::{ColorPicker, color};
pub use fields::{button, dropdown, number, text};
#[expect(unused_imports, reason = "the page redesigns use these")]
pub use marks::{badge, checkbox, chip, icon_button, keycaps};
#[expect(unused_imports, reason = "the page redesigns use these")]
pub use row::card;
pub use row::{RowState, divider, heading, row, section};
pub use segmented::segmented;
pub use slider::slider;
#[expect(unused_imports, reason = "the page redesigns use these")]
pub use tabs::tabs;
pub use toggle::toggle;
