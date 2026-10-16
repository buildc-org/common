use crate::ui::cells::*;
use crate::ui::chrome::*;
use crate::ui::form::*;
use crate::ui::partition_bar::{chosen, placed, said};
use crate::ui::progress::blend;
use crate::ui::table;
use crate::ui::NO_CHOICES;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

const CARET_STEPS: usize = 8;

pub(crate) fn alerted(
    fields: &[Field],
    visited: &[usize],
    invalid: impl Fn(&[Field]) -> Vec<usize>,
) -> Vec<bool> {
    let mut flags = vec![false; fields.len()];
    let named = invalid(fields);
    if !named.is_empty() && named.iter().all(|row| visited.contains(row)) {
        for row in named {
            if let Some(flag) = flags.get_mut(row) {
                *flag = true;
            }
        }
    }
    flags
}

#[expect(
    clippy::too_many_arguments,
    reason = "each parameter is distinct state of the form screen"
)]
#[expect(
    clippy::too_many_lines,
    reason = "one pass lays out every form row and tracks the focus line and the tail"
)]
pub(crate) fn laid_out(
    fields: &[Field],
    visible: &[usize],
    cursor: usize,
    button: usize,
    mode: &Mode,
    actions: &[&str],
    blocked: Option<&str>,
    tried: bool,
    header: &[HeaderLine],
    sections: &[(usize, &str)],
) -> (Vec<Line<'static>>, usize, usize) {
    let typing = matches!(mode, Mode::Typing);
    let table_focused = matches!(mode, Mode::Table | Mode::TableMenu { .. });
    let open = match mode {
        Mode::Open(at) => Some(*at),
        Mode::Rows | Mode::Typing | Mode::Table | Mode::TableMenu { .. } => None,
    };
    let mut width = visible
        .iter()
        .map(|row| fields[*row].label().chars().count())
        .max()
        .unwrap_or(0);
    let labels = width;
    for row in visible {
        if let Field::Table { rows, .. } = &fields[*row] {
            for cells in rows {
                width = width.max(cells.first().map_or(0, |cell| cell.text.chars().count()));
            }
        }
    }
    for row in visible {
        if let Field::Table { widths, .. } = &fields[*row] {
            if !widths.is_empty() {
                let fixed: usize =
                    widths.iter().sum::<usize>() + 2 * widths.len().saturating_sub(1);
                width = form_room().saturating_sub(7 + fixed).max(1).max(labels);
            }
        }
    }
    let mut lines: Vec<Line<'static>> = Vec::new();
    for (at, line) in header.iter().enumerate() {
        match line {
            HeaderLine::Section(title) => {
                if at > 0 {
                    lines.push(Line::default());
                }
                lines.push(Line::from(Span::styled(
                    format!("  {title}"),
                    Style::new().fg(Color::White).bold(),
                )));
            }
            HeaderLine::Row(text) => {
                for line in table::wrap(text, form_room().saturating_sub(5)) {
                    lines.push(Line::from(Span::styled(
                        format!("     {line}"),
                        Style::new().dim(),
                    )))
                }
            }
            HeaderLine::Pair(label, value) => {
                lines.push(Line::from(vec![
                    Span::styled(format!("     {label:<width$}  "), Style::new().dim()),
                    Span::styled(value.clone(), Style::new()),
                ]));
            }
            HeaderLine::Good(label, value) | HeaderLine::Warn(label, value) => {
                let colour = match line {
                    HeaderLine::Good(..) => GOOD,
                    HeaderLine::Section(_)
                    | HeaderLine::Row(_)
                    | HeaderLine::Pair(..)
                    | HeaderLine::Warn(..)
                    | HeaderLine::Placement { .. } => AMBER,
                };
                lines.push(Line::from(vec![
                    Span::styled(format!("     {label:<width$}  "), Style::new().dim()),
                    Span::styled(value.clone(), Style::new().fg(colour)),
                ]));
            }
            HeaderLine::Placement {
                holes,
                offset,
                size,
                available,
            } => {
                let gb = |at: &usize| match fields.get(*at).map(Field::value) {
                    Some(typed) if !typed.is_empty() => typed.parse().unwrap_or(u64::MAX),
                    _ => 0,
                };
                lines.push(Line::from(placed(holes, gb(offset), gb(size), form_room())));
                if let Some((hole, _)) = chosen(holes, gb(offset), gb(size)) {
                    lines.push(Line::default());
                    lines.push(Line::from(vec![
                        Span::styled(format!("     {available:<width$}  "), Style::new().dim()),
                        Span::raw(said(hole.gb)),
                    ]));
                }
                if at + 1 < header.len() {
                    lines.push(Line::default());
                }
            }
        }
    }
    if !header.is_empty() {
        lines.push(Line::default());
    }
    let mut focus = 0usize;
    let mut rows_began = false;
    for (at, row) in visible.iter().enumerate() {
        if let Some((_, title)) = sections.iter().find(|(section, _)| section == row) {
            if rows_began {
                lines.push(Line::default());
            }
            lines.push(Line::from(Span::styled(
                format!("  {title}"),
                Style::new().fg(Color::White).bold(),
            )));
        }
        let field = &fields[*row];
        if matches!(field, Field::Gap) {
            lines.push(Line::default());
            rows_began = true;
            continue;
        }
        let here = at == cursor;
        let (done, glyph) = field.status();
        let status = match done {
            true => Style::new().fg(GOOD),
            false => Style::new().fg(AMBER),
        };
        let glyph = match OVERLAY.with(std::cell::Cell::get) {
            Some(_) => " ",
            None => glyph,
        };
        let label_style = match here {
            true if open.is_some() || table_focused => Style::new(),
            true => Style::new().fg(HIGHLIGHT).bold(),
            false if field.answerable() => Style::new(),
            false => Style::new().dim(),
        };
        if let Field::Table {
            label,
            column,
            headings,
            widths: fixed,
            rows,
            cursor: table_cursor,
            ..
        } = field
        {
            let heads: Vec<&str> = headings.iter().map(String::as_str).collect();
            let widths = match fixed.is_empty() {
                true => column_widths(
                    &heads,
                    &value_cells(rows),
                    form_room().saturating_sub(7 + width),
                ),
                false => fixed.clone(),
            };
            let field_at = lines.len();
            lines.push(Line::from(vec![
                Span::styled(
                    match (here, table_focused) {
                        (true, false) => "> ",
                        _ => "  ",
                    },
                    label_style,
                ),
                Span::styled(format!("{glyph}  "), status),
                Span::styled(format!("{label:<width$}"), label_style),
            ]));
            lines.push(Line::from(vec![
                Span::raw("     "),
                Span::styled(format!("{column:<width$}  "), Style::new().dim()),
                Span::styled(table_headings(&heads, &widths), Style::new().dim()),
            ]));
            let first = lines.len();
            for (n, cells) in rows.iter().enumerate() {
                let hot = table_focused && here && n == *table_cursor;
                let mut spans = vec![match hot {
                    true => Span::styled("   > ", Style::new().fg(HIGHLIGHT).bold()),
                    false => Span::raw("     "),
                }];
                spans.extend(table_row_spans(cells, width, &widths, hot));
                lines.push(Line::from(spans));
            }
            if here {
                focus = match rows.is_empty() {
                    true => field_at,
                    false => first + (*table_cursor).min(rows.len() - 1),
                };
            }
            rows_began = true;
            continue;
        }
        let field_at = lines.len();
        let editing = here && typing;
        let value = match editing {
            true => field.typing(),
            false => field.shown(),
        };
        let mut row = vec![
            Span::styled(
                match (here, open.is_some()) {
                    (true, false) => "> ",
                    _ => "  ",
                },
                label_style,
            ),
            Span::styled(format!("{glyph}  "), status),
            Span::styled(format!("{:<width$}  ", field.label()), label_style),
        ];
        if !matches!(field, Field::Pick { always: true, .. }) {
            row.push(match field {
                Field::Secret { alert: true, .. } => Span::styled(value, Style::new().fg(ALERT)),
                Field::Text { .. }
                | Field::Secret { .. }
                | Field::Pick { .. }
                | Field::Measure { .. }
                | Field::Fixed { .. }
                | Field::Action { .. }
                | Field::Gap
                | Field::Table { .. } => Span::raw(value),
            });
        }
        if let Field::Measure { unit, .. } = field {
            row.push(Span::styled(format!(" {unit}"), Style::new().dim()));
        }
        if editing {
            let caret = CARET.with(std::cell::Cell::get);
            row.push(Span::styled(
                "\u{258e}",
                Style::new().fg(blend(caret as usize % CARET_STEPS, CARET_STEPS)),
            ));
        }
        lines.push(Line::from(row));
        rows_began = true;
        if here {
            focus = field_at;
        }
        let Field::Pick {
            options,
            at,
            always,
            ..
        } = field
        else {
            continue;
        };
        let hot = open.filter(|_| here);
        let any = options.iter().any(|choice| !choice.hidden);
        if hot.is_none() && !*always && any {
            continue;
        }
        let room = form_room();
        let mut option_line = field_at + 1;
        if !any {
            lines.push(Line::from(Span::styled(
                format!("      {NO_CHOICES}"),
                Style::new().dim(),
            )));
            continue;
        }
        for (n, choice) in options.iter().enumerate() {
            if choice.hidden {
                continue;
            }
            let on_it = hot == Some(n);
            let row = match (choice.available, on_it) {
                (false, _) => Style::new().dim(),
                (true, true) => Style::new().fg(HIGHLIGHT).bold(),
                (true, false) => Style::new(),
            };
            let checked = match *always {
                true => *at == Some(n),
                false => on_it,
            };
            let radio = match checked {
                true => "\u{25c9} ",
                false => "\u{25cb} ",
            };
            let marker = match on_it {
                true => "    > ",
                false => "      ",
            };
            let alone =
                marker.chars().count() + radio.chars().count() + choice.label.chars().count();
            let wrapped =
                !choice.detail.is_empty() && alone + 2 + choice.detail.chars().count() > room;
            lines.push(Line::from(vec![
                Span::styled(marker, row),
                Span::styled(radio, row),
                Span::styled(choice.label.clone(), row),
                match wrapped {
                    true => Span::raw(""),
                    false => Span::raw("  "),
                },
                match wrapped {
                    true => Span::raw(""),
                    false => Span::styled(choice.detail.clone(), Style::new().dim()),
                },
            ]));
            if on_it {
                focus = option_line;
            }
            option_line += 1;
            if wrapped {
                for line in table::wrap(&choice.detail, room.saturating_sub(10)) {
                    lines.push(Line::from(Span::styled(
                        format!("          {line}"),
                        Style::new().dim(),
                    )));
                    option_line += 1;
                }
            }
        }
    }

    let mut tail = lines.len();
    if !actions.is_empty() {
        lines.push(Line::default());
        let on_actions = cursor == visible.len();
        let mut buttons: Vec<Span<'static>> = Vec::new();
        for (n, action) in actions.iter().enumerate() {
            let style = match (n == 0 && blocked.is_some(), on_actions && n == button) {
                (true, true) => Style::new().dim().reversed(),
                (true, false) => Style::new().dim(),
                (false, true) => Style::new().fg(HIGHLIGHT).bold().reversed(),
                (false, false) => Style::new(),
            };
            buttons.push(Span::styled(format!("  {action}  "), style));
            buttons.push(Span::raw(" "));
        }
        lines.push(Line::from(buttons));
        let actions_at = lines.len() - 1;
        let said = tried && blocked.is_some_and(|why| !why.is_empty());
        if said {
            if let Some(why) = blocked {
                refusing(&mut lines, why, "  ");
            }
        }
        if cursor >= visible.len() {
            focus = match said {
                true => lines.len() - 1,
                false => actions_at,
            };
        }
    } else {
        let said = tried && blocked.is_some_and(|why| !why.is_empty());
        if said {
            if let Some(why) = blocked {
                refusing(&mut lines, why, "     ");
            }
        }
        if said || cursor >= visible.len() {
            focus = lines.len().saturating_sub(1);
        }
        tail = lines.len();
    }
    (lines, focus, tail)
}

fn refusing(lines: &mut Vec<Line<'static>>, why: &str, indent: &str) {
    let room = form_room().saturating_sub(indent.chars().count());
    lines.push(Line::default());
    for line in table::wrap(why, room) {
        lines.push(Line::from(Span::styled(
            format!("{indent}{line}"),
            Style::new().dim(),
        )));
    }
}
