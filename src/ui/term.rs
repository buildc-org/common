use crate::ui::choose::*;
use crate::ui::chrome::*;
use ratatui::backend::Backend;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::crossterm::terminal;
use ratatui::layout::Rect;
use ratatui::{DefaultTerminal, Frame, TerminalOptions, Viewport};
use std::io::IsTerminal;
use std::sync::OnceLock;

pub(crate) const NARROWEST: u16 = 80;

pub(crate) fn colour() -> bool {
    std::io::stdout().is_terminal()
}

pub fn width() -> usize {
    if !colour() {
        return usize::from(NARROWEST);
    }
    std::env::var("COLUMNS")
        .ok()
        .and_then(|cols| parse_width(&cols).map(usize::from))
        .or_else(|| {
            terminal::size()
                .map(|(cols, _)| usize::from(cols))
                .ok()
                .filter(|cols| *cols > 0)
        })
        .unwrap_or(usize::from(NARROWEST))
}

pub(crate) fn parse_width(width: &str) -> Option<u16> {
    width.parse().ok().filter(|width| *width > 0)
}

pub fn bold(text: &str) -> String {
    ratatui::crossterm::style::Stylize::bold(text).to_string()
}

pub(crate) fn height(rows: usize) -> u16 {
    to_cells(rows.min(VISIBLE) + 2)
}

pub(crate) fn to_cells(count: usize) -> u16 {
    u16::try_from(count).unwrap_or(u16::MAX)
}

// An inline viewport needs a size before the first draw, so a terminal that reports none is given a default.
fn give_size() {
    if !unsized_tty(terminal::size().ok()) {
        return;
    }
    let size = libc::winsize {
        ws_row: 24,
        ws_col: NARROWEST,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    // SAFETY: `size` is a live, aligned `libc::winsize` for the ioctl call, so `TIOCSWINSZ` reads a valid structure through the pointer. The kernel checks the `STDOUT_FILENO` descriptor and returns an error when it does not name a terminal.
    unsafe { libc::ioctl(libc::STDOUT_FILENO, libc::TIOCSWINSZ, &size) };
}

pub(crate) fn unsized_tty(size: Option<(u16, u16)>) -> bool {
    !matches!(size, Some((cols, rows)) if cols > 0 && rows > 0)
}

pub(crate) static CHROME: OnceLock<String> = OnceLock::new();

pub fn own_screen(title: impl Into<String>) {
    let _ = CHROME.set(title.into());
}

pub(crate) fn open(height: u16) -> Result<DefaultTerminal, String> {
    give_size();
    let viewport = match CHROME.get() {
        Some(_) => Viewport::Fullscreen,
        None => Viewport::Inline(height),
    };
    ratatui::try_init_with_options(TerminalOptions { viewport }).map_err(|err| err.to_string())
}

pub(crate) fn close(mut terminal: DefaultTerminal) {
    let origin = terminal.get_frame().area().as_position();
    let _ = terminal.clear();
    let _ = terminal.set_cursor_position(origin);
    if let Err(err) = terminal.show_cursor() {
        eprint!("the cursor could not be shown again: {err}\r\n`tput cnorm` shows it\r\n");
    }
    if let Err(err) = terminal::disable_raw_mode() {
        eprint!("the terminal could not leave raw mode: {err}\r\n`stty sane` restores it\r\n");
    }
}

pub(crate) fn inline<T>(
    height: u16,
    body: impl FnOnce(&mut DefaultTerminal) -> Result<T, String>,
) -> Result<T, String> {
    let mut terminal = open(height)?;
    let out = body(&mut terminal);
    close(terminal);
    out
}

pub(crate) fn render<B: Backend>(
    terminal: &mut ratatui::Terminal<B>,
    rows: u16,
    keys: &str,
    head: &str,
    body: impl FnOnce(&mut Frame, Rect),
) -> Result<(), String> {
    render_with_title(
        terminal,
        CHROME.get().map(String::as_str),
        rows,
        keys,
        head,
        body,
    )
}

pub(crate) fn render_with_title<B: Backend>(
    terminal: &mut ratatui::Terminal<B>,
    title: Option<&str>,
    rows: u16,
    keys: &str,
    head: &str,
    body: impl FnOnce(&mut Frame, Rect),
) -> Result<(), String> {
    terminal
        .draw(|frame| {
            in_frame_box(has_box(title), || {
                let area = chrome(frame, title, head, rows, keys);
                body(frame, area);
            });
        })
        .map(|_| ())
        .map_err(|err| err.to_string())
}

pub(crate) fn read_for(ms: u64) -> Result<Option<KeyCode>, String> {
    let ready = event::poll(std::time::Duration::from_millis(ms)).map_err(|err| err.to_string())?;
    match ready {
        true => read(),
        false => Ok(None),
    }
}

pub const INTERRUPTED: &str = "interrupted";

pub(crate) fn revealed() -> bool {
    REVEAL.with(std::cell::Cell::get)
}

pub(crate) fn read() -> Result<Option<KeyCode>, String> {
    let Event::Key(key) = event::read().map_err(|err| err.to_string())? else {
        return Ok(None);
    };
    // A terminal may report Alt as a modifier key event rather than a modifier on the key.
    let alt = key.modifiers.contains(KeyModifiers::ALT)
        || matches!(
            key.code,
            KeyCode::Modifier(event::ModifierKeyCode::LeftAlt | event::ModifierKeyCode::RightAlt)
        );
    match key.kind {
        KeyEventKind::Press | KeyEventKind::Repeat if alt => {
            REVEAL.with(|reveal| reveal.set(true));
            return Ok(None);
        }
        KeyEventKind::Release if alt => {
            REVEAL.with(|reveal| reveal.set(false));
            return Ok(None);
        }
        KeyEventKind::Press | KeyEventKind::Repeat => {
            REVEAL.with(|reveal| reveal.set(false));
        }
        _ => return Ok(None),
    }
    if key.kind != KeyEventKind::Press {
        return Ok(None);
    }
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Err(INTERRUPTED.to_string())
        }
        code => Ok(Some(code)),
    }
}
