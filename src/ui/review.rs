use crate::ui::choose::*;
use crate::ui::chrome::*;
use crate::ui::table;
use crate::ui::term::*;
use crate::ui::{LINE_KEYS, SIDE};
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub(crate) fn sheet_of(
    frame: &mut Frame,
    area: Rect,
    lines: &[Line<'static>],
    keys: &str,
    focus: usize,
    tail: usize,
) {
    let keys = hint_row(keys);
    let [body, foot] = Layout::vertical([
        Constraint::Min(1),
        Constraint::Length(u16::from(!keys.is_empty())),
    ])
    .areas(area);
    let height = usize::from(body.height);
    let mut shown = lines.to_vec();
    let mut focus = focus;
    if shown.len() < height {
        let pad = height - shown.len();
        for _ in 0..pad {
            shown.insert(tail, Line::default());
        }
        if focus >= tail {
            focus += pad;
        }
    }
    let lines = shown;
    let scroll = focus
        .saturating_sub(height.saturating_sub(1))
        .min(lines.len().saturating_sub(height));
    frame.render_widget(
        Paragraph::new(lines.to_vec()).scroll((scroll as u16, 0)),
        body,
    );
    frame.render_widget(Line::from(keys.dim()), foot);
}

pub fn review(
    question: &str,
    rows: &[(String, String)],
    action: &str,
    keys: &str,
    blocked: Option<&str>,
    at: usize,
) -> Result<Option<usize>, String> {
    let options = sheet(rows, action, blocked);
    let chosen = inline((options.len() + 2) as u16, |terminal| {
        pick(
            terminal,
            question,
            &options,
            keys,
            at.min(options.len() - 1),
        )
    })?;
    // The row count is the action's returned index, because the action is not a row.
    Ok(chosen.map(|at| at.min(rows.len())))
}

pub(crate) fn decide_content_height(
    heading: &str,
    note: &str,
    warning: &[&str],
    rows: &[(String, String)],
) -> usize {
    let cost = match note.is_empty() {
        true => 0,
        false => table::wrap(note, PANEL_ROOM).len() + 2,
    };
    let warned = match warning.is_empty() {
        true => 0,
        false => warning.len() + 1,
    };
    rows.len() + 3 + cost + warned + usize::from(!head_row(heading).is_empty())
}

pub fn decide(
    heading: &str,
    note: &str,
    warning: &[&str],
    rows: &[(String, String)],
    ready: &str,
    yes: &str,
    no: &str,
) -> Result<bool, String> {
    let content = decide_content_height(heading, note, warning, rows);
    let mut button = 0;
    inline((content + 2) as u16, |terminal| loop {
        if !decide_fits(
            content,
            terminal
                .size()
                .map_err(|err| err.to_string())?
                .height
                .into(),
        ) {
            return Err(
                "the confirmation is too tall to show every destructive change".to_string(),
            );
        }
        render(terminal, content as u16, SIDE, heading, |frame, area| {
            decide_draw(
                frame, area, heading, note, warning, rows, ready, yes, no, button,
            )
        })?;
        let Some(key) = read()? else { continue };
        match key {
            KeyCode::Left | KeyCode::Up | KeyCode::Char('h') | KeyCode::Char('k') => button = 0,
            KeyCode::Right | KeyCode::Down | KeyCode::Char('l') | KeyCode::Char('j') => {
                button = 1;
            }
            KeyCode::Enter => return Ok(button == 0),
            KeyCode::Esc | KeyCode::Char('q') => return Ok(false),
            _ => {}
        }
    })
}

pub(crate) fn decide_fits(content: usize, height: usize) -> bool {
    content.saturating_add(2) <= height
}

pub(crate) fn decide_draw(
    frame: &mut Frame,
    area: Rect,
    heading: &str,
    note: &str,
    warning: &[&str],
    rows: &[(String, String)],
    ready: &str,
    yes: &str,
    no: &str,
    button: usize,
) {
    let head = head_row(heading);
    let mut lines: Vec<Line> = Vec::new();
    if !head.is_empty() {
        lines.push(Line::from(Span::styled(
            head.to_string(),
            Style::new().fg(ACCENT).bold(),
        )));
    }
    lines.push(Line::from(Span::styled(
        ready.to_string(),
        Style::new().fg(Color::White).bold(),
    )));
    if !note.is_empty() {
        lines.push(Line::default());
        for line in table::wrap(note, PANEL_ROOM) {
            lines.push(Line::from(Span::styled(line, Style::new())));
        }
        lines.push(Line::default());
    }
    for line in warning {
        lines.push(Line::from(Span::styled(
            (*line).to_string(),
            Style::new().fg(AMBER).bold(),
        )));
    }
    if !warning.is_empty() {
        lines.push(Line::default());
    }
    let width = rows
        .iter()
        .map(|(label, _)| label.chars().count())
        .max()
        .unwrap_or(0);
    for (label, value) in rows {
        lines.push(Line::from(vec![
            Span::styled(format!("{label:<width$}"), Style::new()),
            Span::raw("  "),
            Span::styled(value.clone(), Style::new().dim()),
        ]));
    }
    let mut buttons: Vec<Span<'static>> = vec![Span::raw("  ")];
    for (at, action) in [yes, no].into_iter().enumerate() {
        let style = match at == button {
            true => Style::new().fg(HIGHLIGHT).bold().reversed(),
            false => Style::new(),
        };
        buttons.push(Span::styled(format!("  {action}  "), style));
        buttons.push(Span::raw(" "));
    }
    let footer = vec![Line::default(), Line::from(buttons)];

    let widest = lines
        .iter()
        .chain(&footer)
        .map(Line::width)
        .max()
        .unwrap_or(0) as u16;
    let placed = match CHROME.get().is_some() {
        true => {
            let all = lines.len() as u16 + footer.len() as u16;
            centred(area, widest, all)
        }
        false => area,
    };
    let [body, act] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(footer.len() as u16)])
            .areas(placed);
    frame.render_widget(Paragraph::new(lines), body);
    frame.render_widget(Paragraph::new(footer), act);
}

pub fn offer_over(
    question: &str,
    rows: Vec<Choice>,
    actions: &[Choice],
    keys: &str,
) -> Result<Option<usize>, String> {
    let mut options = rows;
    let first = options.len() + 1;
    options.push(Choice::new("", ""));
    options.extend(actions.iter().cloned());
    let chosen = inline((options.len() + 2) as u16, |terminal| {
        pick(terminal, question, &options, keys, first)
    })?;
    Ok(chosen.and_then(|chosen| chosen.checked_sub(first)))
}

fn sheet(rows: &[(String, String)], action: &str, blocked: Option<&str>) -> Vec<Choice> {
    let width = rows
        .iter()
        .map(|(label, _)| label.chars().count())
        .max()
        .unwrap_or(0);
    let mut options: Vec<Choice> = rows
        .iter()
        .map(|(label, value)| Choice::new(format!("{label:<width$}"), value))
        .collect();
    options.push(Choice::new("", ""));
    options.push(match blocked {
        None => Choice::new(action, ""),
        Some(why) => Choice::new(action, why).unavailable(),
    });
    options
}

pub fn line(question: &str, prefix: &str, default: Option<&str>) -> Result<String, String> {
    inline(3, |terminal| {
        let mut typed = String::new();
        loop {
            render(terminal, 3, LINE_KEYS, question, |frame, area| {
                written(frame, area, question, prefix, &typed, default)
            })?;
            let Some(key) = read()? else { continue };
            match key {
                KeyCode::Enter => return Ok(typed),
                KeyCode::Esc => return Ok(String::new()),
                KeyCode::Backspace => {
                    typed.pop();
                }
                KeyCode::Char(letter) => typed.push(letter),
                _ => {}
            }
        }
    })
}

pub(crate) fn written(
    frame: &mut Frame,
    area: Rect,
    question: &str,
    prefix: &str,
    typed: &str,
    default: Option<&str>,
) {
    let heading = head_row(question);
    let [head, body, foot] = Layout::vertical([
        Constraint::Length(u16::from(!heading.is_empty())),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .areas(area);
    if !heading.is_empty() {
        frame.render_widget(
            Line::from(Span::styled(
                heading.to_string(),
                Style::new().fg(ACCENT).bold(),
            )),
            head,
        );
    }
    let answer = match (typed.is_empty(), default) {
        (true, Some(default)) => Span::styled(default, Style::new().dim()),
        _ => Span::raw(typed),
    };
    frame.render_widget(Line::from(vec![Span::raw(prefix), answer]), body);
    frame.render_widget(Line::from(hint_row(LINE_KEYS).dim()), foot);
}
