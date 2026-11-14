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

Every word Auntie Aerial says is in [`src/host.rs`](../src/host.rs), seventy lines with no drawing in them:

```rust
/// The host's on-air name.
pub const NAME: &str = "Auntie Aerial";

/// The fictional station she broadcasts on.
pub const CHANNEL: &str = "Channel 25";

const GREETINGS: &[&str] = &[
    // ...
    "Ah, you've tuned in. Lovely. Mind the tinsel, it's holding the aerial up.",
    // ...
];
```

`&[&str]` is a slice of string slices: a borrowed list of borrowed text, all of it compiled into the program. Text like that lives for the whole run, which is what `&'static str` means, so the functions that choose a line can return one without copying it:

```rust
/// The opening line for the big nights, if this is one.
pub fn seasonal_greeting(month: i8, day: i8) -> Option<&'static str> {
    match (month, day) {
        (12, 24) => Some(CHRISTMAS_EVE),
        (12, 25) => Some(CHRISTMAS_DAY),
        _ => None,
    }
}

// ...

/// A random opening line.
pub fn greeting(rng: &mut Rng) -> &'static str {
    rng.pick::<&str>(GREETINGS)
}
```

The random choice goes through the `Rng` you wrote on Night 4, as every random choice in the program does. The model puts the two together when it is made:

```rust
        let greeting = host::seasonal_greeting(opts.today.month(), opts.today.day())
            .unwrap_or_else(|| host::greeting(&mut rng));
```

`unwrap_or_else` takes the value out of a `Some`, or runs the closure for a `None`. The closure matters: with plain `unwrap_or`, the random greeting would be drawn every time, and on Christmas Eve that would use up a number from the `Rng` for nothing and shift every later choice.

Keeping the voice in one file means it can be read and edited as writing. Nobody has to hunt through drawing code to find out whether she has already made the joke about the aerial.

## Asking for ticks

On Night 11 the `App` trait had a method with a default: `tick_every`, returning `None`. The explorer overrides it in [`src/explorer/model.rs`](../src/explorer/model.rs):

```rust
const INTRO_TICK: Duration = Duration::from_millis(70);
const SNOW_TICK: Duration = Duration::from_millis(300);
```

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

The event loop asks this question every time round. During the intro the answer is "every 70 milliseconds", about fourteen frames a second. On the gallery and on most pages the answer is `None`, and the loop goes back to sleeping until a key arrives. Nothing has to be started or cancelled: the animation stops because the screen changed and the answer changed with it.

And a tick does one thing. It adds one to `frame`. That number is the only animation state the explorer has.

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

Eighteen frames of static, two with the title flashed in gold, then the greeting at three characters a tick. `div_ceil` is division that rounds up: four characters at three a tick need two ticks, not one. Every drawing function below asks "what does frame N look like?" and nothing else, so any moment of the intro can be drawn directly, in any order. A key press during the intro uses this: it sets `frame` to `done_frame`, and the finished card appears.

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

One call decides one cell. `mix` (below) turns the cell's position and the frame into a scrambled number `h`. `h % 100` is then a number from 0 to 99, and the cell is lit if it is below `thinning`, which starts at 85 and falls to 19 by the last frame. Halfway through, the heavy blocks give way to dots and stars, so the static turns into something like snow before the real snow starts.

## Snow with no state

The usual way to animate snow is a list of flakes, each with a position and a speed, moved on every tick. The explorer has no such list. This is all of it:

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

It answers one question: is there a flake in this cell on this frame? `h` depends on the seed and the column only, so each column has one fixed number, and everything about that column's flake is read from different bits of it:

- `h.is_multiple_of(3)`: about one column in three has a flake at all.
- `slowness` is 1, 2 or 3: how many frames the flake takes to fall one row.
- `lap` is how long its journey is, between one and two screen heights, so it spends a while out of sight below the bottom before it comes round again, and the columns do not fall in step.
- `at` is the row it has reached: the frame count, slowed, plus a starting offset, wrapped to the lap.

If `at` is the row being asked about, there is a flake. `h >> 8` shifts the bits right by eight places, which is a cheap way to get several unrelated-looking numbers from one.

`mix` is in [`src/rng.rs`](../src/rng.rs), beside the generator from Night 4, and uses the same scrambling steps:

```rust
pub fn mix(seed: u64, a: u64, b: u64) -> u64 {
    let mut z = seed
        .wrapping_add(a.wrapping_mul(0x9E37_79B9_7F4A_7C15))
        .wrapping_add(b.wrapping_mul(0xC2B2_AE3D_27D4_EB4F));
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}
```

The difference from `Rng` is in the signature: no `&mut self`. It keeps nothing, so the same inputs always give the same output. That one property pays for itself four times:

- **Drawing stays read-only.** `draw` takes `&self`. A list of flakes would have to be moved somewhere, and the snow would need its own update step.
- **Resizing costs nothing.** Drag the window wider and the new columns already have their flakes. There is no list to rebuild.
- **It can be tested.** `the_same_frame_always_has_the_same_snow` asserts that frame 7 equals frame 7 and differs from frame 8.
- **It can be reused.** On the Dickens pages, which are drawn as engravings, `portrait` in `view.rs` calls the same `flake` to drift snow through the empty parts of the plate, at the slower `SNOW_TICK`.

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

`[&'static str; 2]` is an array of exactly two strings, and the length is part of the type. The font has only the eleven letters in TERMINAL OF TINSEL. `logo` then chooses what fits:

```rust
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
```

One line of big letters, or two stacked, or plain text. `let [a, b] = top;` takes an array apart by pattern, which the compiler allows because it knows there are exactly two.

## Typing the greeting

Revealing text a few characters at a time has a trap. If you wrap whatever has been typed so far, a long word starts on one line and jumps to the next when it outgrows it. `typed_greeting` wraps the *whole* greeting first and then reveals it:

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

`budget` is how many characters have been typed, worked out from `m.frame`. Each line takes what it can and passes on the rest. The `+ 1` pays for the space that wrapping removed at the line break. Lines the typing has not reached come out empty but are still there, so the card keeps its height and nothing on screen moves except the cursor.

```bash
DUMP=1 cargo test dump_screens -- --nocapture
```

```
── intro, frame 400 ──
                                                                                  +

               +
                                       +
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

                               *    +

                             .

                                                                                              .
                                                                     *

                              .            .                                                .
```

That is the finished card as the test draws it, with the seed fixed at 25 and the date at 12 November 2026.

## Plain mode, after the drawing

`--no-color` and `NO_COLOR` promise three things: no colour, no emoji and ASCII borders. Night 17 covers this properly, but the intro shows why each is done at a different moment. Colour is left out where styles are made, and emoji where text is built, through `icon`. The drawing characters are handled last, in [`src/plain.rs`](../src/plain.rs):

```rust
pub fn plainify(buf: &mut Buffer) {
    for cell in &mut buf.content {
        if let Some(plain) = ascii(cell.symbol()) {
            cell.set_symbol(plain);
        }
    }
}
```

`view::draw` calls it once the screen is complete. It can wait until then because swapping `│` for `|` or `░` for `:` changes one cell into one cell, and no layout moves. An emoji is two columns wide, so taking one out shifts everything after it, and that has to happen before anything is measured. Doing the swap afterwards also means the static, the box borders and the scroll markers need no `if plain` of their own: there is one table, and one test, `no_colour_also_means_no_emoji_and_ascii_borders`, that checks every cell of the intro, the gallery, a search and two creatures' pages against it.

The logo is the exception. Swapped, the block letters would be rows of `#`, so in plain mode the intro passes `logo` a width of 0 and gets the spelled-out title.

## Try it

- `TERMINAL_OF_TINSEL_SEED=25 cargo run -- creature` twice. Same greeting, same snow. Then run it without the seed.
- Press a key during the static, then another. Read `intro_key` in `model.rs`: the first key finishes the intro and the second enters.
- `cargo run -- creature marley --explore` in a window at least 100 columns wide, and watch the plate. Find the line in `portrait` that keeps the snow off the figure.
- `NO_COLOR=1 cargo run -- creature` and compare the title.

## 🎄 Festive fact

Two methods in tonight's code are recent arrivals in the standard library. `div_ceil` on unsigned integers became stable in Rust 1.73, and `is_multiple_of` in Rust 1.87. Before them the same things were written as `(n + d - 1) / d` and `n % d == 0`. Both old forms still work, and both are easier to get wrong.

## 🕯️ Before morning

Every glyph must have two rows of the same width, or the letters after it lean. Nothing checks that today. Write a test in `intro.rs` that does, then add a glyph for `C` and prove your test covers it.

Hint: loop over `"TERMINALOFSC".chars()` and compare `glyph(ch)[0].width()` with `glyph(ch)[1].width()`. `width` is already imported at the top of the file. For the `C`, look at how `L` and `E` are built from `█`, `▀` and `▄`.

> 📺 *"So now you know: a ladder, a list of lines, and snow that falls by arithmetic. Don't tell the children. Tomorrow night we start on the crackers, and somebody has to write the questions that go inside."*

[← Night 14](night-14.md) · [Index](README.md) · [Night 16 →](night-16.md)
