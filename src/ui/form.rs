use crate::ui::cells::*;
use crate::ui::choose::*;
use crate::ui::chrome::*;
use crate::ui::layout::*;
use crate::ui::menu::*;
use crate::ui::review::*;
use crate::ui::term::*;
use crate::ui::SUB_KEYS;
use ratatui::crossterm::event::KeyCode;

pub enum Field {
    Text { label: String, value: String },
    Secret {
        label: String,
        value: String,
        alert: bool,
    },
    Pick {
        label: String,
        options: Vec<Choice>,
        at: Option<usize>,
        change: bool,
        always: bool,
    },
    Measure {
        label: String,
        value: String,
        unit: String,
    },
    Fixed { label: String, value: String },
    Action { label: String, value: String },
    Gap,
    Table {
        label: String,
        column: String,
        headings: Vec<String>,
        rows: Vec<Vec<Cell>>,
        selectable: Vec<bool>,
        menus: Vec<Vec<MenuItem>>,
        cursor: usize,
        chosen: bool,
    },
}

impl Field {
    pub fn text(label: &str, value: &str) -> Self {
        Self::Text {
            label: label.to_string(),
            value: value.to_string(),
        }
    }

    pub fn secret(label: &str, value: &str) -> Self {
        Self::Secret {
            label: label.to_string(),
            value: value.to_string(),
            alert: false,
        }
    }

    pub fn pick(label: &str, options: Vec<Choice>, at: Option<usize>) -> Self {
        Self::Pick {
            label: label.to_string(),
            options,
            at,
            change: false,
            always: false,
        }
    }

    pub fn pick_change(label: &str, options: Vec<Choice>, at: Option<usize>) -> Self {
        Self::Pick {
            label: label.to_string(),
            options,
            at,
            change: true,
            always: false,
        }
    }

    pub fn radio(label: &str, options: Vec<Choice>, at: Option<usize>) -> Self {
        Self::Pick {
            label: label.to_string(),
            options,
            at,
            change: true,
            always: true,
        }
    }

    pub fn measure(label: &str, value: &str, unit: &str) -> Self {
        Self::Measure {
            label: label.to_string(),
            value: value.to_string(),
            unit: unit.to_string(),
        }
    }

    pub fn fixed(label: &str, value: &str) -> Self {
        Self::Fixed {
            label: label.to_string(),
            value: value.to_string(),
        }
    }

    pub fn action(label: &str, value: &str) -> Self {
        Self::Action {
            label: label.to_string(),
            value: value.to_string(),
        }
    }

    pub fn gap() -> Self {
        Self::Gap
    }

    pub fn table(
        label: &str,
        column: &str,
        headings: &[&str],
        rows: Vec<Vec<Cell>>,
        selectable: Vec<bool>,
        menus: Vec<Vec<MenuItem>>,
        cursor: usize,
        chosen: bool,
    ) -> Self {
        Self::Table {
            label: label.to_string(),
            column: column.to_string(),
            headings: headings.iter().map(|heading| heading.to_string()).collect(),
            cursor: cursor.min(rows.len().saturating_sub(1)),
            rows,
            selectable,
            menus,
            chosen,
        }
    }

    fn answered(&self) -> bool {
        match self {
            Self::Table { chosen, .. } => *chosen,
            Self::Pick { at, .. } => at.is_some(),
            _ => !self.value().is_empty(),
        }
    }

    pub(crate) fn status(&self) -> (bool, &'static str) {
        match self.answered() {
            true => (true, "\u{2713}"),
            false => (false, "?"),
        }
    }

    pub(crate) fn answerable(&self) -> bool {
        !matches!(self, Self::Fixed { .. } | Self::Gap)
    }

    pub(crate) fn label(&self) -> &str {
        match self {
            Self::Text { label, .. }
            | Self::Secret { label, .. }
            | Self::Pick { label, .. }
            | Self::Measure { label, .. }
            | Self::Fixed { label, .. }
            | Self::Action { label, .. }
            | Self::Table { label, .. } => label,
            Self::Gap => "",
        }
    }

    pub fn value(&self) -> String {
        match self {
            Self::Text { value, .. }
            | Self::Secret { value, .. }
            | Self::Measure { value, .. }
            | Self::Fixed { value, .. }
            | Self::Action { value, .. } => value.clone(),
            Self::Pick { options, at, .. } => at
                .and_then(|at| options.get(at))
                .map(|choice| choice.label.clone())
                .unwrap_or_default(),
            Self::Table { .. } | Self::Gap => String::new(),
        }
    }

    pub(crate) fn shown(&self) -> String {
        match self {
            Self::Secret { value, .. } if !revealed() && !value.is_empty() => {
                "*".repeat(value.chars().count())
            }
            Self::Measure { value, .. } => match value.parse::<f64>() {
                Ok(number) => format!("{number:.1}"),
                Err(_) => value.clone(),
            },
            Self::Table { .. } => String::new(),
            _ => self.value(),
        }
    }

    pub(crate) fn typing(&self) -> String {
        match self {
            Self::Secret { value, .. } if !revealed() => "*".repeat(value.chars().count()),
            Self::Measure { value, .. } => value.clone(),
            _ => self.value(),
        }
    }

    pub(crate) fn push(&mut self, letter: char) {
        match self {
            Self::Text { value, .. } | Self::Secret { value, .. } => value.push(letter),
            Self::Measure { value, .. } if letter.is_ascii_digit() => value.push(letter),
            Self::Measure { .. }
            | Self::Pick { .. }
            | Self::Fixed { .. }
            | Self::Action { .. }
            | Self::Table { .. }
            | Self::Gap => {}
        }
    }

    pub(crate) fn pop(&mut self) {
        match self {
            Self::Text { value, .. } | Self::Secret { value, .. } | Self::Measure { value, .. } => {
                value.pop();
            }
            Self::Pick { .. }
            | Self::Fixed { .. }
            | Self::Action { .. }
            | Self::Table { .. }
            | Self::Gap => {}
        }
    }
}

pub enum Filled {
    Took(usize),
    Opened(usize),
    Changed(usize),
    Table {
        row: usize,
        item: Option<usize>,
        child: Option<usize>,
    },
    Left,
}

pub enum HeaderLine {
    Section(String),
    Row(String),
    Pair(String, String),
    Good(String, String),
    Warn(String, String),
}

pub(crate) enum Mode {
    Rows,
    Typing,
    Open(usize),
    Table,
    TableMenu {
        open: Option<usize>,
        cursor: usize,
    },
}

pub fn opens_at(shown: &[usize], field: usize) -> usize {
    shown.iter().position(|at| *at == field).unwrap_or(0)
}

pub fn form(
    fields: &mut [Field],
    actions: &[&str],
    blocked: impl Fn(&[Field]) -> Option<String>,
    invalid: impl Fn(&[Field]) -> Vec<usize>,
    shown: impl Fn(&[Field]) -> Vec<usize>,
    keys: &str,
    header: &[HeaderLine],
    sections: &[(usize, &str)],
    start: usize,
    left: &mut Vec<usize>,
) -> Result<Filled, String> {
    let rows = fields
        .iter()
        .map(|field| match field {
            Field::Table { rows, .. } => rows.len() + 1,
            _ => 1,
        })
        .sum::<usize>()
        + 3;
    let view = match OVERLAY.with(std::cell::Cell::get) {
        Some((_, height)) => (rows as u16).max(height),
        None => rows as u16,
    };
    inline(view, |terminal| {
        let mut cursor = opens_at(&shown(fields), start);
        let mut button = 0usize;
        let mut mode = Mode::Rows;
        let mut tried = false;
        loop {
            let visible = shown(fields);
            cursor = cursor.min(match actions.is_empty() {
                true => visible.len().saturating_sub(1),
                false => visible.len(),
            });
            let at_row = visible.get(cursor).copied();
            if matches!(mode, Mode::Rows) {
                if let Some(row) = at_row {
                    if let Field::Pick { always: true, .. } = &fields[row] {
                        mode = opened(&fields[row]);
                    }
                }
            }
            let flagged = alerted(fields, left, &invalid);
            for (at, field) in fields.iter_mut().enumerate() {
                if let Field::Secret { alert, .. } = field {
                    *alert = flagged.get(at).copied().unwrap_or(false);
                }
            }
            let blocked = blocked(fields);
            let (lines, focus, tail) = laid_out(
                fields,
                &visible,
                cursor,
                button,
                &mode,
                actions,
                blocked.as_deref(),
                tried,
                header,
                sections,
            );
            let keys = match mode {
                Mode::Open(_) | Mode::Table | Mode::TableMenu { .. } => SUB_KEYS,
                _ => keys,
            };
            render(terminal, lines.len() as u16 + 1, keys, "", |frame, area| {
                sheet_of(frame, area, &lines, keys, focus, tail);
                if let (Mode::TableMenu { open, cursor }, Some(row)) = (&mode, at_row) {
                    if let Field::Table {
                        rows,
                        menus,
                        cursor: table_at,
                        ..
                    } = &fields[row]
                    {
                        let items = menus.get(*table_at).map(Vec::as_slice).unwrap_or(&[]);
                        let title = rows
                            .get(*table_at)
                            .and_then(|cells| cells.first())
                            .map(Cell::text)
                            .unwrap_or("")
                            .trim_start_matches([
                                '\u{251c}', '\u{2514}', '\u{2500}', '\u{2502}', ' ',
                            ]);
                        menu_draw(frame, area, title, items, *open, *cursor);
                    }
                }
                BACKDROP.with(|saved| *saved.borrow_mut() = Some(frame.buffer_mut().clone()));
            })?;
            let Some(key) = read()? else { continue };
            let Some(row) = at_row else {
                match key {
                    KeyCode::Esc | KeyCode::Char('q') => return Ok(Filled::Left),
                    KeyCode::Up | KeyCode::Char('k') => cursor = cursor.saturating_sub(1),
                    KeyCode::Left => button = button.saturating_sub(1),
                    KeyCode::Right => button = (button + 1).min(actions.len().saturating_sub(1)),
                    KeyCode::Enter if button > 0 || blocked.is_none() => {
                        return Ok(Filled::Took(button))
                    }
                    KeyCode::Enter => tried = true,
                    _ => {}
                }
                continue;
            };
            let mut leave: Option<bool> = None;
            match &mut mode {
                Mode::Typing => match key {
                    KeyCode::Enter => {
                        left_row(left, row);
                        if matches!(fields[row], Field::Measure { .. }) {
                            return Ok(Filled::Changed(row));
                        }
                        let next = onward(fields, &visible, &mut cursor);
                        if cursor == visible.len() && actions.is_empty() {
                            if blocked.is_none() {
                                return Ok(Filled::Took(0));
                            }
                            tried = true;
                        }
                        mode = next;
                    }
                    KeyCode::Esc => {
                        left_row(left, row);
                        mode = Mode::Rows;
                    }
                    KeyCode::Up => {
                        left_row(left, row);
                        mode = backward(fields, &visible, &mut cursor);
                    }
                    KeyCode::Down => {
                        left_row(left, row);
                        mode = onward(fields, &visible, &mut cursor);
                    }
                    KeyCode::Backspace => fields[row].pop(),
                    KeyCode::Char(letter) => fields[row].push(letter),
                    _ => {}
                },
                Mode::Open(at) => {
                    let Field::Pick {
                        options,
                        change,
                        always,
                        ..
                    } = &fields[row]
                    else {
                        mode = Mode::Rows;
                        continue;
                    };
                    let (change, always) = (*change, *always);
                    match key {
                        KeyCode::Enter => {
                            let taken = *at;
                            if available(options, taken) {
                                if let Field::Pick { at: held, .. } = &mut fields[row] {
                                    *held = Some(taken);
                                }
                                if change {
                                    return Ok(Filled::Changed(row));
                                }
                                mode = onward(fields, &shown(fields), &mut cursor);
                            }
                        }
                        KeyCode::Esc | KeyCode::Char('q') => match always {
                            true => return Ok(Filled::Left),
                            false => mode = Mode::Rows,
                        },
                        KeyCode::Up | KeyCode::Char('k') => {
                            match option_step(options, *at, false) {
                                Some(next) => *at = next,
                                None => leave = Some(false),
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => match option_step(options, *at, true)
                        {
                            Some(next) => *at = next,
                            None => leave = Some(true),
                        },
                        _ => {}
                    }
                }
                Mode::Table => {
                    let Field::Table {
                        selectable,
                        menus,
                        cursor: table_cursor,
                        ..
                    } = &mut fields[row]
                    else {
                        mode = Mode::Rows;
                        continue;
                    };
                    match key {
                        KeyCode::Esc | KeyCode::Char('q') => mode = Mode::Rows,
                        KeyCode::Up | KeyCode::Char('k') => {
                            match table_step(selectable.as_slice(), *table_cursor, false) {
                                Some(next) => *table_cursor = next,
                                None => leave = Some(false),
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            match table_step(selectable.as_slice(), *table_cursor, true) {
                                Some(next) => *table_cursor = next,
                                None => leave = Some(true),
                            }
                        }
                        KeyCode::Enter => {
                            match menus.get(*table_cursor).map(Vec::as_slice).unwrap_or(&[]) {
                                [] => {
                                    return Ok(Filled::Table {
                                        row: *table_cursor,
                                        item: None,
                                        child: None,
                                    })
                                }
                                _ => {
                                    mode = Mode::TableMenu {
                                        open: None,
                                        cursor: 0,
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Mode::TableMenu { open, cursor } => {
                    let Field::Table {
                        menus,
                        cursor: table_at,
                        ..
                    } = &fields[row]
                    else {
                        mode = Mode::Rows;
                        continue;
                    };
                    let items = menus.get(*table_at).map(Vec::as_slice).unwrap_or(&[]);
                    let visible = menu_rows(items, *open);
                    match key {
                        KeyCode::Esc | KeyCode::Char('q') => match open {
                            Some(was) => {
                                let was = *was;
                                *open = None;
                                *cursor = menu_rows(items, *open)
                                    .iter()
                                    .position(|(at, _)| *at == was)
                                    .unwrap_or(0);
                            }
                            None => mode = Mode::Table,
                        },
                        KeyCode::Enter => {
                            let Some(&(item, child)) = visible.get(*cursor) else {
                                continue;
                            };
                            if child.is_some() || items[item].children.is_empty() {
                                return Ok(Filled::Table {
                                    row: *table_at,
                                    item: Some(item),
                                    child,
                                });
                            }
                            match *open {
                                Some(was) if was == item => {
                                    *open = None;
                                    *cursor = menu_rows(items, *open)
                                        .iter()
                                        .position(|(at, _)| *at == item)
                                        .unwrap_or(0);
                                }
                                _ => {
                                    *open = Some(item);
                                    *cursor = menu_rows(items, *open)
                                        .iter()
                                        .position(|(at, child)| *at == item && child.is_none())
                                        .unwrap_or(0);
                                }
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => *cursor = cursor.saturating_sub(1),
                        KeyCode::Down | KeyCode::Char('j') => {
                            *cursor = (*cursor + 1).min(visible.len().saturating_sub(1))
                        }
                        _ => {}
                    }
                }
                Mode::Rows => match key {
                    KeyCode::Char(letter)
                        if matches!(
                            fields[row],
                            Field::Text { .. } | Field::Secret { .. } | Field::Measure { .. }
                        ) =>
                    {
                        let takes = !matches!(fields[row], Field::Measure { .. })
                            || letter.is_ascii_digit();
                        match takes {
                            true => {
                                fields[row].push(letter);
                                mode = Mode::Typing;
                            }
                            false if letter == 'q' => return Ok(Filled::Left),
                            false => {}
                        }
                    }
                    KeyCode::Backspace
                        if matches!(
                            fields[row],
                            Field::Text { .. } | Field::Secret { .. } | Field::Measure { .. }
                        ) =>
                    {
                        fields[row].pop();
                        mode = Mode::Typing;
                    }
                    KeyCode::Esc | KeyCode::Char('q') => return Ok(Filled::Left),
                    KeyCode::Up | KeyCode::Char('k') => {
                        mode = backward(fields, &visible, &mut cursor)
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        mode = onward(fields, &visible, &mut cursor)
                    }
                    KeyCode::Enter if matches!(fields[row], Field::Action { .. }) => {
                        return Ok(Filled::Opened(row))
                    }
                    KeyCode::Enter => mode = opened(&fields[row]),
                    _ => {}
                },
            }
            if let Some(down) = leave {
                mode = match down {
                    true => onward(fields, &shown(fields), &mut cursor),
                    false => backward(fields, &shown(fields), &mut cursor),
                };
            }
        }
    })
}

pub(crate) fn onward(fields: &[Field], visible: &[usize], cursor: &mut usize) -> Mode {
    loop {
        *cursor = (*cursor + 1).min(visible.len());
        match visible.get(*cursor) {
            Some(row) if !fields[*row].answerable() => continue,
            _ => return landed(fields, visible, *cursor),
        }
    }
}

pub(crate) fn backward(fields: &[Field], visible: &[usize], cursor: &mut usize) -> Mode {
    loop {
        let above = cursor.saturating_sub(1);
        let stuck = above == *cursor;
        *cursor = above;
        match visible.get(*cursor) {
            Some(row) if !fields[*row].answerable() && !stuck => continue,
            _ => return landed(fields, visible, *cursor),
        }
    }
}

pub(crate) fn landed(fields: &[Field], visible: &[usize], cursor: usize) -> Mode {
    match visible.get(cursor) {
        Some(row) => match &fields[*row] {
            Field::Pick { always: true, .. } => opened(&fields[*row]),
            _ => Mode::Rows,
        },
        None => Mode::Rows,
    }
}

pub(crate) fn opened(field: &Field) -> Mode {
    match field {
        Field::Pick { options, at, .. } => options
            .iter()
            .position(|choice| !choice.hidden)
            .map(|first| {
                Mode::Open(
                    at.filter(|at| options.get(*at).is_some_and(|choice| !choice.hidden))
                        .unwrap_or(first),
                )
            })
            .unwrap_or(Mode::Rows),
        Field::Table { .. } => Mode::Table,
        Field::Fixed { .. } | Field::Action { .. } | Field::Gap => Mode::Rows,
        _ => Mode::Typing,
    }
}

pub(crate) fn option_step(options: &[Choice], at: usize, down: bool) -> Option<usize> {
    let mut range: Box<dyn Iterator<Item = usize>> = match down {
        true => Box::new(at.saturating_add(1)..options.len()),
        false => Box::new((0..at).rev()),
    };
    range.find(|next| !options[*next].hidden)
}

pub(crate) fn table_step(selectable: &[bool], at: usize, down: bool) -> Option<usize> {
    let at = at.min(selectable.len().saturating_sub(1));
    let mut next = at;
    loop {
        let ahead = match down {
            true => next + 1,
            false => next.saturating_sub(1),
        };
        if ahead == next || ahead >= selectable.len() {
            return None;
        }
        next = ahead;
        if selectable.get(next).copied().unwrap_or(false) {
            return Some(next);
        }
    }
}

pub(crate) fn form_room() -> usize {
    match OVERLAY.with(std::cell::Cell::get) {
        Some((width, _)) => (width as usize).saturating_sub(2 + 2 * PAD_X as usize),
        None => (WIDEST.min(width() as u16) as usize).saturating_sub(2 + 2 * PAD_X as usize),
    }
}

fn left_row(left: &mut Vec<usize>, row: usize) {
    if !left.contains(&row) {
        left.push(row);
    }
}
