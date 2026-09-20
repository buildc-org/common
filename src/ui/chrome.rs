use crate::ui::progress::*;
use crate::ui::term::*;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, ListItem, Padding};
use ratatui::Frame;

pub(crate) const GOOD: Color = Color::Rgb(0x6c, 0xe0, 0x7b);
pub(crate) const AMBER: Color = Color::Rgb(0xff, 0xbf, 0x00);
pub(crate) const ALERT: Color = Color::Rgb(0xff, 0x5c, 0x57);

pub(crate) const WIDEST: u16 = 92;

pub const PANEL_ROOM: usize = (if WIDEST < NARROWEST as u16 {
    WIDEST
} else {
    NARROWEST as u16
} - 2
    - 2 * PAD_X) as usize;

pub const ROW_ROOM: usize = PANEL_ROOM - 2;

// A terminal cell is about twice as tall as it is wide, so a square frame takes half as many rows as columns.
const CELL: (u16, u16) = (8, 16);

pub(crate) const ACCENT: Color = Color::Rgb(COLD.0, COLD.1, COLD.2);
pub(crate) const HIGHLIGHT: Color = Color::Rgb(HOT.0, HOT.1, HOT.2);

pub(crate) const PAD_X: u16 = 3;
pub(crate) const PAD_Y: u16 = 1;

thread_local! {
    pub(crate) static OVERLAY: std::cell::Cell<Option<(u16, u16)>> = const {
        std::cell::Cell::new(None)
    };
    pub(crate) static BACKDROP: std::cell::RefCell<Option<Buffer>> = const {
        std::cell::RefCell::new(None)
    };
    static OVERLAY_TITLE: std::cell::RefCell<Option<String>> = const {
        std::cell::RefCell::new(None)
    };
    pub(crate) static REVEAL: std::cell::Cell<bool> = const {
        std::cell::Cell::new(false)
    };
}

pub fn in_overlay<T>(
    width: u16,
    height: u16,
    body: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    OVERLAY.with(|overlay| overlay.set(Some((width, height))));
    let done = body();
    OVERLAY.with(|overlay| overlay.set(None));
    done
}

pub fn in_titled_overlay<T>(
    width: u16,
    height: u16,
    title: &str,
    body: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    OVERLAY_TITLE.with(|held| *held.borrow_mut() = Some(title.to_string()));
    let done = in_overlay(width, height, body);
    OVERLAY_TITLE.with(|held| *held.borrow_mut() = None);
    done
}

pub(crate) fn chrome(
    frame: &mut Frame,
    title: Option<&str>,
    head: &str,
    rows: u16,
    keys: &str,
) -> Rect {
    if OVERLAY.with(std::cell::Cell::get).is_some() {
        if let Some(back) = BACKDROP.with(|saved| saved.borrow().clone()) {
            let area = frame.area();
            let width = area.width.min(back.area.width);
            let height = area.height.min(back.area.height);
            for y in 0..height {
                for x in 0..width {
                    frame.buffer_mut()[(x, y)] = back[(x, y)].clone();
                }
            }
        }
    }
    let held = OVERLAY_TITLE.with(|held| held.borrow().clone());
    let title = match OVERLAY.with(std::cell::Cell::get).is_some() {
        true => held.as_deref().or(title),
        false => title,
    };
    let Some(title) = title else {
        return frame.area();
    };
    let wording = match head.trim() {
        "" => title.trim(),
        head => head,
    };
    let full = frame.area();
    let (width, tall) = match OVERLAY.with(std::cell::Cell::get) {
        Some((width, height)) => (width.min(full.width), height.min(full.height)),
        None => {
            let width = WIDEST.min(full.width);
            let square = width.saturating_mul(CELL.0) / CELL.1;
            (width, rows.saturating_add(2 + 2 * PAD_Y).max(square))
        }
    };
    let box_area = centred(full, width, tall);
    let block = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(ACCENT))
        .padding(Padding::symmetric(PAD_X, PAD_Y))
        .title(Line::from(vec![
            Span::styled("\u{2500} ", Style::new().fg(ACCENT)),
            Span::styled(wording.to_string(), Style::new().bold()),
            Span::raw(" "),
        ]))
        .title_bottom(
            Line::from(vec![
                Span::styled("\u{2500} ", Style::new().fg(ACCENT)),
                Span::styled(keys.to_string(), Style::new().fg(TRACK)),
                Span::raw(" "),
            ])
            .left_aligned(),
        );
    let inside = block.inner(box_area);
    frame.render_widget(Clear, box_area);
    frame.render_widget(block, box_area);
    inside
}

pub(crate) fn hint_row(keys: &str) -> &str {
    match CHROME.get() {
        Some(_) => "",
        None => keys,
    }
}

pub(crate) fn head_row(question: &str) -> &str {
    match CHROME.get() {
        Some(_) => "",
        None => question,
    }
}

pub(crate) fn centred(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y + (area.height - height) / 2,
        width,
        height,
    }
}

pub(crate) fn set_in(body: Rect, items: &[ListItem], rows: usize) -> Rect {
    let width = items
        .iter()
        .map(ListItem::width)
        .max()
        .unwrap_or(0)
        .saturating_add(2) as u16;
    centred(body, width, rows.min(usize::from(body.height)) as u16)
}
