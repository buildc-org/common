use crate::ui::chrome::*;
use ratatui::style::{Color, Style};
use ratatui::text::Span;

#[derive(Clone)]
pub struct Cell {
    pub(crate) text: String,
    pub(crate) set: bool,
}

impl Cell {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            set: false,
        }
    }

    pub fn set(text: &str) -> Self {
        Self {
            text: text.to_string(),
            set: true,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn answered(&self) -> bool {
        self.set
    }
}

pub(crate) fn column_widths(headings: &[&str], rows: &[Vec<Cell>], room: usize) -> Vec<usize> {
    let columns = rows
        .iter()
        .map(Vec::len)
        .max()
        .unwrap_or(0)
        .max(headings.len());
    let mut widths: Vec<usize> = (0..columns)
        .map(|column| {
            let cells = rows
                .iter()
                .filter_map(|row| row.get(column))
                .map(|cell| cell.text.chars().count())
                .max()
                .unwrap_or(0);
            cells.max(
                headings
                    .get(column)
                    .map_or(0, |heading| heading.chars().count()),
            )
        })
        .collect();
    let room = room.saturating_sub(2 * columns.saturating_sub(1));
    while widths.iter().sum::<usize>() > room {
        let Some(widest) = widths
            .iter()
            .enumerate()
            .max_by_key(|(_, width)| **width)
            .map(|(at, _)| at)
        else {
            break;
        };
        if widths[widest] == 0 {
            break;
        }
        widths[widest] -= 1;
    }
    widths
}

pub(crate) fn value_cells(rows: &[Vec<Cell>]) -> Vec<Vec<Cell>> {
    rows.iter()
        .map(|row| row.iter().skip(1).cloned().collect())
        .collect()
}

pub(crate) fn table_headings(headings: &[&str], widths: &[usize]) -> String {
    headings
        .iter()
        .enumerate()
        .map(|(column, heading)| {
            let width = widths.get(column).copied().unwrap_or(0);
            let text: String = clipped(heading, width);
            format!("{text:<width$}")
        })
        .collect::<Vec<String>>()
        .join("  ")
}

pub(crate) fn clipped(text: &str, width: usize) -> String {
    if text.chars().count() <= width {
        return text.to_string();
    }
    match width < 3 {
        true => text.chars().take(width).collect(),
        false => format!("{}...", text.chars().take(width - 3).collect::<String>()),
    }
}

pub(crate) fn table_row_spans(
    row: &[Cell],
    first_width: usize,
    widths: &[usize],
    hot: bool,
) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let Some(first) = row.first() else {
        return spans;
    };
    let text = clipped(&first.text, first_width);
    let branch: String = text
        .chars()
        .take_while(|c| matches!(c, '\u{251c}' | '\u{2514}' | '\u{2500}' | '\u{2502}' | ' '))
        .collect();
    let device: String = text.chars().skip(branch.chars().count()).collect();
    spans.push(Span::styled(branch.clone(), Style::new().dim()));
    let ink = match (hot, first.set) {
        (true, _) => Style::new().fg(HIGHLIGHT).bold(),
        (false, true) => Style::new().fg(Color::White),
        (false, false) => Style::new().dim(),
    };
    spans.push(Span::styled(
        format!(
            "{device:<rest$}",
            rest = first_width.saturating_sub(branch.chars().count())
        ),
        ink,
    ));
    for (column, cell) in row.iter().enumerate().skip(1) {
        let width = widths.get(column - 1).copied().unwrap_or(0);
        let text: String = clipped(&cell.text, width);
        let style = match cell.set {
            true => Style::new().fg(Color::White),
            false => Style::new().dim(),
        };
        spans.push(Span::raw("  "));
        spans.push(Span::styled(format!("{text:<width$}"), style));
    }
    spans
}
