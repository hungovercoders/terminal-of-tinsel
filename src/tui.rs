//! What every full-screen page shares: the event loop, the house styles
//! and a few building blocks for laying out lines of styled text.

use std::io;
use std::time::{Duration, Instant};

use ratatui::crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::{DefaultTerminal, Frame};
use unicode_width::UnicodeWidthStr;

use crate::error::{Code, Result, TinselError};
use crate::render::{DIM, GOLD, GREEN, RED, parse_hex};

pub type Lines = Vec<Line<'static>>;

/// A screen that takes over the terminal: the explorer or a game.
pub trait App {
    /// How often the screen animates, if it does.
    fn tick_every(&self) -> Option<Duration> {
        None
    }
    /// Advances the animation by a frame.
    fn on_tick(&mut self) {}
    fn on_key(&mut self, key: KeyEvent);
    fn resize(&mut self, width: u16, height: u16);
    fn draw(&self, frame: &mut Frame);
    fn quitting(&self) -> bool;
}

/// Takes over the terminal and runs the app until it quits. `what` names
/// the screen in the show's own words, for the error if the terminal fails.
pub fn run(app: &mut impl App, what: &str) -> Result<()> {
    let lost = |e: io::Error| {
        TinselError::new(Code::NoTerminal, format!("{what} lost its terminal: {e}"))
            .with_hint("use --json for the data, or tune in from a shell")
    };
    let mut terminal = ratatui::try_init().map_err(lost)?;
    let result = event_loop(&mut terminal, app);
    ratatui::restore();
    result.map_err(lost)
}

/// Draw, wait for a key or the next tick, update, and round again. Unlike
/// Bubble Tea, nothing runs this loop for us: it's ours.
fn event_loop(terminal: &mut DefaultTerminal, app: &mut impl App) -> io::Result<()> {
    let mut last_tick = Instant::now();
    while !app.quitting() {
        let size = terminal.size()?;
        app.resize(size.width, size.height);
        terminal.draw(|frame| app.draw(frame))?;

        // A still screen waits for a key; an animated one for a key or a tick.
        let key_waiting = match app.tick_every() {
            Some(every) => event::poll(every.saturating_sub(last_tick.elapsed()))?,
            None => true,
        };
        if key_waiting {
            // Windows reports key releases too; only presses count.
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                app.on_key(key);
            }
        }
        match app.tick_every() {
            Some(every) if last_tick.elapsed() >= every => {
                app.on_tick();
                last_tick = Instant::now();
            }
            Some(_) => {}
            None => last_tick = Instant::now(),
        }
    }
    Ok(())
}

// ── Colours ────────────────────────────────────────────────────────────────

/// A `#RRGGBB` colour, or the terminal's own if it doesn't parse.
pub fn rgb(hex: &str) -> Color {
    parse_hex(hex).map_or(Color::Reset, |(r, g, b)| Color::Rgb(r, g, b))
}

/// The house styles, in colour or (under `--no-color` and `NO_COLOR`) in the
/// terminal's own colours with only bold, italic and reverse to tell things
/// apart. ratatui has no global colour switch, so every style comes from here.
#[derive(Debug, Clone, Copy)]
pub struct Ink {
    pub colour: bool,
}

impl Ink {
    pub fn fg(self, colour: Color) -> Style {
        if self.colour {
            Style::new().fg(colour)
        } else {
            Style::new()
        }
    }

    /// An emoji and the space after it, or nothing in plain mode, which
    /// promises no emoji and ASCII borders as well as no colour (see `plain`).
    pub fn icon(self, emoji: &str) -> String {
        if self.colour {
            format!("{emoji} ")
        } else {
            String::new()
        }
    }

    pub fn dim(self) -> Style {
        self.fg(rgb(DIM))
    }

    pub fn heading(self) -> Style {
        self.fg(rgb(GOLD)).add_modifier(Modifier::BOLD)
    }

    /// The host's voice.
    pub fn host(self) -> Style {
        self.dim().add_modifier(Modifier::ITALIC)
    }

    pub fn good(self) -> Style {
        self.fg(rgb(GREEN)).add_modifier(Modifier::BOLD)
    }

    pub fn bad(self) -> Style {
        self.fg(rgb(RED)).add_modifier(Modifier::BOLD)
    }

    /// The title bar.
    pub fn title(self) -> Style {
        if self.colour {
            Style::new()
                .fg(rgb(GOLD))
                .bg(Color::Rgb(0x1B, 0x43, 0x32))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::new().add_modifier(Modifier::BOLD | Modifier::REVERSED)
        }
    }

    /// Dark text on a colour: the active tab, a badge.
    pub fn badge(self, background: Color) -> Style {
        if self.colour {
            Style::new()
                .fg(Color::Black)
                .bg(background)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::new().add_modifier(Modifier::BOLD | Modifier::REVERSED)
        }
    }
}

// ── Building blocks ────────────────────────────────────────────────────────

pub fn wrap(text: &str, width: usize, style: Style) -> Lines {
    textwrap::wrap(text, width.max(10))
        .into_iter()
        .map(|l| Line::styled(l.into_owned(), style))
        .collect()
}

/// Wraps text under a marker with a hanging indent.
pub fn hanging(marker: &str, marker_style: Style, text: &str, width: usize, style: Style) -> Lines {
    let lead = format!(" {marker} ");
    let indent = lead.width();
    textwrap::wrap(text, width.saturating_sub(indent).max(10))
        .into_iter()
        .enumerate()
        .map(|(i, l)| {
            let first = if i == 0 {
                Span::styled(lead.clone(), marker_style)
            } else {
                Span::raw(" ".repeat(indent))
            };
            Line::from(vec![first, Span::styled(l.into_owned(), style)])
        })
        .collect()
}

/// Joins short items with a separator, wrapping onto as many lines as the
/// width needs.
pub fn join_fit(items: Vec<Span<'static>>, sep: &str, width: usize) -> Lines {
    let mut lines: Lines = Vec::new();
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut used = 0;
    for item in items {
        let w = item.width();
        if !spans.is_empty() && used + sep.width() + w > width {
            lines.push(Line::from(std::mem::take(&mut spans)));
            used = 0;
        }
        if !spans.is_empty() {
            spans.push(Span::raw(sep.to_string()));
            used += sep.width();
        }
        used += w;
        spans.push(item);
    }
    if !spans.is_empty() {
        lines.push(Line::from(spans));
    }
    lines
}

/// Puts `lines` in a rounded box, `inner` columns wide inside its padding.
pub fn boxed(lines: Lines, inner: usize, pad_x: usize, pad_y: usize, border: Style) -> Lines {
    let span = inner + pad_x * 2;
    let edge = |l: &str, r: &str| Line::styled(format!("{l}{}{r}", "─".repeat(span)), border);
    let row = |line: Line<'static>| {
        let gap = inner.saturating_sub(line.width());
        let mut spans = vec![Span::styled("│", border), Span::raw(" ".repeat(pad_x))];
        spans.extend(line.spans);
        spans.push(Span::raw(" ".repeat(gap + pad_x)));
        spans.push(Span::styled("│", border));
        Line::from(spans)
    };
    let mut out = vec![edge("╭", "╮")];
    out.extend((0..pad_y).map(|_| row(Line::default())));
    out.extend(lines.into_iter().map(row));
    out.extend((0..pad_y).map(|_| row(Line::default())));
    out.push(edge("╰", "╯"));
    out
}

/// Sets two blocks of lines side by side.
pub fn beside(left: Lines, left_width: usize, gap: usize, right: Lines) -> Lines {
    let rows = left.len().max(right.len());
    let mut left = left.into_iter();
    let mut right = right.into_iter();
    (0..rows)
        .map(|_| {
            let l = left.next().unwrap_or_default();
            let pad = left_width.saturating_sub(l.width()) + gap;
            let mut spans = l.spans;
            spans.push(Span::raw(" ".repeat(pad)));
            spans.extend(right.next().unwrap_or_default().spans);
            Line::from(spans)
        })
        .collect()
}

/// A title bar: the title on the left, a status on the right, or the title
/// alone when there isn't room for both.
pub fn title_bar(ink: Ink, title: &str, right: &str, width: usize) -> Line<'static> {
    let left = Span::styled(format!(" {title} "), ink.title());
    let used = left.width() + right.width();
    if used + 2 > width {
        return Line::from(left);
    }
    Line::from(vec![
        left,
        Span::raw(" ".repeat(width - used)),
        Span::styled(right.to_string(), ink.dim()),
    ])
}

/// Draws an app at a size into ratatui's in-memory terminal and returns the
/// screen as text, for tests.
#[cfg(test)]
pub fn snapshot(app: &mut impl App, width: u16, height: u16) -> String {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    app.resize(width, height);
    terminal.draw(|frame| app.draw(frame)).unwrap();
    terminal
        .backend()
        .buffer()
        .content()
        .chunks(usize::from(width))
        .map(|row| {
            // A double-width emoji takes two cells; the second is a filler.
            let mut text = String::new();
            let mut skip = false;
            for cell in row {
                if !std::mem::take(&mut skip) {
                    text.push_str(cell.symbol());
                    skip = cell.symbol().width() == 2;
                }
            }
            text.trim_end().to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_fit_wraps_to_the_width() {
        let items = || {
            ["one", "two", "three", "four"]
                .iter()
                .map(|s| Span::raw(s.to_string()))
                .collect::<Vec<_>>()
        };
        assert_eq!(join_fit(items(), " · ", 80).len(), 1);
        let narrow = join_fit(items(), " · ", 11);
        assert_eq!(narrow.len(), 3);
        assert!(narrow.iter().all(|l| l.width() <= 11));
        assert!(join_fit(Vec::new(), " ", 10).is_empty());
    }

    #[test]
    fn boxes_are_square() {
        let lines = vec![Line::raw("ab"), Line::raw("abcd")];
        let out = boxed(lines, 4, 2, 1, Style::new());
        assert_eq!(out.len(), 6);
        assert!(out.iter().all(|l| l.width() == 10), "{out:?}");
    }

    #[test]
    fn beside_pads_the_left_column() {
        let out = beside(
            vec![Line::raw("a")],
            3,
            2,
            vec![Line::raw("x"), Line::raw("y")],
        );
        let text: Vec<String> = out.iter().map(|l| l.to_string()).collect();
        assert_eq!(text, ["a    x", "     y"]);
    }
}
