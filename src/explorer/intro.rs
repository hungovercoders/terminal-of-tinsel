//! Channel 25 tunes in: static that settles into falling snow, the big
//! title, and the host's greeting typed out.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthStr;

use super::model::Model;
use super::view;
use crate::host;
use crate::render::{DIM, GOLD, GREEN, RED};
use crate::tui::rgb;

/// The intro's timeline, in ticks.
const TUNE_FRAMES: usize = 18; // static while the set warms up
const FLASH_FRAMES: usize = 2; // the star on the tree lights up
const TYPE_SPEED: usize = 3; // characters of the greeting typed per tick

/// The frame at which the greeting has been typed in full.
pub fn done_frame(greeting: &str) -> usize {
    TUNE_FRAMES + FLASH_FRAMES + greeting.chars().count().div_ceil(TYPE_SPEED)
}

/// A tiny two-row block font, just big enough for the title.
fn glyph(ch: char) -> [&'static str; 2] {
    match ch {
        'T' => ["▀█▀", " █ "],
        'E' => ["█▀▀", "██▄"],
        'R' => ["█▀█", "█▀▄"],
        'M' => ["█▀▄▀█", "█ ▀ █"],
        'I' => ["█", "█"],
        'N' => ["█▄ █", "█ ▀█"],
        'A' => ["▄▀█", "█▀█"],
        'L' => ["█  ", "█▄▄"],
        'O' => ["█▀█", "█▄█"],
        'F' => ["█▀▀", "█▀ "],
        'S' => ["█▀", "▄█"],
        _ => [" ", " "],
    }
}

/// Words in the block font: two rows.
fn big_text(words: &[&str]) -> [String; 2] {
    let row = |r: usize| {
        words
            .iter()
            .map(|w| w.chars().map(|c| glyph(c)[r]).collect::<Vec<_>>().join(" "))
            .collect::<Vec<_>>()
            .join("   ")
    };
    [row(0), row(1)]
}

/// The big title, stacked onto two lines on narrow terminals and spelled
/// out plainly on very narrow ones.
pub fn logo(width: usize) -> Vec<String> {
    let full = big_text(&["TERMINAL", "OF", "TINSEL"]);
    if width >= full[0].width() + 4 {
        return full.into();
    }
    let top = big_text(&["TERMINAL"]);
    if width >= top[0].width() + 4 {
        let [a, b] = top;
        let [c, d] = big_text(&["OF", "TINSEL"]);
        return vec![a, b, String::new(), c, d];
    }
    vec!["TERMINAL OF TINSEL".to_string()]
}

/// A well-mixed number from a seed and a position, so the snow needs no
/// state: the same frame always draws the same flakes, at any size.
fn mix(seed: u64, a: u64, b: u64) -> u64 {
    let mut z = seed
        .wrapping_add(a.wrapping_mul(0x9E37_79B9_7F4A_7C15))
        .wrapping_add(b.wrapping_mul(0xC2B2_AE3D_27D4_EB4F));
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// The snowflake at a cell on this frame, if one is falling through it.
/// About one column in three has a flake, each falling at its own speed.
pub fn flake(seed: u64, x: usize, y: usize, height: usize, frame: usize) -> Option<char> {
    let h = mix(seed, x as u64, 0);
    if !h.is_multiple_of(3) || height == 0 {
        return None;
    }
    let slowness = 1 + (h >> 8) as usize % 3;
    let lap = height + (h >> 16) as usize % (height + 1);
    let at = (frame / slowness + (h >> 32) as usize) % lap;
    (at == y).then(|| ['*', '.', '+', '.'][(h >> 4) as usize % 4])
}

/// TV static, thinning out as the set warms up.
fn static_cell(seed: u64, x: usize, y: usize, frame: usize) -> char {
    let h = mix(seed, (x as u64) << 20 | y as u64, frame as u64 + 1);
    let thinning = 85usize.saturating_sub(frame * 70 / TUNE_FRAMES);
    if (h % 100) as usize >= thinning {
        return ' ';
    }
    let chars = if frame < TUNE_FRAMES / 2 {
        ['.', ':', '░', '▒', '▓']
    } else {
        ['.', '.', '*', ':', '░']
    };
    chars[(h >> 8) as usize % chars.len()]
}

/// Draws the intro for the model's current frame.
pub fn draw(m: &Model, buf: &mut Buffer, area: Rect) {
    let (w, h) = (usize::from(area.width), usize::from(area.height));
    let dim = view::fg(m, rgb(DIM));
    let gold = view::fg(m, rgb(GOLD)).add_modifier(Modifier::BOLD);

    if m.frame < TUNE_FRAMES {
        for y in 0..h {
            let row: String = (0..w)
                .map(|x| static_cell(m.snow_seed, x, y, m.frame))
                .collect();
            buf.set_string(area.x, area.y + y as u16, row, dim);
        }
        if m.frame > TUNE_FRAMES / 2 {
            let label = Line::styled(
                format!(
                    " {}{} · MIDWINTER SPECIAL ",
                    view::icon(m, "📺"),
                    host::CHANNEL
                ),
                view::title_style(m),
            );
            centred(buf, area, h / 2, &label);
        }
        return;
    }

    // Snow keeps falling behind the title for as long as the intro is up.
    for y in 0..h {
        for x in 0..w {
            if let Some(ch) = flake(m.snow_seed, x, y, h, m.frame) {
                buf.set_string(area.x + x as u16, area.y + y as u16, ch.to_string(), dim);
            }
        }
    }

    let flashing = m.frame < TUNE_FRAMES + FLASH_FRAMES;
    let logo_style = if flashing {
        m.ink.badge(rgb(GOLD))
    } else {
        view::fg(m, rgb(RED)).add_modifier(Modifier::BOLD)
    };

    let mut lines = vec![
        Line::styled(format!("*  {} presents  *", host::CHANNEL), dim),
        Line::default(),
    ];
    // The block font is all drawing characters, so plain mode spells it out.
    let logo_width = if m.ink.colour { w } else { 0 };
    lines.extend(
        logo(logo_width)
            .into_iter()
            .map(|l| Line::styled(l, logo_style)),
    );
    lines.push(Line::default());
    lines.push(Line::styled("~ The Midwinter Special ~", gold));
    lines.push(Line::default());

    // The greeting is wrapped whole and then revealed, so words never jump
    // from one line to the next as they are typed.
    let full = format!("{}{}: {}", view::icon(m, "📺"), host::NAME, m.greeting);
    let typed = m.frame.saturating_sub(TUNE_FRAMES + FLASH_FRAMES) * TYPE_SPEED;
    let mut budget = if m.intro_done() {
        usize::MAX
    } else {
        // The host's name is already on screen when the typing starts.
        typed + full.chars().count() - m.greeting.chars().count()
    };
    let host_style = view::fg(m, rgb(GOLD)).add_modifier(Modifier::ITALIC);
    let wrap_at = w.saturating_sub(4).clamp(10, 70);
    let wrapped = textwrap::wrap(&full, wrap_at);
    let block_width = wrapped.iter().map(|l| l.width()).max().unwrap_or(0);
    let greeting_at = lines.len();
    let mut typing = true;
    for text in &wrapped {
        let n = text.chars().count();
        let mut shown: String = text.chars().take(budget).collect();
        if typing && budget < n {
            // The cursor sits where the typing has got to, and nowhere else.
            shown.push('▌');
            typing = false;
        }
        budget = budget.saturating_sub(n + 1);
        lines.push(Line::styled(shown, host_style));
    }

    if m.intro_done() {
        if !m.notes.is_empty() {
            lines.push(Line::default());
            let note_style = view::fg(m, rgb(GREEN)).add_modifier(Modifier::ITALIC);
            for note in m.notes.iter().take(3) {
                lines.push(Line::styled(note.clone(), note_style));
            }
        }
        lines.push(Line::default());
        lines.push(Line::from(vec![
            Span::styled("Press any key to meet tonight's guests...", gold),
            Span::styled("  (q to quit)", dim),
        ]));
    }

    let top = h.saturating_sub(lines.len() + 4) / 2;
    for (i, line) in lines.iter().enumerate() {
        let y = top + i;
        if y >= h {
            break;
        }
        if (greeting_at..greeting_at + wrapped.len()).contains(&i) {
            // The greeting is a left-aligned block, centred as a whole.
            let x = w.saturating_sub(block_width) / 2;
            buf.set_line(area.x + x as u16, area.y + y as u16, line, area.width);
        } else {
            centred(buf, area, y, line);
        }
    }
}

/// Draws a line centred on row `y` of the area.
fn centred(buf: &mut Buffer, area: Rect, y: usize, line: &Line) {
    if y >= usize::from(area.height) {
        return;
    }
    let x = usize::from(area.width).saturating_sub(line.width()) / 2;
    buf.set_line(area.x + x as u16, area.y + y as u16, line, area.width);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_logo_fits_the_width_it_is_given() {
        for width in [20, 40, 60, 80, 120] {
            for line in logo(width) {
                assert!(line.width() <= width, "{width}: {line}");
            }
        }
        assert_eq!(logo(120).len(), 2);
        assert_eq!(logo(50).len(), 5);
        assert_eq!(logo(20), ["TERMINAL OF TINSEL"]);
    }

    #[test]
    fn the_same_frame_always_has_the_same_snow() {
        let snow = |frame| {
            (0..40)
                .flat_map(|x| (0..12).map(move |y| flake(25, x, y, 12, frame)))
                .collect::<Vec<_>>()
        };
        assert_eq!(snow(7), snow(7));
        assert_ne!(snow(7), snow(8));
        assert!(snow(7).iter().any(Option::is_some));
        assert_eq!(flake(25, 3, 0, 0, 1), None);
    }

    #[test]
    fn the_intro_ends_when_the_greeting_is_typed() {
        assert_eq!(done_frame(""), 20);
        assert_eq!(done_frame("abcd"), 22);
    }
}
