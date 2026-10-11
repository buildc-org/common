pub mod table;
#[cfg(feature = "test-support")]
pub mod testing;
pub mod tree;

pub(crate) const PICK: &str =
    "\u{2191}\u{2193} navigate \u{2022} \u{23ce}  select \u{2022} Esc cancel";
pub(crate) const TOGGLE: &str =
    "\u{2191}\u{2193} navigate \u{2022} \u{2423} toggle \u{2022} \u{23ce}  confirm \u{2022} Esc cancel";
pub(crate) const EITHER: &str = "\u{2191}\u{2193} navigate \u{2022} \u{23ce}  confirm";
pub(crate) const EITHER_CANCEL: &str =
    "\u{2191}\u{2193} navigate \u{2022} \u{23ce}  confirm \u{2022} Esc cancel";
pub const SIDE: &str = "\u{2190}/\u{2192} select \u{2022} \u{23ce}  confirm \u{2022} Esc back";
pub(crate) const NEST: &str = "\u{2191}\u{2193} navigate \u{2022} \u{2190}/\u{2192} open \u{2022} \u{2423} toggle \u{2022} \u{23ce}  confirm \u{2022} Esc cancel";
pub(crate) const SUB_KEYS: &str =
    "\u{2191}\u{2193} navigate \u{2022} \u{23ce}  select \u{2022} Esc back";
pub(crate) const LINE_KEYS: &str = "\u{23ce}  confirm \u{2022} Esc cancel";
pub(crate) const NO_CHOICES: &str = "No available choices";

mod cells;
mod choose;
mod chrome;
mod form;
mod layout;
mod menu;
mod nest;
mod partition_bar;
mod progress;
mod review;
mod term;

pub use cells::Cell;
pub use choose::{
    choose, confirm, confirm_current, confirm_current_or_no, confirm_or_no, confirm_over, multi,
    select, select_current, Answer, Choice, WINDOW_WIDTH,
};
pub use chrome::{in_overlay, in_titled_overlay, PANEL_ROOM, ROW_ROOM};
pub use form::{form, opens_at, Field, Filled, HeaderLine};
pub use menu::MenuItem;
pub use partition_bar::Hole;
pub use progress::Progress;
pub use review::{decide, line, line_or_empty, offer_over, review};
pub use term::{bold, own_screen, width, INTERRUPTED};

#[cfg(test)]
mod tests;
