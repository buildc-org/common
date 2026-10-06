use crate::ui::chrome::*;
use crate::ui::term::*;
use ratatui::crossterm::terminal;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::{DefaultTerminal, Frame};

const LOGGED: usize = 256;

pub(crate) const ROWS: u16 = 9;

pub struct Progress {
    terminal: DefaultTerminal,
    pct: u16,
    flight: u16,
    within: u32,
    turn: usize,
    step: String,
    notes: Vec<String>,
    foot: String,
}

impl Progress {
    pub fn open(foot: &str) -> Result<Self, String> {
        let terminal = open(ROWS)?;
        let _ = terminal::disable_raw_mode();
        Ok(Self {
            terminal,
            pct: 0,
            flight: 0,
            within: 0,
            turn: 0,
            step: String::new(),
            notes: Vec::new(),
            foot: foot.to_string(),
        })
    }

    pub fn step(&mut self, pct: u16, flight: u16, name: &str) -> Result<(), String> {
        self.pct = pct.min(100);
        self.flight = flight.min(100);
        self.within = 0;
        self.step = name.to_string();
        self.show()
    }

    fn at(&self) -> u16 {
        crept(self.pct, self.flight, self.within)
    }

    pub fn tick(&mut self) -> Result<(), String> {
        self.turn = self.turn.wrapping_add(1);
        self.show()
    }

    pub fn note(&mut self, text: &str) -> Result<(), String> {
        self.within += 1;
        if self.notes.len() == LOGGED {
            self.notes.remove(0);
        }
        self.notes.push(text.to_string());
        self.show()
    }

    fn show(&mut self) -> Result<(), String> {
        let (pct, turn, step, notes, foot) = (
            self.at(),
            self.turn,
            self.step.as_str(),
            self.notes.as_slice(),
            self.foot.as_str(),
        );
        let head = progress_head(turn, step);
        render(&mut self.terminal, ROWS, foot, &head, |frame, area| {
            working(frame, area, pct, turn, step, notes, foot)
        })
    }

    pub fn close(self) {
        close(self.terminal);
    }
}

pub(crate) fn progress_head(turn: usize, step: &str) -> String {
    match boxed() && !step.is_empty() {
        true => step_line(turn, step),
        false => String::new(),
    }
}

pub(crate) fn working(
    frame: &mut Frame,
    area: Rect,
    pct: u16,
    turn: usize,
    step: &str,
    notes: &[String],
    foot: &str,
) {
    let keys = hint_row(foot);
    let [_, meter, name, _, body, tail] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(u16::from(!keys.is_empty())),
    ])
    .areas(area);
    frame.render_widget(bar(pct, meter.width), meter);
    if !boxed() {
        frame.render_widget(
            Line::from(vec![
                Span::styled(TURNING[turn % TURNING.len()], Style::new().fg(HIGHLIGHT)),
                Span::raw(" "),
                Span::styled(step.to_string(), Style::new().fg(HIGHLIGHT).bold()),
            ]),
            name,
        );
    }
    let room = usize::from(body.height);
    let first = notes.len().saturating_sub(room);
    frame.render_widget(Paragraph::new(notes[first..].join("\n")).dim(), body);
    frame.render_widget(Line::from(keys.dim()), tail);
}

pub(crate) const COLD: (u8, u8, u8) = (0x5a, 0x56, 0xe0);
pub(crate) const HOT: (u8, u8, u8) = (0xee, 0x6f, 0xf8);

pub(crate) fn crept(pct: u16, flight: u16, within: u32) -> u16 {
    let left = HELD.powi(within.min(400) as i32);
    pct + (f64::from(flight) * (1.0 - left)) as u16
}

const HELD: f64 = 0.97;

pub(crate) fn bar<'a>(pct: u16, width: u16) -> Line<'a> {
    let label = format!(" {pct:>3}%");
    let room = usize::from(width).saturating_sub(label.chars().count());
    let done = room * usize::from(pct.min(100)) / 100;
    let mut spans: Vec<Span> = (0..room)
        .map(|at| match at < done {
            true => Span::styled("\u{2588}", Style::new().fg(blend(at, room))),
            false => Span::styled("\u{2588}", Style::new().fg(TRACK)),
        })
        .collect();
    spans.push(Span::styled(label, Style::new().bold()));
    Line::from(spans)
}

pub(crate) const TRACK: Color = Color::DarkGray;

pub(crate) const TURNING: [&str; 10] = [
    "\u{280b}", "\u{2819}", "\u{2839}", "\u{2838}", "\u{283c}", "\u{2834}", "\u{2826}", "\u{2827}",
    "\u{2807}", "\u{280f}",
];

pub(crate) fn step_line(turn: usize, step: &str) -> String {
    format!("{} {step}", TURNING[turn % TURNING.len()])
}

pub(crate) fn blend(at: usize, room: usize) -> Color {
    let step = |cold: u8, hot: u8| {
        let (cold, hot) = (i32::from(cold), i32::from(hot));
        (cold + (hot - cold) * at as i32 / room.max(1) as i32) as u8
    };
    Color::Rgb(
        step(COLD.0, HOT.0),
        step(COLD.1, HOT.1),
        step(COLD.2, HOT.2),
    )
}

pub(crate) fn tinted(label: &str) -> Vec<Span<'static>> {
    label
        .chars()
        .map(|letter| {
            let colour = match letter {
                _ if letter.is_ascii_digit() => Color::White,
                _ => HIGHLIGHT,
            };
            Span::styled(letter.to_string(), Style::new().fg(colour).bold())
        })
        .collect()
}
