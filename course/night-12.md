# Night 12 · Fitting the Screen

> 📺 *"Some of you are watching on a set the size of a wardrobe. Some of you are watching on something you found in a cracker. The programme must fit both, and I don't intend to stand behind each of you checking. Tonight we teach it to measure the room, and then we teach the tests to do the checking for us."*

**Tonight you'll learn**
- Why a page is built as a `Vec<Line>` by a plain function of the model
- What `Line<'static>` means, and why owned text keeps lifetimes out of the way
- Wrapping to a width, and putting two blocks side by side
- Scrolling: an offset, a viewport, and clamping
- Placing the header, the page and the keys on the frame with `Rect`
- Testing a screen without a terminal: `TestBackend` at small and large sizes

**Where we are:** `tui::run` takes over the terminal and asks an `App` to draw a frame each time round. Tonight is about what the explorer draws on it.

## Lines first, frame last

The opening comment of [`src/explorer/view.rs`](../src/explorer/view.rs) states the design:

> Every screen is built as lines of styled text and then laid on the frame, so a page is a plain function of the model and can be tested without a terminal.

A `Line` is ratatui's type for one row of text. It holds a list of `Span`s, and a `Span` is a piece of text with one style. So a page is a `Vec<Line>`, and [`src/tui.rs`](../src/tui.rs) gives that a short name:

```rust
pub type Lines = Vec<Line<'static>>;
```

`type` makes an alias, not a new type. The `'static` is a lifetime. A `Span` can either borrow its text or own it. A borrowed one would tie every line to the `Creature` the text came from, and every function that passed lines around would have to say so in its signature. `Line<'static>` promises that the line borrows nothing short-lived: its text is an owned `String`, or a literal compiled into the program. That costs a copy of each string per frame, and in return lines can be built, joined, boxed and returned with no lifetime notes at all.

Here is a whole section of a creature's page:

```rust
    fn legend(&self) -> Lines {
        let mut out = self.heading("The Legend");
        out.extend(wrap(&self.c.legend, self.width, self.p.body));
        out
    }
```

It reads a creature and a width, and returns lines. It does not know where on the screen it will go, or whether a terminal exists. Three things follow:

- **It can be measured.** `lines.len()` is the page's height, which scrolling needs.
- **It can be combined.** A function that takes lines and returns lines can put a box round them or set them beside a portrait.
- **It can be tested.** Call it and look at what comes back.

## Wrapping

On Night 9 you wrapped text with `textwrap` for printing. The explorer uses the same crate and turns each wrapped row into a `Line`:

```rust
pub fn wrap(text: &str, width: usize, style: Style) -> Lines {
    textwrap::wrap(text, width.max(10))
        .into_iter()
        .map(|l| Line::styled(l.into_owned(), style))
        .collect()
}
```

`textwrap::wrap` returns rows that may borrow from `text`. `into_owned()` turns each into a `String`, which is what makes the result `'static`. `width.max(10)` is a floor: on a terminal three columns wide there is no good answer, but there must be *an* answer, and it must not be a panic. Its neighbour `hanging` wraps text under a bullet, indenting every row after the first.

## Wide and narrow

The width decides more than where rows break. From 100 columns up, the portrait sits beside the text. Below that, it goes underneath:

```rust
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
        // ...
    }
    lines
}
```

`boxed` takes lines and returns them inside a rounded border. `beside` takes two blocks of lines and joins them row by row, padding the left one to a fixed width. Both are possible only because a page is data. The text is asked for at a *narrower* width, `w - plate_width - 3`, so it wraps to the space that is left.

## Scrolling

A page can be longer than the terminal. The model keeps one number, `scroll`, the index of the first line to show, and one function cuts the window:

```rust
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
```

The two early returns come first, so by the subtraction `lines.len() - height` the answer cannot be below zero. `usize` has no negative numbers. `drain` moves a range of items out of a vector without copying them.

The keys only change the number. Down adds one, `End` sets it to `usize::MAX / 2`, which is "further than any page", and then one function in [`src/explorer/model.rs`](../src/explorer/model.rs) pulls it back into range:

```rust
    fn clamp_scroll(&mut self) {
        if self.screen != Screen::Detail {
            return;
        }
        let max = view::body_lines(self)
            .len()
            .saturating_sub(view::body_height(self));
        self.scroll = self.scroll.min(max);
    }
```

`saturating_sub` is subtraction that stops at zero: a page shorter than the screen has a maximum scroll of 0. `clamp_scroll` runs after a key and after every change of size, so no key handler has to know where the end is. The model asks the view how long the page is, which it can do because the page is a function.

## Onto the frame

Only the last few lines of `view.rs` touch the frame. By then there are three blocks of lines, the header, the body and the footer:

```rust
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
```

A `Rect` is a rectangle of the screen: x, y, width, height. `Paragraph` is a widget that draws lines into one. Every height is capped with `.min(...)` before it is subtracted from anything.

ratatui also has a `Layout` type that splits an area by constraints. The explorer does not use it: its header is as tall as its lines happen to be at this width, so the heights are known before anything is placed, and three rectangles are quickly worked out by hand.

The last step of all is in `draw`: in plain mode, `plain::plainify` swaps box-drawing characters for ASCII once the screen is drawn. Night 15 comes back to why that happens afterwards.

## Testing a screen

ratatui ships a backend that draws into memory, `TestBackend`. `snapshot`, in `tui.rs`, draws an app onto one and returns the screen as a string:

```rust
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
        // ...
}
```

The buffer is one long list of cells, and `chunks` cuts it into rows. With that (wrapped in a helper called `screen`), and a helper `press` that calls `on_key`, the tests in [`src/explorer/tests.rs`](../src/explorer/tests.rs) drive the explorer as a player would:

```rust
#[test]
fn every_page_draws_at_every_size() {
    let vault = Vault::load();
    for (width, height) in [(1, 1), (20, 6), (40, 12), (80, 24), (100, 30), (160, 50)] {
        for c in &vault.creatures {
            let mut m = on_page(&vault, &c.id);
            for tab in 0..tabs_for(c).len() {
                press(&mut m, KeyCode::Char(char::from(b'1' + tab as u8)));
                press(&mut m, KeyCode::Char('r'));
                press(&mut m, KeyCode::End);
                let text = screen(&mut m, width, height);
                if width >= 80 && height >= 24 {
                    assert!(
                        text.contains(&c.name.to_uppercase()),
                        "{} at {width}x{height}, tab {tab}:\n{text}",
                        c.id
                    );
                }
            }
        }
        // ...
    }
}
```

Every section of every creature, at six sizes, scrolled to the end. At the small sizes it asserts nothing about the text: the test is that drawing returns at all. A subtraction below zero or an index past the end of a vector panics, and a panic fails the test. The `(1, 1)` terminal is there to find exactly those.

One test prints sample screens when asked:

```bash
DUMP=1 cargo test dump_screens -- --nocapture
```

```
── krampus, narrow ──
 🎄 TERMINAL OF TINSEL 🎄     Guest 1 of 22 · St Nicholas's Companions
😈  KRAMPUS
The horned, chain-rattling demon who deals with the naughty while St
Nicholas rewards the good
📺 Auntie Aerial: “First guest of the season, and he's brought his ow…
 1 Facts   2 Legend   3 Myths   4 Stats   5 Legacy

Festive Facts

 • Krampus is a horned, hairy creature from Alpine folklore
 • He accompanies St Nicholas, who rewards the good while Krampus
   punishes the naughty
 • His night is 5 December, Krampusnacht, the eve of St Nicholas Day
 • He carries chains, bells and bundles of birch switches called Ruten
 • Naughty children might be stuffed into his sack or basket and
   carried away
 • In Krampuslauf parades, costumed Krampuses charge through town
   streets
 • Austrians sent each other Krampus greeting cards in the 1800s

  ▼ more below

←/→ guest · tab section · ↑/↓ scroll · / search · g gallery · ? help
q quit
```

That is Krampus at 70 columns by 24 rows: a squeezed header, short tab names, and the keys wrapped onto two rows.

## Try it

- Run the dump and compare `krampus, wide` with `krampus, narrow`. Find the constant in `view.rs` that decides between them.
- In `detail_header`, find `compact` and `tiny`. What does the header give up first on a short terminal, and why does the tab bar never go?
- Run `cargo test scrolling_stops_at_the_ends` and read the test. Which lines show that the scroll offset follows when the terminal gets taller?

## 🎄 Festive fact

In a debug build, Rust checks integer arithmetic: taking 5 from a `usize` that holds 3 panics with "attempt to subtract with overflow". In a release build the check is off by default and the value wraps round to a very large number. That is why this code says what it wants each time, with `saturating_sub` and `min`, and does not leave it to the build profile.

## 🕯️ Before morning

Write a test, in the repository's `src/explorer/tests.rs`, that the keys are always on the bottom row: for every creature, at 100 by 30, the last line of the screen contains `q quit`.

Hint: copy the shape of `naming_a_creature_skips_the_intro`. `screen(&mut m, 100, 30)` returns one string, and `.lines().last()` gives an `Option<&str>`. Then try it at 40 by 12 and look at how `join_fit` wraps the keys before you decide what the assertion should be.

> 📺 *"It fits the wardrobe and it fits the cracker toy, and I have six sizes of proof. Tomorrow is St Lucy's night, the longest of the year by the old reckoning, so we shall need more than one page to get through it."*

[← Night 11](night-11.md) · [Index](README.md) · [Night 13 →](night-13.md)
