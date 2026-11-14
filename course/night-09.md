# Night 9 · Deck the Terminal

> 📺 *"Good evening. The decorations are down from the loft, and so far the programme has been broadcasting in what we in the trade call 'radio'. Tonight: colour. Green, gold and a tasteful box or two. And for the viewers whose sets can't show it, we shall be every bit as clear in black and white."*

**Tonight you'll learn**
- How colour works in a terminal, and `owo-colors` for true colour from a hex code
- Why text has a *width* that isn't its length: `unicode-width`
- Wrapping with `textwrap`, and asking the terminal how wide it is
- Drawing a box whose sides line up
- Plain mode: `--no-color`, `NO_COLOR` and pipes, all through one `Painter`
- Passing a closure to a function

**Where we are:** three commands that print with `println!` straight from `main.rs`, in whatever colour the terminal happens to be.

## How a terminal does colour

A terminal shows the characters it's sent. Some sequences of characters are instructions instead: the escape character followed by `[38;2;233;196;106m` means "from here on, draw text in the colour with red 233, green 196, blue 106", and escape `[0m` means "back to normal". That's **true colour**, any of 16 million shades, and every modern terminal understands it.

Two things follow. A coloured string is longer than it looks, so text must be measured and wrapped *before* it's coloured. And if the output isn't going to a terminal (it's piped into a file or another program) those sequences are litter, so we leave them out.

## Four crates

```bash
cargo add owo-colors@4.4.0 unicode-width@0.2.3 terminal_size@0.4.4
cargo add textwrap@0.16.4 --no-default-features --features unicode-width,smawk
```

- **owo-colors** writes the escape sequences.
- **unicode-width** says how many columns a string occupies. An emoji is one character and two columns wide.
- **terminal_size** asks the terminal how wide it is.
- **textwrap** breaks text into lines. `--no-default-features` switches its optional parts off, and we switch two back on: measuring by columns, and the algorithm that balances line lengths.

## The Painter

Create `src/render.rs`. It's the biggest file so far, so we'll take it in four parts. First the `Painter`:

```rust
//! Drawing for the terminal: colours, boxes and wrapped text.
//!
//! Everything here returns a `String`, so it can be tested without a terminal.
//! In plain mode (`--no-color`, `NO_COLOR` or a pipe) there's no colour, no
//! emoji and only ASCII borders.

use owo_colors::{OwoColorize, Style};
use unicode_width::UnicodeWidthStr;

use crate::creatures::{Creature, Vault};
use crate::host;

/// The house colours of Channel 25.
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
            return text.to_string();
        }
        let mut style = Style::new();
        if let Some((r, g, b)) = parse_hex(hex) {
            style = style.truecolor(r, g, b);
        }
        if bold {
            style = style.bold();
        }
        if italic {
            style = style.italic();
        }
        text.style(style).to_string()
    }

    /// An emoji and a space, or nothing in plain mode.
    pub fn glyph(&self, emoji: &str) -> String {
        if self.plain {
            String::new()
        } else {
            format!("{emoji} ")
        }
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
}
```

A `Painter` is two facts: are we plain, and how wide is the page. Everything that draws takes one, and **no other code asks whether colour is on**. It calls `p.paint(text, GOLD)` and gets back coloured text or the same text.

- **`styled`** is where colour happens. In plain mode it returns the text untouched. Otherwise it builds a `Style` and applies it with `.style(...)`, a method that owo-colors adds to anything printable through its `OwoColorize` trait. That's why the trait is imported though its name is never used: a trait's methods are only available where the trait is in scope.
- **`glyph`** gives an emoji and a space, or nothing in plain mode.
- **`wrap`** takes a third argument, `paint: impl Fn(&str) -> String`: any function or closure from a line to a painted line. It wraps first and paints each line after, which is the rule from above. Callers pass a closure such as `|l| p.italic(l, GOLD)`.
- **`heading`** rules a title across the page. `title.width()` is from unicode-width, and `saturating_sub` is subtraction that stops at zero instead of overflowing, for the day a title is wider than the page.
- **`boxed`** draws a frame. Each line is padded on the right to the box's inner width, measured with `visible_width` so that colour codes don't count. In plain mode the corners are `+` and the sides `-` and `|`.

`clamp(40, 100)` in `new` keeps the width sensible.

## Measuring

Add the three helpers below the `impl`:

```rust
/// Parses `#RRGGBB`.
pub fn parse_hex(hex: &str) -> Option<(u8, u8, u8)> {
    let h = hex.strip_prefix('#')?;
    // Six bytes that aren't six ASCII characters ("#aé123") would be sliced
    // through the middle of a character below.
    if h.len() != 6 || !h.is_ascii() {
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
pub fn truncate(text: &str, max: usize) -> String {
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
```

**`parse_hex`** turns `#E9C46A` into three numbers. It returns an `Option`, and uses `?` on options: inside a function that returns `Option`, `?` means "if this is `None`, return `None`". `&h[i..i + 2]` slices two bytes out of the string. Slicing a string through the middle of a multi-byte character would panic, which is what the `is_ascii` check is there to prevent.

**`visible_width`** copies a string while skipping escape sequences, then measures what's left. `while let Some(c) = chars.next()` loops for as long as the pattern fits. **`truncate`** cuts text to a number of columns and adds an ellipsis.

## The pages

Each page is a function from data to a `String`. Nothing in this file prints, which is what lets us test it:

```rust
/// What you see with no command: the host and where to start.
pub fn welcome(p: &Painter, greeting: &str) -> String {
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
        p.text_width(),
        |l| p.italic(l, GOLD),
    ));
    out.push(String::new());
    out.push(p.heading("START HERE", GREEN));
    for (cmd, what) in [
        ("list", "meet tonight's guests"),
        ("creature krampus", "read a creature's page"),
        ("random", "a fact from the cracker"),
    ] {
        out.push(format!(
            "  {}  {}",
            p.bold(&format!("terminal-of-tinsel {cmd:<17}"), SNOW),
            p.paint(what, DIM)
        ));
    }
    out.join("\n") + "\n"
}

/// Every creature, grouped by pack.
pub fn list(p: &Painter, vault: &Vault) -> String {
    let w = p.text_width();
    let title = format!(
        "{}TONIGHT'S GUESTS · {} creatures",
        p.glyph("🎄"),
        vault.creatures.len()
    );
    let mut out = vec![p.bold(&title, GOLD)];
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
                p.bold(&name, SNOW),
                p.paint(&truncate(&c.description, room), DIM)
            ));
        }
    }
    out.push(String::new());
    out.push(p.paint("Meet one: terminal-of-tinsel creature <name>", DIM));
    out.join("\n") + "\n"
}

/// A boxed fact, introduced by a title and signed off by the host.
pub fn fact_card(p: &Painter, title: &str, c: &Creature, fact: &str, quip: &str) -> String {
    let outer = p.width.saturating_sub(2).min(66);
    let inner = outer.saturating_sub(4).max(20);
    let mut lines = vec![
        p.bold(&format!("{}{title}", p.glyph("🎄")), GOLD),
        String::new(),
        p.bold(&format!("{}{}", p.glyph(&c.emoji), c.name), SNOW),
    ];
    lines.extend(p.wrap(fact, inner, |l| p.paint(l, SNOW)));
    lines.push(String::new());
    lines.extend(p.wrap(
        &format!("{}{}: “{quip}”", p.glyph("📺"), host::NAME),
        inner,
        |l| p.italic(l, GOLD),
    ));
    p.boxed(&lines, inner, GREEN)
}
```

They all build a `Vec<String>` of lines, then `join("\n")`. In `list`, `{:<name_w$}` pads to a width held in a variable.

And the tests, at the end of the file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_colours_parse() {
        assert_eq!(parse_hex("#E63946"), Some((0xE6, 0x39, 0x46)));
        assert_eq!(parse_hex("E63946"), None);
        assert_eq!(parse_hex("#aé123"), None);
    }

    #[test]
    fn visible_width_ignores_colour_codes() {
        let p = Painter::new(false, 80);
        assert_eq!(visible_width(&p.bold("Krampus", GOLD)), 7);
        assert_eq!(visible_width("🐈 cat"), 6);
    }

    #[test]
    fn plain_mode_has_no_escape_codes_or_emoji() {
        let p = Painter::new(true, 80);
        let page = list(&p, &Vault::load());
        assert!(!page.contains('\u{1b}'));
        assert!(!page.contains('🎄'));
        assert!(page.lines().all(|l| l.width() <= 80));
    }
}
```

## The host, the flag and main

The fact card ends with a quip. In `src/host.rs`, add the lines under `GREETINGS` and the function under `greeting`:

```rust
const QUIPS: &[&str] = &[
    "Put that in the cracker jokes. It's better than the usual ones.",
    "Now you know. Tell them over the sprouts.",
    "Festive, isn't it? In a worrying sort of way.",
    "Something to think about while you're putting the star on top.",
    "And they say Christmas telly isn't educational.",
];
```

```rust
/// A short reaction to a fact.
pub fn quip(rng: &mut Rng) -> &'static str {
    QUIPS[rng.below(QUIPS.len())]
}
```

In `src/cli.rs`, add a flag as the first field of `Cli`:

```rust
    /// Plain output: no colour, no emoji, ASCII borders (also set by NO_COLOR, or by piping)
    #[arg(long, global = true)]
    pub no_color: bool,
```

`global = true` lets the flag go before or after the subcommand.

In `src/main.rs`, the top of the file becomes:

```rust
mod cli;
mod creatures;
mod error;
mod host;
mod render;
mod rng;

use std::io::{self, IsTerminal};
use std::process::ExitCode;

use clap::Parser;

use cli::{Cli, Command};
use creatures::{Creature, Vault};
use error::{Code, Result, TinselError};
use render::Painter;
use rng::Rng;
```

`main` itself and `random_fact` stay as they are. Delete `run`, `welcome`, `list`, `random` and `creature`, and put this after `main`:

```rust
/// What every command needs: how to draw and which creatures are in play.
struct Show {
    painter: Painter,
    vault: Vault,
}

fn run(cli: Cli) -> Result<()> {
    let no_colour = cli.no_color || std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty());
    let plain = no_colour || !io::stdout().is_terminal();
    let show = Show {
        painter: Painter::new(plain, terminal_width()),
        vault: Vault::load(),
    };
    show.dispatch(cli.command)
}

impl Show {
    /// Runs the command. One flat match, so that a new `Command` variant
    /// fails to compile until it has an arm here.
    fn dispatch(&self, command: Option<Command>) -> Result<()> {
        match command {
            None => self.welcome(),
            Some(Command::List) => self.list(),
            Some(Command::Creature { name }) => self.creature(&name.join(" ")),
            Some(Command::Random) => self.random(),
        }
    }

    fn welcome(&self) -> Result<()> {
        let mut rng = Rng::from_env_or_clock();
        print!(
            "{}",
            render::welcome(&self.painter, host::greeting(&mut rng))
        );
        Ok(())
    }

    fn list(&self) -> Result<()> {
        print!("{}", render::list(&self.painter, &self.vault));
        Ok(())
    }

    fn creature(&self, name: &str) -> Result<()> {
        let c = self.vault.find(name)?;
        println!("{} {}", c.emoji, c.name.to_uppercase());
        println!("{}", c.description);
        println!("{} · pack: {}", c.origin, c.pack);
        println!();
        println!("📺 {}: “{}”", host::NAME, host::intro(c));
        println!();
        println!("── FACTS ──");
        for fact in &c.facts {
            println!(" • {fact}");
        }
        println!();
        println!("── THE LEGEND ──");
        println!("{}", c.legend);
        Ok(())
    }

    fn random(&self) -> Result<()> {
        let mut rng = Rng::from_env_or_clock();
        let (who, which) = random_fact(&self.vault.creatures, &mut rng)
            .ok_or_else(|| TinselError::new(Code::Usage, "no facts to tell tonight"))?;
        let c = &self.vault.creatures[who];
        let quip = host::quip(&mut rng);
        print!(
            "{}",
            render::fact_card(
                &self.painter,
                "RANDOM FESTIVE FACT",
                c,
                &c.facts[which],
                quip
            )
        );
        Ok(())
    }
}
```

And add one function at the end of the file:

```rust
fn terminal_width() -> usize {
    terminal_size::terminal_size()
        .map(|(w, _)| usize::from(w.0))
        .or_else(|| std::env::var("COLUMNS").ok()?.parse().ok())
        .unwrap_or(80)
}
```

`Show` gathers what every command needs, and the commands become its methods, as in the finished program. `run` decides on plain mode, once. Colour is off if any of three things is true:

- the player passed `--no-color`
- the environment variable `NO_COLOR` is set to something, a convention many programs honour (see [no-color.org](https://no-color.org))
- stdout is not a terminal. `is_terminal()` comes from the standard library's `IsTerminal` trait.

`terminal_width` asks the terminal, falls back to the `COLUMNS` variable, then to 80. The `creature` page still prints directly; tomorrow it gets the full treatment.

## Run it

```bash
TERMINAL_OF_TINSEL_SEED=25 cargo run -q -- random
```

In your terminal that's a green rounded box with a gold title. Here it is piped through `cat`, which makes it plain:

```bash
TERMINAL_OF_TINSEL_SEED=25 cargo run -q -- random | cat
```

```
+----------------------------------------------------------------+
| RANDOM FESTIVE FACT                                            |
|                                                                |
| The Ghost of Christmas Yet to Come                             |
| It never speaks, answering only by pointing                    |
|                                                                |
| Auntie Aerial: “Now you know. Tell them over the sprouts.”     |
+----------------------------------------------------------------+
```

```bash
cargo run -q -- list --no-color
```

```
TONIGHT'S GUESTS · 22 creatures

-- ST NICHOLAS'S COMPANIONS (companions) ---------------------------------------
The saint brings the presents. These are the ones he brings along for everybody
else.
  Krampus                   The horned, chain-rattling demon who deals with the…
  Belsnickel                The ragged, fur-clad gift-giver who brings sweets i…
  Père Fouettard            St Nicholas's grim, black-robed servant, who carrie…
  Hans Trapp                The Alsatian bogeyman who comes with the Christkind…
  Knecht Ruprecht           St Nicholas's bearded German servant, who asks chil…
...
```

The names line up, the descriptions are cut to fit, and nothing is wider than 80 columns.

```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
git add .
git commit -m "feat: colour, boxes and plain mode"
```

## Try it

- Drag your terminal narrower and run `random` and `list` again. The box and the rules shrink to fit, down to 40 columns.
- In `boxed`, change `visible_width(line)` to `line.len()` and run `random` in colour. The right-hand side goes ragged, because `len` counts the bytes of the colour codes. Put it back.
- Run `NO_COLOR=1 cargo run -q -- list`.

## 🎄 Festive fact

Until Rust 1.70, in June 2023, the standard library couldn't tell you whether stdout was a terminal, and nearly every command-line program depended on a small crate called `atty` to find out. `std::io::IsTerminal` replaced it. It's a trait, implemented for stdin, stdout, stderr and files, which is why the code says `io::stdout().is_terminal()` and why the trait has to be imported for that method to exist.

## 🕯️ Before morning

Give `Painter` a method `rule(&self, hex: &str) -> String` that returns a full-width line with no title, and use it to close the `list` page above the "Meet one" line.

Hint: `heading` already chooses the character. A rule is `self.text_width()` copies of it, painted. Then read the plain-mode test: would it catch a `─` slipping into plain output? Improve it so that it would.

> 📺 *"There. Green, gold, and perfectly legible in black and white. Tomorrow is the last night you type every line yourself, and we finish in style: every guest in their own colours, with a portrait."*

[← Night 8](night-08.md) · [Index](README.md) · [Night 10 →](night-10.md)
