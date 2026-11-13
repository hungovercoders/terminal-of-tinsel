//! Plain mode for the full-screen pages. `--no-color` and `NO_COLOR` promise
//! no colour, no emoji and ASCII borders. Colour is switched off where the
//! styles are made and emoji are left out where the text is built; this
//! swaps the box-drawing and block characters once the screen is drawn.

use ratatui::buffer::Buffer;

/// The ASCII stand-in for a drawing character, if it is one.
pub fn ascii(symbol: &str) -> Option<&'static str> {
    Some(match symbol {
        "╭" | "╮" | "╰" | "╯" | "╔" | "╗" | "╚" | "╝" => "+",
        "─" => "-",
        "═" => "=",
        "│" | "║" => "|",
        "█" | "▀" | "▄" | "▓" => "#",
        "▒" => "%",
        "░" => ":",
        "▌" => "_",
        "▶" => ">",
        "▲" => "^",
        "▼" => "v",
        "•" => "-",
        "✔" => "*",
        "✘" => "x",
        _ => return None,
    })
}

/// Swaps every drawing character on the screen for its ASCII stand-in.
pub fn plainify(buf: &mut Buffer) {
    for cell in &mut buf.content {
        if let Some(plain) = ascii(cell.symbol()) {
            cell.set_symbol(plain);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    #[test]
    fn drawing_characters_become_ascii_and_text_is_left_alone() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 12, 1));
        buf.set_string(0, 0, "╭─╮│█░▶ Grý", ratatui::style::Style::new());
        plainify(&mut buf);
        let text: String = buf.content.iter().map(|c| c.symbol()).collect();
        assert_eq!(text.trim_end(), "+-+|#:> Grý");
        assert!(ascii("a").is_none() && ascii("é").is_none());
        for stand_in in ["╭", "═", "█", "▒", "✔", "✘", "•", "▌"] {
            assert!(ascii(stand_in).unwrap().is_ascii());
        }
    }
}
