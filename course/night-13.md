# Night 13 · St Lucy's Night: Turn the Page

> 📺 *"The thirteenth of December. Under the old calendar this was reckoned the longest night of the year, and in Norway it belonged to Lussi, who rode out to see whose Christmas work wasn't finished. Ours isn't. We have one page per guest and far too much to say on it, so tonight we cut it into sections and build the hall they all wait in."*

**Tonight you'll learn**
- Enums for state: which screen is showing, which section is open
- Routing keys by state, with `match` guards and range patterns
- Sections that depend on the data: a creature only has the tabs it can fill
- Stepping round a list with `%`, forwards and backwards, without going below zero
- A gallery with a cursor, and keeping the cursor on screen
- What the `'a` in `Model<'a>` is for

**Where we are:** a creature's page is built as lines, wrapped to the width, scrolled and tested at six sizes. So far we have looked at one section of one page.

## Which screen?

The explorer has three screens, and the model in [`src/explorer/model.rs`](../src/explorer/model.rs) says so with an enum:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Intro,
    Gallery,
    Detail,
}
```

You could track this with two booleans, `in_intro` and `in_gallery`. Two booleans have four combinations, and one of them (both true) means nothing. An enum with three variants has exactly three values. When the state can only be one of a few things, an enum makes the impossible combinations unwritable.

`Copy` lets a `Screen` be passed around by value like a number. `PartialEq` and `Eq` allow `self.screen == Screen::Intro`.

Every key press arrives at one function, which decides who should hear it:

```rust
    pub fn on_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            if key.code == KeyCode::Char('c') {
                self.quit();
            }
            return;
        }
        if self.screen == Screen::Intro {
            return self.intro_key(key.code);
        }
        if self.searching {
            return self.search_key(key.code);
        }
        match key.code {
            KeyCode::Char('q') => self.quit(),
            KeyCode::Char('?') => self.show_help = !self.show_help,
            KeyCode::Char('/') => {
                // ...
            }
            code if self.screen == Screen::Gallery => self.gallery_key(code),
            code => self.detail_key(code),
        }
        self.clamp_scroll();
    }
```

Read it as a list of priorities. Ctrl+C quits from anywhere and is checked first. In raw mode it arrives as an ordinary key, so if the program did not handle it, nothing would. Then the intro takes every key, then a search does (tomorrow night). Only then do `q`, `?` and `/` mean what they usually mean, and whatever is left goes to the screen that is showing.

`code if self.screen == Screen::Gallery =>` is a **match guard**. The pattern `code` matches anything and gives it a name. The `if` then decides whether this arm is taken. If not, matching carries on to the next arm, which is the same pattern with no guard.

## Sections as an enum

```rust
/// A numbered section of a creature's page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Facts,
    Legend,
    Film,
    Myths,
    Quotes,
    Stats,
    Legacy,
}
```

Not every creature has all seven. Most folklore never had a classic film, and tonight's guest has no quotations: the rule is that quotes come from public-domain sources, and nobody wrote Lussi's lines down. An empty section with a heading and nothing under it would be a poor page, so the tabs are worked out from the data:

```rust
/// The sections a creature has something to say in.
pub fn tabs_for(c: &Creature) -> Vec<Tab> {
    let mut tabs = vec![Tab::Facts];
    if !c.legend.is_empty() {
        tabs.push(Tab::Legend);
    }
    if c.film.is_some() {
        tabs.push(Tab::Film);
    }
    // ...
    tabs.push(Tab::Stats);
    if !c.legacy.is_empty() {
        tabs.push(Tab::Legacy);
    }
    tabs
}
```

Lussi gets five: Facts, Legend, Myths, Stats and Legacy. Marley's Ghost gets all seven. Nothing stores this list. It is worked out from the creature whenever it is wanted, so it cannot disagree with the data, and a community pack with short creatures (Night 22) gets short tab bars for free.

The names have two lengths, chosen by matching on a pair:

```rust
    pub fn name(self, wide: bool) -> &'static str {
        match (self, wide) {
            (Tab::Facts, _) => "Facts",
            (Tab::Legend, _) => "Legend",
            (Tab::Film, true) => "On Screen",
            (Tab::Film, false) => "Screen",
            // ...
        }
    }
```

`(self, wide)` is a tuple, built on the spot to be matched. `_` means "either". The compiler checks that every combination of seven tabs and two booleans is covered, so adding an eighth `Tab` is a compile error here until it has a name.

## Moving between sections

The model stores the open section as `tab: usize`, an index into `tabs_for` the current creature. Here are the keys that change it, from `detail_key`:

```rust
            KeyCode::Tab => self.set_tab((self.tab + 1) % tabs),
            KeyCode::BackTab => self.set_tab((self.tab + tabs - 1) % tabs),
            // ...
            KeyCode::Char(c @ '1'..='9') => {
                let i = c as usize - '1' as usize;
                if i < tabs {
                    self.set_tab(i);
                }
            }
```

`tabs` is the number of sections. `%` is the remainder, so `(self.tab + 1) % tabs` steps forward and wraps from the last to the first. Going backwards, the obvious `self.tab - 1` would panic at zero, because `usize` cannot be negative. Adding `tabs` first and *then* taking one away gives the same answer after the `%` and never dips below zero. You will find this same line in the gallery and in moving between guests.

`c @ '1'..='9'` is two patterns in one. `'1'..='9'` is a range: it matches any character from `1` to `9`. `c @` keeps hold of whichever one matched. A `char` converts to its number with `as`, and the digits are consecutive, so `c as usize - '1' as usize` turns `'1'` into 0 and `'3'` into 2. Pressing `6` on Lussi's page does nothing, because `i < tabs` is false.

## The same section on the next guest

Press the right arrow on Krampus's legend and you would expect the next guest's legend, not their facts. But an index is no good for that: section 3 is Myths for Krampus and On Screen for Marley. So `open` remembers the *kind* of section and looks it up again:

```rust
    fn open(&mut self, i: usize) {
        let previous = (self.screen == Screen::Detail).then(|| self.current_tab());
        self.index = i;
        self.cursor = i;
        self.enter(Screen::Detail);
        self.scroll = 0;
        self.revealed = self.reveal;
        self.tab = previous
            .and_then(|t| self.tabs().iter().position(|&x| x == t))
            .unwrap_or(0);
    }
```

`bool::then` turns a condition into an `Option`: `Some(the tab)` if we were already on a page, `None` if we came from the gallery. After the creature changes, `position` searches the new creature's tabs for the same variant. `and_then` runs that search only if there was a previous tab, and `unwrap_or(0)` falls back to Facts, which every creature has.

This is the reason `Tab` is an enum and not a number: `x == t` compares meanings.

## The tab bar

Drawing the bar is a loop over the same list, in [`src/explorer/view.rs`](../src/explorer/view.rs):

```rust
        .map(|(i, t)| {
            let label = format!(" {} {} ", i + 1, t.name(w >= WIDE));
            if i == m.tab {
                Span::styled(label, p.badge)
            } else {
                Span::styled(label, dim(m))
            }
        })
```

The number on each label is `i + 1`, the key that opens it. The labels then go through `join_fit` from `tui.rs`, which puts as many on a row as the width allows and starts another row for the rest. Run last night's dump and look at `marley, narrow`: seven tabs take two rows at 70 columns.

## The gallery

The gallery is a list of every creature with a cursor. Its keys are short:

```rust
    fn gallery_key(&mut self, code: KeyCode) {
        let n = self.vault.creatures.len();
        match code {
            KeyCode::Up | KeyCode::Char('k') => self.cursor = (self.cursor + n - 1) % n,
            KeyCode::Down | KeyCode::Char('j') => self.cursor = (self.cursor + 1) % n,
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = n - 1,
            KeyCode::Enter | KeyCode::Char(' ' | 'l') | KeyCode::Right => self.open(self.cursor),
            _ => {}
        }
    }
```

`|` in a pattern means "or", and it can nest: `KeyCode::Char(' ' | 'l')` is a space or an `l`. The `n - 1` is safe because `explorer::run` refuses an empty vault before a model is ever made.

The harder part is drawing. The list is longer than the screen, and the cursor must stay visible. `gallery_lines` builds the whole list and returns, alongside it, the number of the line the cursor landed on. Then `gallery_body` picks where the window starts:

```rust
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
```

Each creature takes two lines, a name and a description, and last night's `viewport` overwrites its first and last rows with markers. So the window must end at least one line after the description, and start at least one line before the name. Notice what the model does *not* hold: a scroll position for the gallery. The offset is worked out from the cursor on every draw, so the two cannot drift apart.

A test, `the_selection_stays_on_screen_in_a_short_gallery`, walks the cursor down every creature at a height of ten rows and checks the `▶` is on screen each time.

From 100 columns up, the selected creature's portrait is boxed beside the list, with last night's `beside`:

```
── gallery ──
 🎄 TERMINAL OF TINSEL 🎄                                                                              22 guests tonight

── St Nicholas's Companions ──                              ╭────────────────────────────────────────────────────────╮
▶ 😈 Krampus                                                │ 😈 Krampus                                             │
    The horned, chain-rattling demon who deals with the n…  │ First appeared: centuries-old customs, on greeting     │
  🧥 Belsnickel                                             │ cards from the 1800s                                   │
    The ragged, fur-clad gift-giver who brings sweets in…   │                                                        │
  🖤 Père Fouettard                                         │       /\                /\                             │
    St Nicholas's grim, black-robed servant, who carries…   │      /  \              /  \                            │
  🌾 Hans Trapp                                             │     | /\ \            / /\ |                           │
    The Alsatian bogeyman who comes with the Christkindel…  │     | \ \ \__________/ / / |                           │
...
```

## Borrowing the vault

One line of the model has gone unexplained:

```rust
pub struct Model<'a> {
    pub(super) vault: &'a Vault,
    // ...
}
```

The model does not own the creatures. It borrows the `Vault` that `main` loaded, and `'a` is the name of that borrow. Writing it on the struct tells the compiler that a `Model` must not outlive the vault it points into. `pub(super)` makes a field visible to the parent module, `explorer`, so the view and the tests can read it and the rest of the program cannot.

## Try it

- `cargo run -- creature lussi --explore`, then press `tab` until you are back where you started. Count the sections. Press `6`.
- `cargo run -- creature marley --explore`, press `3` for On Screen, then the right arrow. The next ghost has no film. Which section opens, and which line of `open` chose it?
- `cargo run -- creature --all --no-intro` in a window ten rows tall. Hold the down arrow and watch the two markers.

## 🎄 Festive fact

A `match` in Rust must cover every case, and the compiler proves it. Leave a variant out and the build stops with error E0004, "non-exhaustive patterns", naming the variant you forgot. `rustc --explain E0004` prints the full explanation in your terminal, as it does for any error code the compiler shows you.

## 🕯️ Before morning

Write a test that Lussi has exactly five sections and that pressing `6` leaves her page on the section it was showing. Then add a second check to it: Marley's Ghost has seven, and `7` opens the last of them.

Hint: start from `tab_and_numbers_switch_sections` in [`src/explorer/tests.rs`](../src/explorer/tests.rs). `on_page(&vault, "lussi")` opens her page, `m.tabs().len()` counts the sections and `m.tab` is the open one.

> 📺 *"Twenty-two guests, a hall to hold them, and a door for each. Lussi has checked the work and found it finished, which is more than most households manage. Tomorrow night somebody asks which of them it was that wanted porridge."*

[← Night 12](night-12.md) · [Index](README.md) · [Night 14 →](night-14.md)
