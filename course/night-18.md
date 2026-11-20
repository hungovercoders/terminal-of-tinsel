# Night 18 · Behind the Frosted Window

> 📺 *"There's somebody at the studio window, and the glass has frosted over. I can see an ear. Possibly a horn. You may wipe a little of it clear, but every wipe costs you points, and I shall be watching your sleeve."*

**Tonight you'll learn**
- A game made of two parallel arrays and one shuffled list
- Why the frost never comes back: a prefix of a fixed order
- `Option<char>` as "a character, or frost", and `bool::then_some`
- A struct that borrows: `Round<'a>` and what the `'a` says
- Private fields that keep a struct's promises
- Plain mode when colour was doing a job

**Where we are:** the quiz is playable. Tonight's game reuses its keys, its option lines and its layout, and adds one new idea: a picture revealed in stages.

## The rules, as data

Guess the Guest shows a creature's ASCII portrait with most of it hidden. Name the creature at once for 100 points, or press space to clear more frost for fewer. The whole rule book is three constants at the top of [`src/guess.rs`](../src/guess.rs):

```rust
/// How much of the portrait shows at each stage, in percent, and what a
/// right answer is worth there.
pub const CLEARED: [usize; 5] = [15, 30, 50, 75, 100];
pub const POINTS: [usize; 5] = [100, 80, 60, 40, 20];

/// The last stage: the window is clear.
pub const CLEAR: usize = CLEARED.len() - 1;
```

A stage is an index, 0 to 4, into both arrays. `CLEAR` is computed from the array's length at compile time, so adding a sixth stage means editing the two arrays and nothing else. The `5` is part of each array's type: six numbers in an array declared as five is a compile error, not a surprise at run time.

## A round

```rust
/// One portrait to identify.
#[derive(Debug, Clone)]
pub struct Round<'a> {
    pub creature: &'a Creature,
    pub options: Vec<String>,
    /// The index of the right option.
    pub answer: usize,
    /// What the player chose, once they have.
    pub chosen: Option<usize>,
    /// How far the frost had cleared when they chose.
    pub stage: usize,
    /// The portrait as a grid, padded to a rectangle.
    grid: Vec<Vec<char>>,
    /// The order the frost clears in, as cell numbers.
    order: Vec<usize>,
}
```

A quiz `Question` owns its text, because the text was made up by the generator. A `Round` points at a creature that already lives in the `Vault`, so it borrows it: `&'a Creature`. The `'a` is a lifetime, a name for "how long the thing I point at is alive". Writing `Round<'a>` tells the compiler that a round must not outlive the creatures it was made from, and the signature of the generator ties the two together:

```rust
pub fn rounds<'a>(rng: &mut Rng, all: &'a [Creature], n: usize) -> Vec<Round<'a>> {
```

Read it as: the rounds live no longer than `all`. You rarely write lifetimes on functions, because the compiler works out the common cases. A struct that holds a reference always needs one.

`grid` and `order` have no `pub`. Code outside this file can read a round's options but cannot rearrange its frost. That matters, because the next section depends on `order` never changing.

## Frost that never comes back

The obvious way to hide 85% of a picture is to roll a die for each cell on each draw. Then the picture would shimmer: a cell clear at one stage could be frosted at the next, and the player would lose what they had paid for. The fix is to decide everything once, when the round is made:

```rust
            let width = creature.ascii.lines().map(|l| l.chars().count()).max();
            let width = width.unwrap_or(0);
            let grid: Vec<Vec<char>> = creature
                .ascii
                .lines()
                .map(|l| {
                    let mut row: Vec<char> = l.chars().collect();
                    row.resize(width, ' ');
                    row
                })
                .collect();
            let mut order: Vec<usize> = (0..grid.len() * width).collect();
            rng.shuffle(&mut order);
```

The portrait's lines are different lengths, so each row is padded with spaces to the widest. Now every cell has a number, `y * width + x`, and `order` is those numbers, shuffled. It is the order in which the cells will clear. Showing a stage means clearing the first so-many cells of that list:

```rust
    pub fn frosted(&self, stage: usize) -> Vec<Vec<Option<char>>> {
        let width = self.grid.first().map_or(0, Vec::len);
        let total = self.grid.len() * width;
        let mut clear = vec![false; total];
        for &cell in &self.order[..total * CLEARED[stage.min(CLEAR)] / 100] {
            clear[cell] = true;
        }
        self.grid
            .iter()
            .enumerate()
            .map(|(y, row)| {
                row.iter()
                    .enumerate()
                    .map(|(x, ch)| clear[y * width + x].then_some(*ch))
                    .collect()
            })
            .collect()
    }
```

At stage 0 the slice is the first 15% of `order`. At stage 1 it is the first 30%, which contains the first 15%. A longer prefix always includes a shorter one, so a cleared cell stays clear, with no bookkeeping. `frosted` takes `&self` and changes nothing: ask for stage 2 twice and you get the same grid twice.

The result is a grid of `Option<char>`: `Some('/')` where the glass is clear, `None` where it is frosted. `then_some` is a method on `bool` that makes that in one step: `true.then_some(x)` is `Some(x)` and `false.then_some(x)` is `None`. Returning `Option` keeps this module ignorant of what frost looks like. That is the screen's business.

The frost itself has no state either, the same trick as the snow on Night 15:

```rust
/// The frost over a cell that hasn't cleared: a fixed, uneven pattern.
pub fn frost(x: usize, y: usize) -> char {
    ['░', '▒', '░', '*', '░'][(x * 7 + y * 3) % 5]
}
```

## Clues

From the third stage the game adds written clues:

```rust
    pub fn clues(&self, stage: usize) -> Vec<String> {
        let mut out = Vec::new();
        if stage >= 2 {
            let debut = &self.creature.debut;
            out.push(match (debut.year, &debut.era) {
                (Some(year), _) => format!("First appeared in {year}"),
                // Folklore has an era, not a year.
                (None, Some(era)) => format!("The legend: {era}"),
                (None, None) => format!("First appeared in {}", debut.title),
            });
        }
        if stage >= 3 {
            out.push(format!("Origin: {}", self.creature.origin));
        }
        // A clue mustn't name the guest, any more than a quiz clue does.
        out.iter()
            .map(|clue| crate::quiz::mask(clue, self.creature))
            .collect()
    }
```

The `match` is on a pair, which covers every combination of two `Option`s in three arms. Marley's Ghost has a year, 1843. Krampus has no year of first appearance, only an era, and the data says so instead of inventing one. Then every clue goes through Night 16's `mask`, because an origin can contain the name.

## The screen

`GuessApp` in [`src/games.rs`](../src/games.rs) has the same shape as `QuizApp`, plus one field: `stage`, how far the frost has cleared in the round being played. Two branches of its `on_key` are new:

```rust
        } else if matches!(key.code, KeyCode::Char(' ' | 'f')) {
            self.stage = (self.stage + 1).min(guess::CLEAR);
        } else {
            let round = &mut self.result.rounds[self.i];
            if let Some(n) = pick_option(key.code, &mut self.cursor, round.options.len()) {
                round.chosen = Some(n);
                round.stage = self.stage;
                self.result.completed = self.i + 1 == self.result.rounds.len();
                self.answered = true;
                self.cursor = n;
                self.quip = host::quip(&mut self.rng);
            }
        }
```

Space clears a stage and stops at the last. A guess writes the choice and the current stage into the round, and from then on the round can score itself:

```rust
    pub fn points(&self) -> usize {
        if self.correct() {
            POINTS[self.stage]
        } else {
            0
        }
    }
```

There are two `stage`s on purpose. The app's is where the frost is now, and the round's is where it was when the player guessed. Once a round is answered the window is drawn fully clear and in the creature's own colour, but the status line still reads the round's stage: "You guessed with 50% of the frost cleared".

Drawing the window turns each `Option<char>` into a styled span:

```rust
                        .map(|(x, cell)| match cell {
                            Some(ch) => Span::styled(ch.to_string(), shown),
                            // Plain mode has no colour to tell frost from
                            // portrait, so the frost is one character that
                            // no portrait is drawn with.
                            None if !ink.colour => Span::raw("?"),
                            None => Span::styled(guess::frost(x, y).to_string(), ink.dim()),
                        })
```

In colour, frost is dim and the portrait is bright, so `*` can be both a frost character and part of a drawing. Plain mode takes the colour away, and after last night's `plainify` the frost would be `:`, `%` and `*`, when four of the built-in portraits are drawn with `*`. So in plain mode the frost is `?` everywhere, and a test, `plain_frost_is_one_character_no_portrait_uses`, checks that no portrait in the packs contains a question mark. `None if !ink.colour` is a match guard: the arm applies only when the condition holds, and otherwise the next arm is tried.

One layout decision is worth reading in `lines()`: the options sit beside the window if the widest option fits there, and below it if not. That is judged once for the whole round from the option names, not from what is on screen now, so a clue arriving at stage 2 can never make the layout jump.

## Testing a promise

The claim "the frost never comes back" is a test in `guess.rs`:

```rust
        for (stage, percent) in CLEARED.iter().enumerate() {
            let clear = shown(stage).iter().filter(|c| **c).count();
            assert_eq!(clear, total * percent / 100);
            if stage > 0 {
                let before = shown(stage - 1);
                let now = shown(stage);
                assert!(before.iter().zip(&now).all(|(b, n)| !b || *n));
            }
        }
```

`shown(stage)` is a list of booleans, one per cell. `!b || *n` reads "if it was clear before, it is clear now". The same test ends by clearing the window fully and comparing it with the creature's portrait, character for character.

## Run it

```bash
DUMP=1 cargo test dump_games -- --nocapture
```

The test uses seed 25. After two presses of space, the top of the screen is:

```
 🔍 GUESS THE GUEST                                                   Round 1 of 3 · 0 pts

╭───────────────────────────────────────────╮   Who's at the window?
│ ░░░▒*xx░▒ ░░  *░ ░▒*░ ░▒ ░░ ▒ ░ ░ *░xx▒*░ │   Frost cleared: 50% · worth 60 pts
│ *░  ▒*x░xx ░    ░ ░  ░  ▒*░░░▒    x*░xxx  │
│  *░░x▒*░xxx* ░░ *░   *░ ░ *░ ░▒  ░░▒x░░░  │   ▶ 1. The Ghost of Christmas Past
│ ░▒  ░░▒* x░▒*░ ░x*░░xx*xxxx*░ ░ *xx░▒ ░x  │     2. The Mari Lwyd
│  ░▒ x░    ░xx*x░░xxxx░▒*xxxx*xxx▒* ░░▒*x░ │     3. The Yule Cat
│ ░░  *░░xxxxx░xx░░░▒*xx░x*xxxx*x░x▒xxxx▒x░ │     4. La Befana
│ *   ▒xx░░▒*xx░▒*x░   ░ ░  ░x░x*░░xxxxxxx* │
│    ░░▒*░x░▒x░░   ░ ░▒  ░    ░░▒xxxx▒*░xx  │   Clue: The legend: recorded in the 1800s,
│ ░▒    ▒*░x░▒ ░x░▒x░░░  ░ ░▒*░░░▒*░xx▒   ░ │   made famous by a poem of 1932
```

Half the window is still frosted, but two ears are showing at the top corners, and the clue is a legend made famous by a poem of 1932. `guess --json` prints the rounds with their answers through a small `RoundJson` struct that leaves the grid and the order out.

## Try it

- Play `cargo run -- guess -n 3`, and guess one round without pressing space.
- Play `cargo run -- guess --no-color` and watch the `?` thin out.
- Read `too_few_creatures_is_no_game` to see what happens with two creatures.

## 🎄 Festive fact

`bool::then_some` was stabilised in Rust 1.62, in 2022. Its older sibling `bool::then`, which takes a closure and so only does the work when the answer is `true`, arrived in Rust 1.50. `pick_option` on Night 17 used `then`; `frosted` uses `then_some` because copying one `char` is cheaper than a closure is worth.

## 🕯️ Before morning

Add a third clue at the last stage: the creature's pack. Hint: `Creature` has a `pack` field holding the pack's id; push `format!("Pack: {}", self.creature.pack)` when `stage >= 4`, then add a line to the test `clues_arrive_at_the_third_and_fourth_stages` asserting that Marley has three clues at stage 4.

> 📺 *"It was the cat. It's usually the cat. Tomorrow night the guests settle an old argument on the frozen pond, three rounds, and I've been asked to say there will be no refunds."*

[← Night 17](night-17.md) · [Index](README.md) · [Night 19 →](night-19.md)
