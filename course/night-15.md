# Night 15 · Your Host, Auntie Aerial

> 📺 *"Fourteen nights I've been doing the talking and not one of you has asked how I get up here. There's a ladder, since you're wondering, and a great deal of tinsel. Tonight you may look behind the set: where my lines are kept, how the snow falls without anybody shovelling it, and why the picture is all fuzz for the first second."*

**Tonight you'll learn**
- Keeping a character's lines in one module: constants, slices and `&'static str`
- Ticks: how a screen asks the event loop for animation, and stops asking
- Animation as a function of the frame number
- Snow with no state: hashing a position and a frame
- A block-letter font from `match`, with fallbacks for narrow screens
- Typewriter text that never re-wraps, and why plain mode swaps characters after drawing

**Where we are:** the explorer has a gallery, pages, sections and search. The first thing anybody sees of it is the one screen we have not opened: the intro.

## Where the host lives

Auntie Aerial's greetings, quips and sign-offs are in [`src/host.rs`](../src/host.rs), seventy lines with no drawing in them. (Her introduction for each guest is in that creature's JSON, as `hostIntro`.)

```rust
const GREETINGS: &[&str] = &[
    // ...
    "Ah, you've tuned in. Lovely. Mind the tinsel, it's holding the aerial up.",
    // ...
];
```

`&[&str]` is a slice of string slices, all of it compiled into the program. Text like that lives for the whole run, which is what `&'static str` means, so a function can return a line without copying it:

```rust
/// A random opening line.
pub fn greeting(rng: &mut Rng) -> &'static str {
    rng.pick::<&str>(GREETINGS)
}
```

The choice goes through the `Rng` from Night 4, as every random choice in the program does. A second function, `seasonal_greeting`, returns `Some` line on 24 and 25 December and `None` otherwise. The model takes the seasonal line if there is one and a random line otherwise, using `unwrap_or_else` with a closure, so that on Christmas Eve no number is drawn from the `Rng` for a greeting nobody will hear.

## Asking for ticks

On Night 11 the `App` trait had a method with a default: `tick_every`, returning `None`. The explorer overrides it in [`src/explorer/model.rs`](../src/explorer/model.rs):

```rust
    pub fn tick_every(&self) -> Option<Duration> {
        match self.screen {
            Screen::Intro => Some(INTRO_TICK),
            Screen::Detail if self.engraving() => Some(SNOW_TICK),
            _ => None,
        }
    }

    /// Advances the animation by a frame.
    pub fn on_tick(&mut self) {
        self.frame = self.frame.wrapping_add(1);
    }
```

The event loop asks this question every time round. During the intro the answer is `INTRO_TICK`, 70 milliseconds. On the gallery and on most pages it is `None`, and the loop goes back to sleeping until a key arrives. Nothing has to be started or cancelled: the animation stops because the screen changed and the answer changed with it.

A tick does one thing: it adds one to `frame`. That number is the only animation state the explorer has.

## A timeline in ticks

[`src/explorer/intro.rs`](../src/explorer/intro.rs) opens with the schedule:

```rust
/// The intro's timeline, in ticks.
const TUNE_FRAMES: usize = 18; // static while the set warms up
const FLASH_FRAMES: usize = 2; // the star on the tree lights up
const TYPE_SPEED: usize = 3; // characters of the greeting typed per tick

/// The frame at which the greeting has been typed in full.
pub fn done_frame(greeting: &str) -> usize {
    TUNE_FRAMES + FLASH_FRAMES + greeting.chars().count().div_ceil(TYPE_SPEED)
}
```

Eighteen frames of static, two with the title flashed in gold, then the greeting at three characters a tick. `div_ceil` is division that rounds up: four characters at three a tick need two ticks. Every drawing function below asks "what does frame N look like?" and nothing else, so any moment can be drawn directly. A key press during the intro uses this: it sets `frame` to `done_frame`, and the finished card appears.

## Static

```rust
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
```

One call decides one cell. `mix` (below) turns the cell's position and the frame into a scrambled number `h`. `h % 100` is from 0 to 99, and the cell is lit if that is below `thinning`, which starts at 85 and falls to 19 by the last frame.

## Snow with no state

The usual way to animate snow is a list of flakes, each with a position and a speed, moved on every tick. The explorer has no such list:

```rust
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
```

It answers one question: is there a flake in this cell on this frame? `h` depends on the seed and the column only, and everything about that column's flake is read from different bits of it:

- `h.is_multiple_of(3)`: about one column in three has a flake at all.
- `slowness` is 1, 2 or 3: how many frames the flake takes to fall one row.
- `lap` is how long its journey is, between one and two screen heights, so it spends a while out of sight before it comes round again.
- `at` is the row it has reached: the frame count, slowed, plus a starting offset, wrapped to the lap.

If `at` is the row being asked about, there is a flake. `h >> 8` shifts the bits right by eight places, a cheap way to get several unrelated-looking numbers from one.

`mix` is in [`src/rng.rs`](../src/rng.rs), beside the generator from Night 4, and ends with the same scrambling steps. Its signature is the difference:

```rust
pub fn mix(seed: u64, a: u64, b: u64) -> u64 {
```

There is no `&mut self`. It keeps nothing, so the same inputs always give the same output. That one property pays for itself four times:

- **Drawing stays read-only.** `draw` takes `&self`, and there is no list of flakes to move.
- **Resizing costs nothing.** Drag the window wider and the new columns already have their flakes.
- **It can be tested.** `the_same_frame_always_has_the_same_snow` asserts that frame 7 equals frame 7 and differs from frame 8.
- **It can be reused.** On the Dickens pages, drawn as engravings, `portrait` in `view.rs` calls the same `flake` to drift snow round the figure, at the slower `SNOW_TICK`.

The seed is one number taken from the `Rng` when the model is made, so `TERMINAL_OF_TINSEL_SEED` fixes the snow along with everything else.

## Big letters

The title is drawn in a font two rows high, and the font is a `match`:

```rust
fn glyph(ch: char) -> [&'static str; 2] {
    match ch {
        'T' => ["▀█▀", " █ "],
        'E' => ["█▀▀", "██▄"],
        // ...
        _ => [" ", " "],
    }
}
```

`[&'static str; 2]` is an array of exactly two strings, and the length is part of the type. The font has only the eleven letters in TERMINAL OF TINSEL. `logo` then chooses what fits the width: the whole title on one line of big letters, or TERMINAL stacked above OF TINSEL, or on a very narrow screen the plain words.

## Typing the greeting

Revealing text a few characters at a time has a trap. Wrap whatever has been typed so far, and a long word starts on one line and jumps to the next when it outgrows it. `typed_greeting` wraps the *whole* greeting first and then reveals it:

```rust
    let wrapped = textwrap::wrap(&full, w.saturating_sub(4).clamp(10, 70));
    let width = wrapped.iter().map(|l| l.width()).max().unwrap_or(0);
    let mut typing = true;
    let lines = wrapped
        .iter()
        .map(|text| {
            let n = text.chars().count();
            let mut shown: String = text.chars().take(budget).collect();
            if typing && budget < n {
                // The cursor sits where the typing has got to, and nowhere else.
                shown.push('▌');
                typing = false;
            }
            budget = budget.saturating_sub(n + 1);
            Line::styled(shown, style)
        })
        .collect();
```

`budget` is how many characters have been typed, worked out from `m.frame`. Each line takes what it can and passes on the rest. The `+ 1` pays for the space that wrapping removed at the line break. Lines the typing has not reached come out empty but are still there, so the card keeps its height.

```bash
DUMP=1 cargo test dump_screens -- --nocapture
```

```
── intro, frame 400 ──
...
         .                           *  Channel 25 presents  *

                 ▀█▀ █▀▀ █▀█ █▀▄▀█ █ █▄ █ ▄▀█ █     █▀█ █▀▀   ▀█▀ █ █▄ █ █▀ █▀▀ █
      +           █  ██▄ █▀▄ █ ▀ █ █ █ ▀█ █▀█ █▄▄   █▄█ █▀     █  █ █ ▀█ ▄█ ██▄ █▄▄
                                                                                     .
                                     ~ The Midwinter Special ~
                 .
     +          📺 Auntie Aerial: Pull up a chair by the fire, pour something warm
       .        and turn the lights down low. Tonight's guests have come a long way
                through the snow.
                                          +
                                   🎄 43 sleeps until Christmas.
                                  +
                       Press any key to meet tonight's guests...  (q to quit)
...
```

The finished card, at the test's seed of 25 and date of 12 November 2026.

## Plain mode, after the drawing

`--no-color` and `NO_COLOR` promise three things: no colour, no emoji and ASCII borders. Each is done at a different moment. Colour is left out where styles are made, and emoji where text is built, through `icon`. The drawing characters are handled last: `view::draw` calls `plainify`, in [`src/plain.rs`](../src/plain.rs), once the screen is complete. It can wait until then because swapping `│` for `|` or `░` for `:` changes one cell into one cell, and no layout moves. An emoji is two columns wide, so taking one out shifts everything after it, and that has to happen before anything is measured. Doing the swap afterwards also means the static, the borders and the scroll markers need no `if plain` of their own: there is one table, and a test, `no_colour_also_means_no_emoji_and_ascii_borders`, that checks every cell against it.

The logo is the exception. Swapped, the block letters would be rows of `#`, so in plain mode the intro passes `logo` a width of 0 and gets the spelled-out title.

## Try it

- `TERMINAL_OF_TINSEL_SEED=25 cargo run -- creature` twice. Same greeting, same snow. Then run it without the seed.
- `cargo run -- creature marley --explore` in a window at least 100 columns wide. Find the line in `portrait` that keeps the snow off the figure.
- `NO_COLOR=1 cargo run -- creature` and compare the title.

## 🎄 Festive fact

Two methods in tonight's code are recent arrivals in the standard library. `div_ceil` on unsigned integers became stable in Rust 1.73, and `is_multiple_of` in Rust 1.87. Before them the same things were written as `(n + d - 1) / d` and `n % d == 0`.

## 🕯️ Before morning

Every glyph needs two rows of the same width, and nothing checks that today. Write a test in `intro.rs` that does, then add a glyph for `C` and prove your test covers it.

Hint: loop over `"TERMINALOFSC".chars()` and compare `glyph(ch)[0].width()` with `glyph(ch)[1].width()`. `width` is already imported at the top of the file. For the `C`, look at how `L` and `E` are built from `█`, `▀` and `▄`.

> 📺 *"So now you know: a ladder, a list of lines, and snow that falls by arithmetic. Don't tell the children. Tomorrow night we start on the crackers, and somebody has to write the questions that go inside."*

[← Night 14](night-14.md) · [Index](README.md) · [Night 16 →](night-16.md)
