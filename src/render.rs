//! Drawing for the terminal: colours, boxes, the big numbers and the tree.
//!
//! Everything here returns a `String`, so it can be tested without a terminal.
//! In plain mode (`--no-color`, `NO_COLOR` or a pipe) there's no colour, no
//! emoji and only ASCII borders.

use jiff::civil::Date;
use owo_colors::{OwoColorize, Style};
use unicode_width::UnicodeWidthStr;

use crate::calendar::{self, Note};
use crate::creatures::{Creature, Vault};
use crate::host;

/// The house colours of Channel 25.
pub const RED: &str = "#E63946";
pub const GREEN: &str = "#52B788";
pub const GOLD: &str = "#E9C46A";
pub const SNOW: &str = "#F1FAEE";
pub const DIM: &str = "#8D99AE";

/// How to draw: in colour or plain, and how wide.
#[derive(Debug, Clone, Copy)]
pub struct Painter {
    pub plain: bool,
    pub width: usize,
}

impl Painter {
    pub fn new(plain: bool, width: usize) -> Self {
        Self {
            plain,
            width: width.clamp(40, 100),
        }
    }

    fn style(&self, hex: &str, bold: bool, italic: bool) -> Style {
        let mut s = Style::new();
        if let Some((r, g, b)) = parse_hex(hex) {
            s = s.truecolor(r, g, b);
        }
        if bold {
            s = s.bold();
        }
        if italic {
            s = s.italic();
        }
        s
    }

    /// Colours text, or leaves it alone in plain mode.
    pub fn paint(&self, text: &str, hex: &str) -> String {
        self.styled(text, hex, false, false)
    }

    pub fn bold(&self, text: &str, hex: &str) -> String {
        self.styled(text, hex, true, false)
    }

    pub fn italic(&self, text: &str, hex: &str) -> String {
        self.styled(text, hex, false, true)
    }

    fn styled(&self, text: &str, hex: &str, bold: bool, italic: bool) -> String {
        if self.plain || text.is_empty() {
            text.to_string()
        } else {
            text.style(self.style(hex, bold, italic)).to_string()
        }
    }

    /// An emoji and a space, or nothing in plain mode.
    pub fn glyph(&self, emoji: &str) -> String {
        if self.plain {
            String::new()
        } else {
            format!("{emoji} ")
        }
    }

    /// A seasonal note, with its emoji unless plain.
    fn note(&self, n: &Note) -> String {
        format!("{}{}", self.glyph(n.emoji), n.text)
    }

    /// Wraps text to `width` and paints every line, so colour codes never
    /// confuse the wrapping.
    fn wrap(&self, text: &str, width: usize, paint: impl Fn(&str) -> String) -> Vec<String> {
        textwrap::wrap(text, width.max(10))
            .iter()
            .map(|line| paint(line))
            .collect()
    }

    /// A section heading ruled across the page.
    fn heading(&self, title: &str, hex: &str) -> String {
        let (rule, lead) = if self.plain {
            ("-", "--")
        } else {
            ("─", "──")
        };
        let used = title.width() + 4;
        let fill = rule.repeat(self.text_width().saturating_sub(used));
        format!(
            "{} {} {}",
            self.paint(lead, hex),
            self.bold(title, hex),
            self.paint(&fill, hex)
        )
    }

    /// The width for running text.
    pub fn text_width(&self) -> usize {
        self.width.min(80)
    }

    /// Draws lines inside a rounded box.
    fn boxed(&self, lines: &[String], inner: usize, hex: &str) -> String {
        let (tl, tr, bl, br, h, v) = if self.plain {
            ("+", "+", "+", "+", "-", "|")
        } else {
            ("╭", "╮", "╰", "╯", "─", "│")
        };
        let edge = h.repeat(inner + 2);
        let mut out = vec![self.paint(&format!("{tl}{edge}{tr}"), hex)];
        for line in lines {
            let pad = inner.saturating_sub(visible_width(line));
            out.push(format!(
                "{} {}{} {}",
                self.paint(v, hex),
                line,
                " ".repeat(pad),
                self.paint(v, hex)
            ));
        }
        out.push(self.paint(&format!("{bl}{edge}{br}"), hex));
        out.join("\n") + "\n"
    }

    fn bullet(&self) -> &'static str {
        if self.plain { "-" } else { "•" }
    }
}

/// Parses `#RRGGBB`.
pub fn parse_hex(hex: &str) -> Option<(u8, u8, u8)> {
    let h = hex.strip_prefix('#')?;
    if h.len() != 6 {
        return None;
    }
    let byte = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).ok();
    Some((byte(0)?, byte(2)?, byte(4)?))
}

/// The width of a string on screen, ignoring colour codes.
pub fn visible_width(s: &str) -> usize {
    let mut plain = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            // Skip a CSI sequence: ESC [ ... final byte in @..~
            for c in chars.by_ref() {
                if ('@'..='~').contains(&c) && c != '[' {
                    break;
                }
            }
        } else {
            plain.push(c);
        }
    }
    plain.width()
}

/// Cuts text to fit `max` columns, with an ellipsis.
fn truncate(text: &str, max: usize) -> String {
    if text.width() <= max {
        return text.to_string();
    }
    let mut out = String::new();
    for c in text.chars() {
        if out.width() + 2 > max {
            break;
        }
        out.push(c);
    }
    out.trim_end().to_string() + "…"
}

// ── The welcome screen ─────────────────────────────────────────────────────

/// What you see with no command: the host, the countdown and where to start.
pub fn welcome(p: &Painter, vault: &Vault, today: Date, greeting: &str) -> String {
    let w = p.text_width();
    let mut out = vec![
        p.bold(&format!("{}TERMINAL OF TINSEL", p.glyph("🎄")), GOLD),
        p.italic(
            &format!(
                "{}'s Midwinter Special, live in your terminal",
                host::CHANNEL
            ),
            DIM,
        ),
        String::new(),
    ];
    out.extend(p.wrap(
        &format!("{}{}: “{greeting}”", p.glyph("📺"), host::NAME),
        w,
        |l| p.italic(l, GOLD),
    ));
    out.push(String::new());
    let sleeps = calendar::sleeps_until_christmas(today);
    let line = match sleeps {
        0 => "It's Christmas Day!".to_string(),
        1 => "1 sleep until Christmas.".to_string(),
        n => format!("{n} sleeps until Christmas."),
    };
    out.push(p.bold(&line, RED));
    if let Some((night, c, _)) = calendar::advent_guest(today, vault) {
        out.push(p.paint(
            &format!(
                "Behind door {}: {}{}",
                night.night,
                p.glyph(&c.emoji),
                c.name
            ),
            &c.theme.primary,
        ));
    }
    out.push(String::new());
    out.push(p.heading("START HERE", GREEN));
    for (cmd, what) in [
        ("list", "meet tonight's guests"),
        ("creature krampus", "read a creature's page"),
        ("random --daily", "the Fact of the Night"),
        ("countdown", "sleeps until Christmas"),
        ("advent", "open the advent calendar"),
    ] {
        out.push(format!(
            "  {}  {}",
            p.bold(&format!("terminal-of-tinsel {cmd:<17}"), SNOW),
            p.paint(what, DIM)
        ));
    }
    out.join("\n") + "\n"
}

// ── The roster ─────────────────────────────────────────────────────────────

/// Every creature, grouped by pack.
pub fn list(p: &Painter, vault: &Vault) -> String {
    let w = p.text_width();
    let mut out = vec![p.bold(
        &format!(
            "{}TONIGHT'S GUESTS · {} creatures",
            p.glyph("🎄"),
            vault.creatures.len()
        ),
        GOLD,
    )];
    let name_w = vault
        .creatures
        .iter()
        .map(|c| c.name.width())
        .max()
        .unwrap_or(10)
        .min(24);
    for (pack, creatures) in vault.by_pack() {
        out.push(String::new());
        out.push(p.heading(
            &format!("{} ({})", pack.name.to_uppercase(), pack.id),
            GREEN,
        ));
        out.extend(p.wrap(&pack.description, w, |l| p.italic(l, DIM)));
        for c in creatures {
            let name = format!("{:<name_w$}", truncate(&c.name, name_w));
            let room = w.saturating_sub(name_w + if p.plain { 4 } else { 7 });
            out.push(format!(
                "  {}{}  {}",
                p.glyph(&c.emoji),
                p.bold(&name, &c.theme.primary),
                p.paint(&truncate(&c.description, room), DIM)
            ));
        }
    }
    out.push(String::new());
    out.push(p.paint(
        "Meet one: terminal-of-tinsel creature <name>   Only some: --pack nordic,dickens",
        DIM,
    ));
    out.join("\n") + "\n"
}

// ── A creature's page ──────────────────────────────────────────────────────

/// A creature's full page. With `reveal`, the myth verdicts are shown.
pub fn creature(p: &Painter, c: &Creature, pack_name: &str, reveal: bool) -> String {
    let w = p.text_width();
    let (primary, accent) = (c.theme.primary.as_str(), c.theme.accent.as_str());
    let b = p.bullet();
    let mut out = vec![p.bold(
        &format!("{}{}", p.glyph(&c.emoji), c.name.to_uppercase()),
        primary,
    )];
    out.extend(p.wrap(&c.description, w, |l| p.italic(l, SNOW)));
    let first = match (&c.debut.creator, c.debut.year, &c.debut.era) {
        (Some(who), Some(year), _) => format!("{}, {} ({year})", c.debut.title, who),
        (_, _, Some(era)) => era.clone(),
        (_, Some(year), _) => year.to_string(),
        _ => c.debut.title.clone(),
    };
    let meta = format!("{} · first appeared: {first} · pack: {pack_name}", c.origin);
    out.extend(p.wrap(&meta, w, |l| p.paint(l, DIM)));
    out.push(String::new());
    out.extend(p.wrap(
        &format!("{}{}: “{}”", p.glyph("📺"), host::NAME, host::intro(c)),
        w,
        |l| p.italic(l, GOLD),
    ));

    out.push(String::new());
    out.push(p.heading("FACTS", accent));
    for fact in &c.facts {
        out.extend(hanging(p, b, fact, w, |l| l.to_string()));
    }

    out.push(String::new());
    out.push(p.heading("THE LEGEND", accent));
    out.extend(p.wrap(&c.legend, w, |l| l.to_string()));

    if !c.myths.is_empty() {
        out.push(String::new());
        out.push(p.heading("MYTH OR FACT?", accent));
        for (i, m) in c.myths.iter().enumerate() {
            out.extend(hanging(p, &format!("{}.", i + 1), &m.claim, w, |l| {
                p.bold(l, SNOW)
            }));
            if reveal {
                let (mark, word, hex) = match (m.is_true, p.plain) {
                    (true, false) => ("✔", "TRUE", GREEN),
                    (false, false) => ("✘", "FALSE", RED),
                    (true, true) => ("", "TRUE", GREEN),
                    (false, true) => ("", "FALSE", RED),
                };
                let verdict = format!("{mark}{}{word}", if mark.is_empty() { "" } else { " " });
                let text = format!("{verdict}: {}", m.explanation);
                out.extend(hanging(p, "", &text, w, |l| {
                    l.replacen(&verdict, &p.bold(&verdict, hex), 1)
                }));
            }
        }
        if !reveal {
            out.push(p.italic(
                "   True or false? Run again with --reveal for the verdicts.",
                DIM,
            ));
        }
    }

    if !c.quotes.is_empty() {
        out.push(String::new());
        out.push(p.heading("FROM THE PAGE", accent));
        for q in &c.quotes {
            out.extend(hanging(p, "", &format!("“{}”", q.text), w, |l| {
                p.italic(l, SNOW)
            }));
            out.extend(hanging(
                p,
                "",
                &format!("  {}, {}", q.speaker, q.source),
                w,
                |l| p.paint(l, DIM),
            ));
        }
    }

    if let Some(f) = &c.film {
        out.push(String::new());
        out.push(p.heading("ON SCREEN", accent));
        let mut text = format!(
            "{} ({}), directed by {}, with {} as {}.",
            f.title, f.year, f.director, f.star, c.name
        );
        if let Some(note) = &f.note {
            text.push(' ');
            text.push_str(note);
        }
        out.extend(p.wrap(&text, w, |l| l.to_string()));
    }

    out.push(String::new());
    out.push(p.heading("STAT CARD", accent));
    for (label, value) in c.stats.labelled() {
        let (full, empty) = if p.plain { ("#", ".") } else { ("█", "░") };
        let v = usize::from(value.min(10));
        out.push(format!(
            "  {:<9} {}{} {:>2}",
            label,
            p.paint(&full.repeat(v), primary),
            p.paint(&empty.repeat(10 - v), DIM),
            value
        ));
    }
    out.push(String::new());
    out.extend(
        p.wrap(&format!("Powers: {}", c.powers.join(" · ")), w, |l| {
            l.replacen("Powers:", &p.bold("Powers:", GREEN), 1)
        }),
    );
    out.extend(p.wrap(
        &format!("Weaknesses: {}", c.weaknesses.join(" · ")),
        w,
        |l| l.replacen("Weaknesses:", &p.bold("Weaknesses:", RED), 1),
    ));

    if !c.legacy.is_empty() {
        out.push(String::new());
        out.push(p.heading("LEGACY", accent));
        for item in &c.legacy {
            out.extend(hanging(p, b, item, w, |l| p.paint(l, DIM)));
        }
    }
    out.join("\n") + "\n"
}

/// Wraps text under a marker with a hanging indent: " • first line\n   more".
fn hanging(
    p: &Painter,
    marker: &str,
    text: &str,
    width: usize,
    paint: impl Fn(&str) -> String,
) -> Vec<String> {
    let lead = if marker.is_empty() {
        "    ".to_string()
    } else {
        format!(" {marker} ")
    };
    let indent = lead.width();
    let opts = textwrap::Options::new(width.saturating_sub(indent).max(10));
    textwrap::wrap(text, opts)
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let prefix = if i == 0 {
                lead.clone()
            } else {
                " ".repeat(indent)
            };
            let marker_painted = if i == 0 && !marker.is_empty() {
                prefix.replacen(marker, &p.paint(marker, GOLD), 1)
            } else {
                prefix
            };
            format!("{marker_painted}{}", paint(line))
        })
        .collect()
}

// ── Cards ──────────────────────────────────────────────────────────────────

/// A boxed fact, introduced by a title and signed off by the host.
pub fn fact_card(
    p: &Painter,
    title: &str,
    c: &Creature,
    fact: &str,
    notes: &[Note],
    quip: &str,
) -> String {
    let outer = p.width.saturating_sub(2).min(66);
    let inner = outer.saturating_sub(4).max(20);
    let mut lines = vec![
        p.bold(&format!("{}{title}", p.glyph("🎄")), GOLD),
        String::new(),
    ];
    lines.push(p.bold(
        &format!("{}{}", p.glyph(&c.emoji), c.name),
        &c.theme.primary,
    ));
    lines.extend(p.wrap(fact, inner, |l| p.paint(l, SNOW)));
    for n in notes {
        lines.push(String::new());
        lines.extend(p.wrap(&p.note(n), inner, |l| p.paint(l, DIM)));
    }
    if !quip.is_empty() {
        lines.push(String::new());
        lines.extend(p.wrap(
            &format!("{}{}: “{quip}”", p.glyph("📺"), host::NAME),
            inner,
            |l| p.italic(l, GOLD),
        ));
    }
    p.boxed(&lines, inner, &c.theme.accent)
}

// ── The countdown ──────────────────────────────────────────────────────────

const TREE: &[&str] = &[
    "     *     ",
    "    /o\\    ",
    "   /.*.\\   ",
    "  /o.*.o\\  ",
    " /.*.o.*.\\ ",
    "/o.*.o.*.o\\",
    "    |_|    ",
];

/// The tree, with a gold star, red baubles and a brown trunk.
fn tree(p: &Painter) -> Vec<String> {
    TREE.iter()
        .enumerate()
        .map(|(row, line)| {
            if p.plain {
                return line.to_string();
            }
            if row == TREE.len() - 1 {
                return p.paint(line, "#9C6644");
            }
            line.chars()
                .map(|ch| match ch {
                    '*' => p.bold("*", GOLD),
                    'o' => p.bold("o", RED),
                    ' ' => " ".to_string(),
                    c => p.paint(&c.to_string(), GREEN),
                })
                .collect()
        })
        .collect()
}

const DIGITS: [[&str; 5]; 10] = [
    ["###", "#.#", "#.#", "#.#", "###"],
    [".#.", "##.", ".#.", ".#.", "###"],
    ["###", "..#", "###", "#..", "###"],
    ["###", "..#", "###", "..#", "###"],
    ["#.#", "#.#", "###", "..#", "..#"],
    ["###", "#..", "###", "..#", "###"],
    ["###", "#..", "###", "#.#", "###"],
    ["###", "..#", "..#", "..#", "..#"],
    ["###", "#.#", "###", "#.#", "###"],
    ["###", "#.#", "###", "..#", "###"],
];

/// A number in big block digits, five rows tall.
pub fn big_number(p: &Painter, n: i64) -> Vec<String> {
    let block = if p.plain { "##" } else { "██" };
    let digits: Vec<usize> = n
        .to_string()
        .chars()
        .filter_map(|c| c.to_digit(10).map(|d| d as usize))
        .collect();
    (0..5)
        .map(|row| {
            digits
                .iter()
                .map(|&d| {
                    DIGITS[d][row]
                        .chars()
                        .map(|c| if c == '#' { block } else { "  " })
                        .collect::<String>()
                })
                .collect::<Vec<_>>()
                .join("  ")
        })
        .collect()
}

/// Sleeps until Christmas, tonight's advent door and the season's notes.
pub fn countdown(p: &Painter, vault: &Vault, today: Date) -> String {
    let w = p.text_width();
    let sleeps = calendar::sleeps_until_christmas(today);
    let mut out = Vec::new();
    let tree = tree(p);
    let right: Vec<String> = if sleeps == 0 {
        vec![
            String::new(),
            String::new(),
            p.bold(&format!("{}MERRY CHRISTMAS!", p.glyph("🎁")), RED),
        ]
    } else {
        let mut r: Vec<String> = big_number(p, sleeps)
            .iter()
            .map(|l| p.bold(l, RED))
            .collect();
        r.push(String::new());
        let caption = if sleeps == 1 {
            "SLEEP UNTIL CHRISTMAS"
        } else {
            "SLEEPS UNTIL CHRISTMAS"
        };
        r.push(p.bold(caption, SNOW));
        r
    };
    for i in 0..tree.len().max(right.len()) {
        let left = tree
            .get(i)
            .cloned()
            .unwrap_or_else(|| " ".repeat(TREE[0].len()));
        out.push(
            format!("{left}    {}", right.get(i).cloned().unwrap_or_default())
                .trim_end()
                .to_string(),
        );
    }
    out.push(String::new());

    if let Some((night, c, fact)) = calendar::advent_guest(today, vault) {
        out.push(p.heading(&format!("ADVENT · DOOR {}", night.night), GREEN));
        out.push(p.bold(
            &format!("Tonight's guest: {}{}", p.glyph(&c.emoji), c.name),
            &c.theme.primary,
        ));
        out.extend(p.wrap(&night.note, w, |l| p.italic(l, GOLD)));
        out.extend(p.wrap(&fact, w, |l| p.paint(l, SNOW)));
        out.push(String::new());
    } else if sleeps == 0 {
        out.extend(p.wrap(
            &format!(
                "{}{}: “It's the big day! Presents open, porridge out for the tomte, and a new jumper on, for the Yule Cat's sake. Merry Christmas from all of us at {}.”",
                p.glyph("📺"),
                host::NAME,
                host::CHANNEL
            ),
            w,
            |l| p.italic(l, GOLD),
        ));
        out.push(String::new());
    } else if calendar::twelve_days(today).is_none() {
        out.extend(p.wrap(
            &format!(
                "{}{}: “The advent calendar opens on 1 December, with a different guest behind every door. Don't miss it!”",
                p.glyph("📺"),
                host::NAME
            ),
            w,
            |l| p.italic(l, GOLD),
        ));
        out.push(String::new());
    }
    for n in calendar::notes(today) {
        out.extend(p.wrap(&p.note(&n), w, |l| p.paint(l, DIM)));
    }
    while out.last().is_some_and(|l| l.is_empty()) {
        out.pop();
    }
    out.join("\n") + "\n"
}

// ── The advent calendar ────────────────────────────────────────────────────

/// Whether a door is shut, open, or tonight's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Door {
    Shut,
    Open,
    Tonight,
}

/// A door's state on a date: doors open as December goes on, and stay open
/// until the new year.
pub fn door(night: u8, today: Date) -> Door {
    if today.month() != 12 {
        return Door::Shut;
    }
    let day = i16::from(today.day());
    match i16::from(night).cmp(&day) {
        std::cmp::Ordering::Less => Door::Open,
        std::cmp::Ordering::Equal => Door::Tonight,
        std::cmp::Ordering::Greater => Door::Shut,
    }
}

/// The 24 doors as a grid, six to a row, then tonight's door in full.
pub fn advent(p: &Painter, vault: &Vault, today: Date) -> String {
    let w = p.text_width();
    let mut out = vec![
        p.bold(
            &format!("{}THE CHANNEL 25 ADVENT CALENDAR", p.glyph("📅")),
            GOLD,
        ),
        String::new(),
    ];
    for row in vault.advent.chunks(6) {
        let cells: Vec<String> = row
            .iter()
            .map(|n| {
                let c = vault.get(&n.id);
                let state = door(n.night, today);
                let inside = match (state, c, p.plain) {
                    (Door::Shut, _, true) => "  ".to_string(),
                    (Door::Shut, _, false) => "··".to_string(),
                    (_, Some(c), false) => c.emoji.clone(),
                    (Door::Tonight, _, true) => "<>".to_string(),
                    (_, _, _) => "++".to_string(),
                };
                let cell = format!("[{:>2} {inside}]", n.night);
                match (state, c) {
                    (Door::Tonight, _) => p.bold(&cell, GOLD),
                    (Door::Open, Some(c)) => p.paint(&cell, &c.theme.primary),
                    _ => p.paint(&cell, DIM),
                }
            })
            .collect();
        out.push(format!("  {}", cells.join(" ")));
    }
    out.push(String::new());
    match calendar::advent_guest(today, vault) {
        Some((night, c, fact)) => {
            out.push(p.bold(
                &format!("Door {}: {}{}", night.night, p.glyph(&c.emoji), c.name),
                &c.theme.primary,
            ));
            out.extend(p.wrap(&night.note, w, |l| p.italic(l, GOLD)));
            out.extend(p.wrap(&fact, w, |l| p.paint(l, SNOW)));
        }
        None if today.month() == 12 => {
            out.extend(p.wrap(
                "Every door is open. Merry Christmas from all of us at Channel 25!",
                w,
                |l| p.italic(l, GOLD),
            ));
        }
        None => {
            let to_go = (jiff::civil::date(today.year(), 12, 1) - today).get_days();
            out.extend(p.wrap(
                &format!("The first door opens on 1 December, in {to_go} sleeps."),
                w,
                |l| p.italic(l, GOLD),
            ));
        }
    }
    out.join("\n") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::date;

    fn plain() -> Painter {
        Painter::new(true, 80)
    }

    fn colour() -> Painter {
        Painter::new(false, 80)
    }

    #[test]
    fn hex_colours_parse() {
        assert_eq!(parse_hex("#E63946"), Some((0xE6, 0x39, 0x46)));
        assert_eq!(parse_hex("E63946"), None);
        assert_eq!(parse_hex("#E639"), None);
    }

    #[test]
    fn visible_width_ignores_colour_codes() {
        let p = colour();
        assert_eq!(visible_width(&p.bold("Krampus", RED)), 7);
        assert_eq!(visible_width("😈 x"), 4);
    }

    #[test]
    fn plain_mode_has_no_escape_codes_or_emoji() {
        let p = plain();
        let v = Vault::load();
        for c in &v.creatures {
            let page = creature(&p, c, "pack", true);
            assert!(!page.contains('\u{1b}'), "{}: colour in plain mode", c.id);
            assert!(!page.contains(&c.emoji), "{}: emoji in plain mode", c.id);
        }
        let all = [
            list(&p, &v),
            countdown(&p, &v, date(2026, 12, 5)),
            advent(&p, &v, date(2026, 12, 5)),
        ];
        for text in all {
            assert!(!text.contains('\u{1b}'));
            assert!(!text.contains('😈'));
        }
    }

    #[test]
    fn pages_fit_the_width() {
        for width in [40, 60, 80] {
            let p = Painter::new(false, width);
            let v = Vault::load();
            for c in &v.creatures {
                for line in creature(&p, c, "A Christmas Carol", true).lines() {
                    assert!(visible_width(line) <= p.text_width(), "{}: {line:?}", c.id);
                }
            }
        }
    }

    #[test]
    fn fact_card_lines_line_up() {
        let p = colour();
        let v = Vault::load();
        let c = v.get("yule-cat").unwrap();
        let card = fact_card(
            &p,
            "FACT OF THE NIGHT",
            c,
            &c.facts[0],
            &calendar::notes(date(2026, 12, 24)),
            "Festive!",
        );
        let widths: Vec<usize> = card.lines().map(visible_width).collect();
        assert!(
            widths.windows(2).all(|w| w[0] == w[1]),
            "{widths:?}\n{card}"
        );
    }

    #[test]
    fn big_numbers_are_five_rows() {
        let rows = big_number(&plain(), 43);
        assert_eq!(rows.len(), 5);
        assert_eq!(rows[0], "##  ##  ######");
    }

    #[test]
    fn countdown_says_merry_christmas_on_the_day() {
        let v = Vault::load();
        assert!(countdown(&plain(), &v, date(2026, 12, 25)).contains("MERRY CHRISTMAS"));
        assert!(countdown(&plain(), &v, date(2026, 12, 24)).contains("SLEEP UNTIL CHRISTMAS"));
        assert!(countdown(&plain(), &v, date(2026, 12, 5)).contains("Krampusnacht"));
    }

    #[test]
    fn doors_open_through_december() {
        assert_eq!(door(1, date(2026, 11, 30)), Door::Shut);
        assert_eq!(door(1, date(2026, 12, 2)), Door::Open);
        assert_eq!(door(2, date(2026, 12, 2)), Door::Tonight);
        assert_eq!(door(3, date(2026, 12, 2)), Door::Shut);
        assert_eq!(door(24, date(2026, 12, 28)), Door::Open);
        assert_eq!(door(24, date(2027, 1, 2)), Door::Shut);
    }

    #[test]
    fn myths_hide_their_verdicts_until_revealed() {
        let v = Vault::load();
        let c = v.get("krampus").unwrap();
        assert!(!creature(&plain(), c, "x", false).contains("FALSE:"));
        assert!(creature(&plain(), c, "x", true).contains("FALSE:"));
    }

    #[test]
    fn plain_notes_lose_their_emoji() {
        let n = &calendar::notes(date(2026, 12, 13))[0];
        assert_eq!(plain().note(n), "2 Yule Lads are in town.");
        assert_eq!(colour().note(n), "🥄 2 Yule Lads are in town.");
    }
}
