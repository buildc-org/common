pub mod table;
pub mod tree;

pub(crate) const PICK: &str = "up and down to move, enter to choose, esc cancels";
pub(crate) const TOGGLE: &str = "space toggles, enter confirms, esc cancels";
pub(crate) const EITHER: &str = "up and down to move, enter to answer";
pub(crate) const EITHER_CANCEL: &str = "up and down to move, enter to answer, esc cancels";
pub const SIDE: &str = "left and right to move, enter to answer";
pub(crate) const NEST: &str = "filter, space toggles, ←/→ opens, enter confirms";
pub(crate) const SUB_KEYS: &str =
    "\u{2191}\u{2193} navigate \u{2022} \u{23ce}  select \u{2022} Esc back";
pub(crate) const LINE_KEYS: &str = "enter confirms, esc cancels";
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
