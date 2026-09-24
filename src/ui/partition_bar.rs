use crate::ui::chrome::ALERT;
use crate::ui::progress::{blend, TRACK};
use ratatui::style::{Color, Style};
use ratatui::text::Span;
use std::ops::Range;

#[derive(Debug, PartialEq)]
pub struct Hole {
    pub before: Option<String>,
    pub after: Option<String>,
    pub gb: u64,
}

pub(crate) const NEIGHBOUR: Color = Color::Rgb(0xa0, 0xa0, 0xa0);

pub(crate) fn placed(holes: &[Hole], offset: u64, size: u64, width: usize) -> Vec<Span<'static>> {
    let end = offset.saturating_add(size);
    let Some((hole, fits)) = chosen(holes, offset, size) else {
        return Vec::new();
    };
    let before = hole.before.as_deref().map(|label| boxed(label, width / 4));
    let after = hole.after.as_deref().map(|label| boxed(label, width / 4));
    let taken = [&before, &after]
        .iter()
        .flat_map(|side| side.iter())
        .map(Vec::len)
        .sum::<usize>();
    let room = width.saturating_sub(taken);
    let cell = |gb: u64| {
        let gb = u128::from(gb.min(hole.gb));
        (gb * room as u128 / u128::from(hole.gb.max(1))) as usize
    };
    let mut spans = before.unwrap_or_default();
    if size == 0 {
        spans.extend(painted(0..room, &said(hole.gb), |_| TRACK));
    } else {
        let from = cell(offset).min(room.saturating_sub(1));
        let to = cell(end).max(from + 1).min(room);
        let partition = |at| match fits {
            true => blend(at, room),
            false => ALERT,
        };
        spans.extend(painted(0..from, &said(offset), |_| TRACK));
        spans.extend(painted(from..to, &said(size), partition));
        spans.extend(painted(
            to..room,
            &said(hole.gb.saturating_sub(end)),
            |_| TRACK,
        ));
    }
    spans.extend(after.unwrap_or_default());
    spans
}

pub(crate) fn chosen(holes: &[Hole], offset: u64, size: u64) -> Option<(&Hole, bool)> {
    let end = offset.saturating_add(size);
    match holes.iter().find(|hole| end <= hole.gb) {
        Some(hole) => Some((hole, true)),
        None => holes
            .iter()
            .max_by_key(|hole| hole.gb)
            .map(|hole| (hole, false)),
    }
}

fn boxed(label: &str, most: usize) -> Vec<Span<'static>> {
    let width = (label.chars().count() + 2).min(most);
    let label: String = label.chars().take(width.saturating_sub(2)).collect();
    painted(0..width, &label, |_| NEIGHBOUR)
}

pub(crate) fn said(gb: u64) -> String {
    format!("{gb}.0 GB")
}

fn painted(cells: Range<usize>, text: &str, colour: impl Fn(usize) -> Color) -> Vec<Span<'static>> {
    let letters: Vec<char> = match text.chars().count() + 2 <= cells.len() {
        true => text.chars().collect(),
        false => Vec::new(),
    };
    let lead = cells.start + (cells.len() - letters.len()) / 2;
    cells
        .map(
            |at| match at.checked_sub(lead).and_then(|index| letters.get(index)) {
                Some(letter) => Span::styled(
                    letter.to_string(),
                    Style::new().fg(Color::Black).bg(colour(at)),
                ),
                None => Span::styled("\u{2588}", Style::new().fg(colour(at))),
            },
        )
        .collect()
}
