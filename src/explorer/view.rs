//! Drawing the gallery and the creature pages.
//!
//! Every screen is built as lines of styled text and then laid on the frame,
//! so a page is a plain function of the model and can be tested without a
//! terminal.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};
use unicode_width::UnicodeWidthStr;

use super::intro;
use super::model::{Model, Screen, Tab};
use crate::creatures::Creature;
use crate::host;
use crate::render::{GREEN, RED, SNOW, truncate};
use crate::tui::{self, Lines, beside, boxed, hanging, join_fit, rgb, wrap};

/// Portraits sit beside the text from this width up.
const WIDE: usize = 100;

const GALLERY_KEYS: &[&str] = &["↑/↓ choose", "enter open", "/ search", "? help", "q quit"];
const DETAIL_KEYS: &[&str] = &[
    "←/→ guest",
    "tab section",
    "↑/↓ scroll",
    "/ search",
    "g gallery",
    "? help",
    "q quit",
];
const SEARCH_KEYS: &[&str] = &["type to search", "↑/↓ choose", "enter open", "esc cancel"];
const FULL_HELP: &[&str] = &[
    "Guests: ←/→ h/l p/n",
    "Sections: tab/shift+tab or 1-7",
    "Scroll: ↑/↓ j/k space pgup/pgdn home/end",
    "Myth or Fact?: r reveals verdicts",
    "Gallery: g or esc",
    "Search: /",
    "Help: ?",
    "Quit: q or ctrl+c",
];

// ── Colours ────────────────────────────────────────────────────────────────

/// A foreground colour, or no colour at all under `--no-color`.
pub fn fg(m: &Model, colour: Color) -> Style {
    m.ink.fg(colour)
}

/// An emoji and the space after it, or nothing under `--no-color`, which
/// promises plain output: no emoji, and ASCII borders (see `plain`).
pub fn icon(m: &Model, emoji: &str) -> String {
    m.ink.icon(emoji)
}

/// The style of the title bar.
pub fn title_style(m: &Model) -> Style {
    m.ink.title()
}

/// The colours one creature is drawn in.
struct Palette {
    primary: Style,
    accent: Style,
    body: Style,
    /// The active tab and the engraving's badge: dark text on the primary.
    badge: Style,
}

/// A creature's colours. The engraved packs are always drawn in sepia, like
/// the plates of the books they came from.
fn palette(m: &Model, c: &Creature) -> Palette {
    let engraving = m.vault.pack_of(c).is_some_and(|p| p.is_engraving());
    let (primary, accent, body) = if engraving {
        (rgb("#E8D5B0"), rgb("#A68A64"), rgb("#CDB891"))
    } else {
        (rgb(&c.theme.primary), rgb(&c.theme.accent), rgb(SNOW))
    };
    let badge = m.ink.badge(primary);
    Palette {
        primary: fg(m, primary),
        accent: fg(m, accent),
        body: fg(m, body),
        badge,
    }
}

fn dim(m: &Model) -> Style {
    m.ink.dim()
}

fn heading_style(m: &Model) -> Style {
    m.ink.heading()
}

fn host_style(m: &Model) -> Style {
    m.ink.host()
}

// ── Building blocks ────────────────────────────────────────────────────────

fn keys(m: &Model, items: &[&str], sep: &str, width: usize) -> Lines {
    let spans = items
        .iter()
        .map(|k| Span::styled(k.to_string(), dim(m)))
        .collect();
    join_fit(spans, sep, width)
}

/// The slice of `lines` that fits `height`, starting at `offset`, with
/// markers where there is more to scroll to.
fn viewport(m: &Model, mut lines: Lines, offset: usize, height: usize) -> Lines {
    if height == 0 {
        return Vec::new();
    }
    if lines.len() <= height {
        return lines;
    }
    let offset = offset.min(lines.len() - height);
    let more_below = offset + height < lines.len();
    let mut visible: Lines = lines.drain(offset..offset + height).collect();
    // The markers each take a row. With fewer than three there would be
    // nothing left to read between them, so the text has the rows instead.
    if height < 3 {
        return visible;
    }
    if offset > 0 {
        visible[0] = Line::styled("  ▲ more above", dim(m));
    }
    if more_below {
        visible[height - 1] = Line::styled("  ▼ more below", dim(m));
    }
    visible
}

fn art_width(art: &str) -> usize {
    art.lines().map(UnicodeWidthStr::width).max().unwrap_or(0)
}

/// A creature's portrait as lines. On an engraving, snow drifts through
/// the empty parts of the plate.
fn portrait(m: &Model, c: &Creature, p: &Palette, snowing: bool) -> Lines {
    let width = art_width(&c.ascii);
    let rows: Vec<&str> = c.ascii.lines().collect();
    rows.iter()
        .enumerate()
        .map(|(y, row)| {
            if !snowing {
                return Line::styled(row.to_string(), p.primary);
            }
            // Snow falls round the figure, never through it.
            let first = row.chars().take_while(|ch| *ch == ' ').count();
            let last = row.trim_end().chars().count();
            let mut spans = Vec::new();
            let mut chars = row.chars();
            for x in 0..width {
                let ink = chars.next().unwrap_or(' ');
                let clear = x + 1 < first || x > last;
                let flake = clear
                    .then(|| intro::flake(m.snow_seed, x, y, rows.len(), m.frame))
                    .flatten();
                spans.push(match flake {
                    Some(f) => Span::styled(f.to_string(), dim(m)),
                    None => Span::styled(ink.to_string(), p.primary),
                });
            }
            Line::from(spans)
        })
        .collect()
}

// ── Shared chrome ──────────────────────────────────────────────────────────

fn title_bar(m: &Model, right: &str) -> Line<'static> {
    let tree = icon(m, "🎄");
    let title = format!("{tree}TERMINAL OF TINSEL {}", tree.trim_end());
    tui::title_bar(m.ink, title.trim_end(), right, m.size().0)
}

fn footer(m: &Model, normal: &[&str]) -> Lines {
    let (w, _) = m.size();
    if m.searching {
        let mut lines = vec![Line::from(vec![
            Span::styled("/", heading_style(m)),
            Span::raw(format!(" {}▌", m.query)),
        ])];
        lines.extend(keys(m, SEARCH_KEYS, " · ", w));
        return lines;
    }
    if m.show_help {
        return keys(m, FULL_HELP, "   ", w);
    }
    keys(m, normal, " · ", w)
}

fn pack_name<'a>(m: &'a Model, c: &'a Creature) -> &'a str {
    m.vault.pack_of(c).map_or(c.pack.as_str(), |p| &p.name)
}

// ── The gallery ────────────────────────────────────────────────────────────

fn gallery_header(m: &Model) -> Lines {
    let n = m.vault.creatures.len();
    let guests = if n == 1 { "guest" } else { "guests" };
    vec![title_bar(m, &format!("{n} {guests} tonight"))]
}

/// The gallery's list, and the line the selected creature is on.
fn gallery_lines(m: &Model, width: usize) -> (Lines, usize) {
    let mut lines: Lines = Vec::new();
    let mut cursor_line = 0;
    let mut pack = "";
    for (i, c) in m.vault.creatures.iter().enumerate() {
        if c.pack != pack {
            pack = &c.pack;
            if !lines.is_empty() {
                lines.push(Line::default());
            }
            lines.push(Line::styled(
                format!("── {} ──", pack_name(m, c)),
                heading_style(m),
            ));
        }
        let p = palette(m, c);
        let selected = i == m.cursor;
        let mut name_style = p.primary;
        if selected {
            name_style = name_style.add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
            cursor_line = lines.len();
        }
        let mut spans = vec![
            Span::styled(if selected { "▶ " } else { "  " }, fg(m, rgb(RED))),
            Span::styled(format!("{}{}", icon(m, &c.emoji), c.name), name_style),
        ];
        if let Some(year) = c.debut.year {
            spans.push(Span::styled(format!(" ({year})"), dim(m)));
        }
        lines.push(Line::from(spans));
        lines.push(Line::styled(
            format!(
                "    {}",
                truncate(&c.description, width.saturating_sub(6).max(10))
            ),
            host_style(m),
        ));
    }
    (lines, cursor_line)
}

/// The selected creature's portrait and debut, boxed, beside the list.
fn preview(m: &Model, c: &Creature, width: usize) -> Lines {
    let p = palette(m, c);
    let inner = width.saturating_sub(4);
    let mut lines = vec![Line::styled(
        format!("{}{}", icon(m, &c.emoji), c.name),
        p.primary.add_modifier(Modifier::BOLD),
    )];
    lines.extend(wrap(
        &format!("First appeared: {}", c.first_appearance()),
        inner,
        fg(m, rgb(GREEN)).add_modifier(Modifier::ITALIC),
    ));
    lines.push(Line::default());
    lines.extend(portrait(m, c, &p, false));
    boxed(lines, inner, 1, 0, p.accent)
}

fn gallery_body(m: &Model, height: usize) -> Lines {
    let (w, _) = m.size();
    if m.searching {
        return search_results(m, w, height);
    }
    let wide = w >= WIDE;
    let list_width = if wide { w / 2 } else { w };
    let (lines, cursor_line) = gallery_lines(m, list_width);
    // Keep the selected name and its description above the "more below"
    // marker, but never under the "more above" one.
    let offset = if cursor_line + 2 >= height {
        (cursor_line + 3)
            .saturating_sub(height)
            .min(cursor_line.saturating_sub(1))
    } else {
        0
    };
    let list = viewport(m, lines, offset, height);
    if !wide {
        return list;
    }
    let mut side = preview(m, &m.vault.creatures[m.cursor], w - list_width - 2);
    side.truncate(height);
    beside(list, list_width, 0, side)
}

// ── A creature's page ──────────────────────────────────────────────────────

fn detail_header(m: &Model) -> Lines {
    let c = m.current();
    let p = palette(m, c);
    let (w, h) = m.size();
    // Squeeze the header on short terminals so the content still fits.
    let compact = h < 32;
    let tiny = h < 22;

    let right = format!(
        "Guest {} of {} · {}",
        m.index + 1,
        m.vault.creatures.len(),
        pack_name(m, c)
    );
    let mut lines = vec![title_bar(m, &right)];
    if !compact {
        lines.push(Line::default());
    }

    let mut name = vec![Span::styled(
        format!(
            "{}{}",
            icon(m, &c.emoji).replace(' ', "  "),
            c.name.to_uppercase()
        ),
        p.primary.add_modifier(Modifier::BOLD),
    )];
    if m.engraving() {
        let badge = match c.debut.year {
            Some(year) => format!(" ENGRAVER'S INK · {year} "),
            None => " ENGRAVER'S INK ".to_string(),
        };
        name.push(Span::raw("  "));
        name.push(Span::styled(badge, p.badge));
    }
    lines.push(Line::from(name));

    let caption = p.accent.add_modifier(Modifier::ITALIC);
    if tiny {
        lines.push(Line::styled(truncate(&c.description, w), caption));
    } else {
        lines.extend(wrap(&c.description, w, caption));
    }
    let intro = format!("{}{}: “{}”", icon(m, "📺"), host::NAME, host::intro(c));
    if compact && !tiny {
        lines.push(Line::styled(truncate(&intro, w), host_style(m)));
    } else if !tiny {
        lines.extend(wrap(&intro, w, host_style(m)));
    }
    if !compact {
        lines.push(Line::default());
    }
    lines.extend(tab_bar(m, &p));
    lines
}

fn tab_bar(m: &Model, p: &Palette) -> Lines {
    let (w, _) = m.size();
    let labels = m
        .tabs()
        .into_iter()
        .enumerate()
        .map(|(i, t)| {
            let label = format!(" {} {} ", i + 1, t.name(w >= WIDE));
            if i == m.tab {
                Span::styled(label, p.badge)
            } else {
                Span::styled(label, dim(m))
            }
        })
        .collect();
    join_fit(labels, " ", w)
}

/// The scrollable part of a creature's page.
pub fn body_lines(m: &Model) -> Lines {
    let c = m.current();
    let p = palette(m, c);
    let (w, _) = m.size();
    let tab = m.current_tab();
    let art = art_width(&c.ascii);
    let mut lines = vec![Line::default()];
    if !c.ascii.is_empty() && w >= WIDE {
        let plate = boxed(portrait(m, c, &p, m.engraving()), art, 2, 1, p.accent);
        let plate_width = art + 6;
        let text = tab_content(m, tab, c, &p, w - plate_width - 3);
        lines.extend(beside(plate, plate_width, 3, text));
    } else {
        lines.extend(tab_content(m, tab, c, &p, w));
        if !c.ascii.is_empty() && tab == Tab::Facts && w >= art + 6 {
            // Narrow screens: the facts first, the portrait underneath.
            lines.push(Line::default());
            lines.extend(boxed(
                portrait(m, c, &p, m.engraving()),
                art,
                2,
                1,
                p.accent,
            ));
        }
    }
    lines
}

/// How many lines of the page fit between the header and the footer.
pub fn body_height(m: &Model) -> usize {
    let (_, h) = m.size();
    h.saturating_sub(detail_header(m).len() + footer(m, DETAIL_KEYS).len() + 1)
}

fn tab_content(m: &Model, tab: Tab, c: &Creature, p: &Palette, width: usize) -> Lines {
    let heading = |title: &str| {
        vec![
            Line::styled(title.to_string(), heading_style(m)),
            Line::default(),
        ]
    };
    let bullets = |items: &[String], style: Style| -> Lines {
        items
            .iter()
            .flat_map(|item| hanging("•", p.accent, item, width, style))
            .collect()
    };
    let meta = fg(m, rgb(GREEN)).add_modifier(Modifier::ITALIC);
    match tab {
        Tab::Facts => {
            let mut out = heading("Festive Facts");
            out.extend(bullets(&c.facts, p.body));
            out.push(Line::default());
            out.extend(wrap(&format!("Origin: {}", c.origin), width, meta));
            out.extend(wrap(
                &format!("First appeared: {}", c.first_appearance()),
                width,
                meta,
            ));
            out
        }
        Tab::Legend => {
            let mut out = heading("The Legend");
            out.extend(wrap(&c.legend, width, p.body));
            out
        }
        Tab::Film => {
            let mut out = heading("On Screen");
            if let Some(f) = &c.film {
                out.push(Line::styled(
                    format!("{} ({})", f.title, f.year),
                    p.primary.add_modifier(Modifier::BOLD),
                ));
                out.push(Line::default());
                out.extend(wrap(&format!("Directed by {}", f.director), width, p.body));
                out.extend(wrap(
                    &format!("Starring {} as {}", f.star, c.name),
                    width,
                    p.body,
                ));
                if let Some(note) = &f.note {
                    out.push(Line::default());
                    out.extend(wrap(note, width, meta));
                }
            }
            out
        }
        Tab::Myths => {
            let mut out = heading("Myth or Fact?");
            for (i, myth) in c.myths.iter().enumerate() {
                out.extend(hanging(
                    &format!("{}.", i + 1),
                    p.accent,
                    &myth.claim,
                    width,
                    p.body.add_modifier(Modifier::BOLD),
                ));
                if m.revealed {
                    let (verdict, colour) = if myth.is_true {
                        ("✔ TRUE", GREEN)
                    } else {
                        ("✘ FALSE", RED)
                    };
                    out.push(Line::styled(
                        format!("    {verdict}"),
                        fg(m, rgb(colour)).add_modifier(Modifier::BOLD),
                    ));
                    out.extend(hanging(
                        "  ",
                        Style::new(),
                        &myth.explanation,
                        width,
                        dim(m),
                    ));
                }
                out.push(Line::default());
            }
            let hint = if m.revealed {
                "Press r to hide the verdicts."
            } else {
                "True or false? Make your guesses, then press r for the verdicts."
            };
            out.extend(wrap(hint, width, host_style(m)));
            out
        }
        Tab::Quotes => {
            let mut out = heading("From the Page");
            for q in &c.quotes {
                out.extend(hanging(
                    "“",
                    p.accent,
                    &format!("{}”", q.text),
                    width,
                    p.body.add_modifier(Modifier::ITALIC),
                ));
                out.extend(hanging(
                    " ",
                    Style::new(),
                    &format!("{}, {}", q.speaker, q.source),
                    width,
                    dim(m),
                ));
                out.push(Line::default());
            }
            out.pop();
            out
        }
        Tab::Stats => {
            let mut out = heading("Stat Card");
            for (label, value) in c.stats.labelled() {
                let v = usize::from(value.min(10));
                out.push(Line::from(vec![
                    Span::styled(format!("  {label:<9} "), p.body),
                    Span::styled("█".repeat(v), p.primary),
                    Span::styled("░".repeat(10 - v), dim(m)),
                    Span::styled(format!(" {value:>2}"), p.body),
                ]));
            }
            out.push(Line::default());
            out.push(Line::styled(
                "Powers",
                fg(m, rgb(GREEN)).add_modifier(Modifier::BOLD),
            ));
            out.extend(bullets(&c.powers, p.body));
            out.push(Line::default());
            out.push(Line::styled(
                "Weaknesses",
                fg(m, rgb(RED)).add_modifier(Modifier::BOLD),
            ));
            out.extend(bullets(&c.weaknesses, p.body));
            out
        }
        Tab::Legacy => {
            let mut out = heading("Legacy");
            out.extend(bullets(&c.legacy, p.body));
            out
        }
    }
}

// ── Search ─────────────────────────────────────────────────────────────────

fn search_results(m: &Model, width: usize, height: usize) -> Lines {
    if m.query.trim().is_empty() {
        return wrap(
            "Search names, nicknames, facts and legends. Try \"birch\", \"porridge\" or \"iceland\".",
            width,
            host_style(m),
        );
    }
    if m.hits.is_empty() {
        return vec![Line::styled(
            "Nobody at the door by that name... try another search.",
            host_style(m),
        )];
    }
    let mut lines: Lines = Vec::new();
    for (i, hit) in m.hits.iter().enumerate() {
        let c = &m.vault.creatures[hit.index];
        let selected = i == m.hit_cursor;
        let mut name = palette(m, c).primary;
        if selected {
            name = name.add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
        }
        lines.push(Line::from(vec![
            Span::styled(if selected { "▶ " } else { "  " }, fg(m, rgb(RED))),
            Span::styled(format!("{}{}", icon(m, &c.emoji), c.name), name),
        ]));
        lines.push(Line::styled(
            format!(
                "    {}",
                truncate(
                    &format!("({}) {}", hit.field, hit.snippet),
                    width.saturating_sub(6).max(10)
                )
            ),
            dim(m),
        ));
    }
    let offset = (m.hit_cursor * 2 + 3).saturating_sub(height);
    viewport(m, lines, offset, height)
}

// ── Laying it on the frame ─────────────────────────────────────────────────

/// Draws the current screen.
pub fn draw(m: &Model, frame: &mut Frame) {
    draw_screen(m, frame);
    if !m.ink.colour {
        crate::plain::plainify(frame.buffer_mut());
    }
}

fn draw_screen(m: &Model, frame: &mut Frame) {
    let area = frame.area();
    let buf = frame.buffer_mut();
    let (header, body, footer) = match m.screen {
        Screen::Intro => return intro::draw(m, buf, area),
        Screen::Gallery => {
            let (header, footer) = (gallery_header(m), footer(m, GALLERY_KEYS));
            let height = usize::from(area.height).saturating_sub(header.len() + footer.len() + 2);
            let body = gallery_body(m, height);
            (header, body, footer)
        }
        Screen::Detail => {
            let height = body_height(m);
            let body = if m.searching {
                let mut lines = vec![Line::default()];
                lines.extend(search_results(m, m.size().0, height.saturating_sub(1)));
                lines
            } else {
                viewport(m, body_lines(m), m.scroll, height)
            };
            (detail_header(m), body, footer(m, DETAIL_KEYS))
        }
    };

    // The header at the top, the keys at the bottom, the page in between.
    let rows = |n: usize| u16::try_from(n).unwrap_or(u16::MAX).min(area.height);
    let head = rows(header.len());
    let foot = rows(footer.len()).min(area.height - head);
    let gap = u16::from(m.screen == Screen::Gallery);
    let top = (head + gap).min(area.height - foot);
    let middle = area.height - top - foot;
    let place = |y: u16, height: u16| Rect::new(area.x, area.y + y, area.width, height);
    Paragraph::new(header).render(place(0, head), buf);
    Paragraph::new(body).render(place(top, middle), buf);
    Paragraph::new(footer).render(place(area.height - foot, foot), buf);
}
