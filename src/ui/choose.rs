use crate::ui::chrome::*;
use crate::ui::nest::*;
use crate::ui::progress::*;
use crate::ui::term::*;
use crate::ui::{EITHER, EITHER_CANCEL, PICK, TOGGLE, TOGGLE_HIDE, TOGGLE_SHOW};
use ratatui::backend::Backend;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{HighlightSpacing, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

#[derive(Clone)]
pub struct Choice {
    pub label: String,
    pub detail: String,
    pub parent: Option<usize>,
    pub group: String,
    pub available: bool,
    pub hidden: bool,
    pub dim: bool,
    pub tint: bool,
    pub heading: bool,
    pub warning: bool,
}

impl Choice {
    pub fn new(label: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            detail: detail.into(),
            parent: None,
            group: String::new(),
            available: true,
            hidden: false,
            dim: false,
            tint: false,
            heading: false,
            warning: false,
        }
    }

    pub fn unavailable(mut self) -> Self {
        self.available = false;
        self.dim = true;
        self
    }

    pub fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }

    pub fn content(mut self) -> Self {
        self.available = false;
        self
    }

    pub fn tinted(mut self) -> Self {
        self.tint = true;
        self
    }

    pub fn heading(mut self) -> Self {
        self.available = false;
        self.heading = true;
        self
    }

    pub fn warning(mut self) -> Self {
        self.warning = true;
        self
    }

    pub fn under(mut self, parent: usize) -> Self {
        self.parent = Some(parent);
        self
    }

    pub fn within(mut self, group: impl Into<String>) -> Self {
        self.group = group.into();
        self
    }
}

pub enum Answer {
    Cancelled,
    Chosen(Vec<usize>),
}

pub(crate) const VISIBLE: usize = 8;

pub fn select(question: &str, options: &[Choice]) -> Result<Option<usize>, String> {
    select_current(question, options, 0)
}

pub fn select_current(
    question: &str,
    options: &[Choice],
    at: usize,
) -> Result<Option<usize>, String> {
    inline(height(options.len()), |terminal| {
        pick(terminal, question, options, PICK, at)
    })
}

pub const WINDOW_WIDTH: u16 = 68;

pub fn choose(
    title: &str,
    keys: &str,
    options: &[&str],
    at: usize,
) -> Result<Option<usize>, String> {
    let height = to_cells(options.len() + 4);
    in_titled_overlay(WINDOW_WIDTH, height, title, || {
        inline(height, |terminal| {
            let mut cursor = at.min(options.len().saturating_sub(1));
            loop {
                render(terminal, height, keys, "", |frame, area| {
                    let lines: Vec<Line> = options
                        .iter()
                        .enumerate()
                        .map(|(n, option)| {
                            let (marker, style) = match n == cursor {
                                true => ("> ", Style::new().fg(HIGHLIGHT).bold()),
                                false => ("  ", Style::new()),
                            };
                            Line::from(vec![
                                Span::raw(marker),
                                Span::styled((*option).to_string(), style),
                            ])
                        })
                        .collect();
                    frame.render_widget(Paragraph::new(lines), area);
                })?;
                let Some(key) = read()? else { continue };
                match key {
                    KeyCode::Enter => return Ok(options.get(cursor).map(|_| cursor)),
                    KeyCode::Esc | KeyCode::Char('q') => return Ok(None),
                    KeyCode::Up | KeyCode::Char('k') => cursor = cursor.saturating_sub(1),
                    KeyCode::Down | KeyCode::Char('j') => {
                        cursor = (cursor + 1).min(options.len().saturating_sub(1))
                    }
                    _ => {}
                }
            }
        })
    })
}

pub fn confirm_over(
    title: &str,
    heading: &str,
    rows: &[(String, String)],
    yes: &str,
    no: &str,
    keys: &str,
) -> Result<bool, String> {
    let mut button = 0usize;
    let head = usize::from(!heading.is_empty()) * 2;
    let height = to_cells(rows.len() + head + 2 + 4);
    in_titled_overlay(WINDOW_WIDTH, height, title, || {
        inline(height, |terminal| loop {
            let mut lines: Vec<Line<'static>> = Vec::new();
            if !heading.is_empty() {
                lines.push(Line::from(Span::styled(
                    format!("  {heading}"),
                    Style::new().fg(Color::White).bold(),
                )));
                lines.push(Line::default());
            }
            let width = rows
                .iter()
                .map(|(name, _)| name.chars().count())
                .max()
                .unwrap_or(0);
            for (name, detail) in rows {
                lines.push(Line::from(vec![
                    Span::styled(format!("  {name:<width$}  "), Style::new()),
                    Span::styled(detail.clone(), Style::new().dim()),
                ]));
            }
            lines.push(Line::default());
            let mut buttons: Vec<Span<'static>> = Vec::new();
            for (at, action) in [yes, no].into_iter().enumerate() {
                let style = match at == button {
                    true => Style::new().fg(HIGHLIGHT).bold().reversed(),
                    false => Style::new(),
                };
                buttons.push(Span::styled(format!("  {action}  "), style));
                buttons.push(Span::raw(" "));
            }
            lines.push(Line::from(buttons));
            render(terminal, height, keys, "", |frame, area| {
                frame.render_widget(Paragraph::new(lines.clone()), area);
            })?;
            let Some(key) = read()? else { continue };
            match key {
                KeyCode::Left | KeyCode::Char('h') => button = button.saturating_sub(1),
                KeyCode::Right | KeyCode::Char('l') => button = (button + 1).min(1),
                KeyCode::Enter => return Ok(button == 0),
                KeyCode::Esc | KeyCode::Char('q') => return Ok(false),
                _ => {}
            }
        })
    })
}

pub fn confirm(question: &str, yes: &str, no: &str) -> Result<Option<bool>, String> {
    confirm_current(question, yes, no, true)
}

pub fn confirm_current(
    question: &str,
    yes: &str,
    no: &str,
    current: bool,
) -> Result<Option<bool>, String> {
    confirm_choice(question, yes, no, current, EITHER_CANCEL)
}

pub fn confirm_or_no(question: &str, yes: &str, no: &str) -> Result<bool, String> {
    confirm_current_or_no(question, yes, no, true)
}

pub fn confirm_current_or_no(
    question: &str,
    yes: &str,
    no: &str,
    current: bool,
) -> Result<bool, String> {
    Ok(confirm_choice(question, yes, no, current, EITHER)?.unwrap_or(false))
}

fn confirm_choice(
    question: &str,
    yes: &str,
    no: &str,
    current: bool,
    keys: &str,
) -> Result<Option<bool>, String> {
    let options = [Choice::new(yes, ""), Choice::new(no, "")];
    let chosen = inline(height(options.len()), |terminal| {
        pick(terminal, question, &options, keys, usize::from(!current))
    })?;
    Ok(chosen.map(|chosen| chosen == 0))
}

pub fn multi(question: &str, options: &[Choice], on: &[usize]) -> Result<Answer, String> {
    if options.iter().any(|choice| !choice.group.is_empty()) {
        let rows = nodes(options).len();
        return inline(height(rows) + 2, |terminal| {
            nest(terminal, question, options, on)
        });
    }
    inline(height(options.len()), |terminal| {
        toggle(terminal, question, options, on)
    })
}

pub(crate) fn pick<B: Backend>(
    terminal: &mut ratatui::Terminal<B>,
    question: &str,
    options: &[Choice],
    hint: &str,
    selected: usize,
) -> Result<Option<usize>, String> {
    let mut state = initial_state(options, selected);
    loop {
        render(
            terminal,
            height(options.len()),
            hint,
            question,
            |frame, area| draw(frame, area, question, options, None, hint, &mut state),
        )?;
        let Some(key) = read()? else { continue };
        match key {
            KeyCode::Enter => match state.selected() {
                Some(at) if !available(options, at) => {}
                chosen => return Ok(chosen),
            },
            KeyCode::Esc | KeyCode::Char('q') => return Ok(None),
            code => {
                let from = state.selected();
                move_by(code, &mut state, options.len());
                skip_spacers(code, options, &mut state, from);
            }
        }
    }
}

pub(crate) fn initial_state(options: &[Choice], selected: usize) -> ListState {
    let mut state = ListState::default().with_selected(Some(selected));
    skip_spacers(KeyCode::Down, options, &mut state, Some(selected));
    state
}

fn passed_over(options: &[Choice], at: usize) -> bool {
    options
        .get(at)
        .is_some_and(|row| row.label.is_empty() || row.heading)
}

pub(crate) fn skip_spacers(
    code: KeyCode,
    options: &[Choice],
    state: &mut ListState,
    from: Option<usize>,
) {
    let step = match code {
        KeyCode::Home => KeyCode::Down,
        KeyCode::End => KeyCode::Up,
        code => code,
    };
    for _ in 0..options.len() {
        let Some(at) = state.selected() else { return };
        if !passed_over(options, at) {
            break;
        }
        move_by(step, state, options.len());
        if state.selected() == Some(at) {
            break;
        }
    }
    if state.selected().is_some_and(|at| passed_over(options, at)) {
        state.select(from);
    }
    if let Some(at) = state.selected() {
        let top = (0..at)
            .rev()
            .take_while(|&above| passed_over(options, above))
            .last()
            .unwrap_or(at);
        if state.offset() > top {
            *state.offset_mut() = top;
        }
    }
}

pub(crate) fn toggle<B: Backend>(
    terminal: &mut ratatui::Terminal<B>,
    question: &str,
    options: &[Choice],
    held: &[usize],
) -> Result<Answer, String> {
    let mut state = ListState::default().with_selected(Some(0));
    let mut on: Vec<usize> = held.to_vec();
    let mut hide = false;
    let dimmed = options.iter().any(|choice| choice.dim);
    loop {
        let (kept, rows) = visible(options, hide);
        let marked: Vec<usize> = kept
            .iter()
            .enumerate()
            .filter(|(_, at)| on.contains(at))
            .map(|(row, _)| row)
            .collect();
        let legend = match (dimmed, hide) {
            (false, _) => TOGGLE,
            (true, false) => TOGGLE_HIDE,
            (true, true) => TOGGLE_SHOW,
        };
        render(
            terminal,
            height(options.len()),
            legend,
            question,
            |frame, area| {
                draw(
                    frame,
                    area,
                    question,
                    &rows,
                    Some(&marked),
                    legend,
                    &mut state,
                )
            },
        )?;
        let Some(key) = read()? else { continue };
        match key {
            KeyCode::Enter => return Ok(Answer::Chosen(on)),
            KeyCode::Char(' ') => match state.selected().and_then(|row| kept.get(row)) {
                Some(at) if available(options, *at) => flip(&mut on, *at, options),
                _ => {}
            },
            KeyCode::Tab if dimmed => {
                hide = !hide;
                let left = visible(options, hide).0.len();
                state.select(Some(
                    state.selected().unwrap_or(0).min(left.saturating_sub(1)),
                ));
            }
            KeyCode::Esc | KeyCode::Char('q') => return Ok(Answer::Cancelled),
            code => move_by(code, &mut state, kept.len()),
        }
    }
}

pub(crate) fn visible(options: &[Choice], hide: bool) -> (Vec<usize>, Vec<Choice>) {
    let kept: Vec<usize> = (0..options.len())
        .filter(|at| !hide || !options[*at].dim)
        .collect();
    let rows = kept
        .iter()
        .map(|at| {
            let mut row = options[*at].clone();
            row.parent = row
                .parent
                .and_then(|parent| kept.iter().position(|held| *held == parent));
            row
        })
        .collect();
    (kept, rows)
}

pub(crate) fn available(options: &[Choice], at: usize) -> bool {
    options.get(at).is_some_and(|choice| choice.available)
}

pub(crate) fn flip(on: &mut Vec<usize>, at: usize, options: &[Choice]) {
    if let Some(held) = on.iter().position(|held| *held == at) {
        on.remove(held);
        return;
    }
    let parent = options[at].parent;
    on.retain(|held| Some(*held) != parent && options[*held].parent != Some(at));
    on.push(at);
}

pub(crate) fn branch(options: &[Choice], at: usize) -> &'static str {
    let Some(parent) = options[at].parent else {
        return "";
    };
    match options[at + 1..]
        .iter()
        .any(|choice| choice.parent == Some(parent))
    {
        true => "\u{251c}\u{2500} ",
        false => "\u{2514}\u{2500} ",
    }
}

pub(crate) fn move_by(code: KeyCode, state: &mut ListState, len: usize) {
    match code {
        KeyCode::Up | KeyCode::Char('k') => state.select_previous(),
        KeyCode::Down | KeyCode::Char('j') => state.select_next(),
        KeyCode::Home => state.select_first(),
        KeyCode::End => state.select_last(),
        _ => {}
    }
    if let (Some(at), Some(last)) = (state.selected(), len.checked_sub(1)) {
        state.select(Some(at.min(last)));
    }
}

pub(crate) fn draw(
    frame: &mut Frame,
    area: Rect,
    question: &str,
    options: &[Choice],
    on: Option<&[usize]>,
    hint_text: &str,
    state: &mut ListState,
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

    let selected = state.selected();
    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(at, choice)| {
            let mark = match on {
                None => "",
                Some(on) if on.contains(&at) => "[x] ",
                Some(_) => "[ ] ",
            };
            let row = match (choice.heading, choice.dim) {
                (true, _) => Style::new().fg(Color::White).bold(),
                (false, true) => Style::new().dim(),
                (false, false) if choice.warning => Style::new().fg(AMBER).bold(),
                (false, false) => Style::new(),
            };
            let pointer = match (choice.heading, selected == Some(at)) {
                (true, _) => "",
                (false, true) => "> ",
                (false, false) => "  ",
            };
            let mut spans = vec![
                Span::styled(pointer, row),
                Span::styled(mark, row),
                Span::styled(branch(options, at), row),
            ];
            match choice.tint {
                true => spans.extend(tinted(&choice.label)),
                false => spans.push(Span::styled(choice.label.clone(), row)),
            }
            spans.push(Span::raw("  "));
            spans.push(Span::styled(choice.detail.clone(), Style::new().dim()));
            ListItem::new(Line::from(spans))
        })
        .collect();
    let rows = items.len();
    let placed = match boxed() {
        true => set_in(body, &items, rows),
        false => body,
    };
    frame.render_stateful_widget(
        List::new(items)
            .highlight_spacing(HighlightSpacing::Never)
            .highlight_style(Style::new().fg(HIGHLIGHT).bold()),
        placed,
        state,
    );
    frame.render_widget(
        Line::from(hint(hint_row(hint_text), state, placed.height, rows).dim()),
        foot,
    );
}

pub(crate) fn hint(hint: &str, state: &ListState, height: u16, rows: usize) -> String {
    let seen = (state.offset() + usize::from(height)).min(rows);
    match seen < rows || state.offset() > 0 {
        true => format!("{hint}  {seen} of {rows}"),
        false => hint.to_string(),
    }
}
