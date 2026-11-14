# Night 19 · The Snowball Showdown

> 📺 *"Every family has the argument. Who'd win: the goat-horned gentleman from the Alps or the very large cat from Iceland? Tonight we settle it properly, on the frozen pond, over three rounds. No rules, no refunds, and mind the punch bowl."*

**Tonight you'll learn**
- A simulation: decide everything first, tell the story afterwards
- A table of function pointers, and closures that become them
- Stats plus a die, and what to do about a tie
- Narration that reads the same for one creature or thirteen
- Dramatic pauses, `--fast`, and why a pipe never waits
- A test that runs two hundred seeds

**Where we are:** two games take over the screen. Tonight's does not: it prints, like `creature` and `list`, so it works in a pipe and needs no `App`.

## Stats and a die

Every creature has a stat card: strength, speed, cunning and dread, each from 1 to 10. A showdown is three rounds. Each round picks a stat, each side adds a six-sided die to its own, and the higher total wins the round. Best of three wins the bout. With a die, Krampus (dread 9) usually out-stares the Yule Lads (dread 4), but not every time.

The die is one line on `Rng` in [`src/rng.rs`](../src/rng.rs):

```rust
    /// The roll of a six-sided die: 1 to 6.
    pub fn die(&mut self) -> u8 {
        self.below(6) as u8 + 1
    }
```

The house rule is that every random choice goes through `Rng`, so one seed replays a whole bout.

## A table of contests

[`src/mash.rs`](../src/mash.rs) describes the four contests as data:

```rust
type Contest = (&'static str, fn(&Stats) -> u8, &'static str);

const CONTESTS: [Contest; 4] = [
    (
        "Strength",
        |s| s.strength,
        "{A} and {B} locked arms on the frozen pond. {W} hurled {L} clean into a snowdrift!",
    ),
    // ...
];
```

`type Contest` is an alias: a short name for a tuple of three things. The middle one, `fn(&Stats) -> u8`, is a function pointer: "any function that takes a stat card and returns a number". Each entry supplies one as a closure, `|s| s.strength`. A closure that captures nothing can be used where a plain function is expected, which is what lets it sit in a `const`. The table pairs each stat's name with how to read it and how to tell it, so the loop below has no `match` on the stat and a fifth contest would be one more entry.

## The fight

```rust
pub fn fight<'a>(rng: &mut Rng, a: &'a Creature, b: &'a Creature) -> Bout<'a> {
    let mut order = [0, 1, 2, 3];
    rng.shuffle(&mut order);
    // ...
    for (n, &contest) in order.iter().take(3).enumerate() {
        let (stat, value, line) = CONTESTS[contest];
        // Stats are 1 to 10, but only the data tests say so: a pack could
        // claim 255, which must neither overflow nor win by default.
        let (stat_a, stat_b) = (value(&a.stats).min(10), value(&b.stats).min(10));
        let mut roll = || (stat_a + rng.die(), stat_b + rng.die());
        let (mut roll_a, mut roll_b) = roll();
        // Re-roll a tie a few times; if it won't break, the better stat wins.
        for _ in 0..5 {
            if roll_a != roll_b {
                break;
            }
            (roll_a, roll_b) = roll();
        }
        let winner = u8::from(roll_b > roll_a || (roll_b == roll_a && stat_b > stat_a));
        // ...
    }
```

Shuffling `[0, 1, 2, 3]` and taking three gives three different contests in a random order, with no "have we had this one?" check.

`.min(10)` is there because a `u8` holds up to 255 and nothing in the type says a stat stops at 10. The loader turns away a community creature with a stat over 10 (Night 22), but `fight` is a public function and does not lean on that. Without the cap, `255 + rng.die()` would overflow: a panic in a debug build, a small wrapped number in a release build. With it, the largest roll is 16.

`roll` is a closure that borrows `rng` mutably, so it is declared `let mut`: calling it changes something it captured. A tie is re-rolled up to five times. If the rolls still match, the better stat takes it, and if the stats match too, the round goes to the first creature. A loop that re-rolled "until different" would almost always stop, and "almost always" is a poor property for a loop. `u8::from(true)` is 1 and `u8::from(false)` is 0, so the long condition yields the winner's side directly.

After three rounds:

```rust
    let wins_b = bout.rounds.iter().filter(|r| r.winner == 1).count();
    bout.winner = u8::from(wins_b * 2 > bout.rounds.len());
```

More than half the rounds, written without division.

## One Krampus, thirteen Yule Lads

Now the writing, which holds tonight's most useful lesson. The comment above `CONTESTS` reads:

```rust
/// A stat to contest and how the round is told: {A} and {B} are the
/// creatures, {W} the round's winner and {L} its loser. The lines are in the
/// past tense so they read the same for one Krampus or thirteen Yule Lads.
```

Try the strength line in the present tense. "Krampus hurls the Tomte into a snowdrift" is fine. "The Yule Lads hurls the Tomte" is not: a plural subject wants "hurl". In English the present tense changes with the number of the subject and the past tense does not: "Krampus hurled", "the Yule Lads hurled" (only "was" and "were" differ, and the lines avoid them). The program could carry a `plural` flag on every creature and two forms of every verb. Writing in the past tense means the data needs no flag, a community pack cannot get it wrong, and the template needs no logic. The lines also avoid "he", "she", "it" and "they" by using names.

The same care shows in two more places in `fight`:

```rust
            // No possessive: "The Yule Lads's" would be wrong.
            signature: format!("Signature move from {}: {power}!", mid_sentence(&w.name)),
```

and the loser's exit, which quotes the weakness instead of weaving it into a sentence:

```rust
    // Quoted, because weaknesses are written every which way: "Sticks",
    // "Twelfth Night", "Can only warn, not save".
    bout.finale = format!(
        "{} trudged off into the snow, muttering: “{}...”",
        loser.name,
        weakness.trim_end_matches('.')
    );
```

When text is assembled from data you did not write, choose sentence shapes that survive any value.

Names need one more touch. "The Yule Cat" begins with a capital at the start of a sentence, and reads "the Yule Cat" in the middle of one. `fill` puts the names into a line and decides which:

```rust
fn fill(line: &str, a: &Creature, b: &Creature, w: &Creature, l: &Creature) -> String {
    let mut out = String::new();
    let mut rest = line;
    while let Some(at) = rest.find('{') {
        out.push_str(&rest[..at]);
        let name = match &rest[at..at + 3] {
            "{A}" => &a.name,
            "{B}" => &b.name,
            "{W}" => &w.name,
            _ => &l.name,
        };
        let starts_sentence =
            out.trim_end().is_empty() || out.trim_end().ends_with(['.', '!', '?']);
        if starts_sentence {
            out.push_str(name);
        } else {
            out.push_str(&mid_sentence(name));
        }
        rest = &rest[at + 3..];
    }
    out + rest
}
```

`while let` loops for as long as the pattern matches: here, while there is another `{`. `rest` is a `&str` that shrinks from the front, a view onto the same template, with nothing copied. A name starts a sentence if nothing has been written yet, or if what has been written ends in a full stop, an exclamation mark or a question mark.

## Telling it slowly

`fight` returns a finished `Bout`: three `Round`s, a winner and a finale. Nothing has been printed. `Show::mash` in [`src/main.rs`](../src/main.rs) does the telling:

```rust
        let pause = |ms: u64| {
            if !fast && io::stdout().is_terminal() {
                std::thread::sleep(std::time::Duration::from_millis(ms));
            }
        };
        print(&render::mash_header(&self.painter, &bout));
        pause(1200);
        for round in &bout.rounds {
            print(&render::mash_round(&self.painter, &bout, round));
            pause(1500);
        }
        print(&render::mash_result(&self.painter, &bout));
        self.record(&Outcome::new(Kind::Mash, calendar::now()));
```

The suspense is theatre. The result was settled before the first line appeared, which is what makes `--json` simple: the same `Bout` is serialised and returned a few lines earlier, with no pauses to skip. `pause` sleeps only if `--fast` was not given and the output is a terminal. A script piping the output has no use for drama, and an end-to-end test, `piped_mash_is_plain_and_does_not_pause`, runs a showdown through a pipe and fails if it takes three seconds.

One detail in `Round`: the signature move is a field called `signature`, written to JSON as `"move"` with `#[serde(rename = "move")]`, because `move` is a Rust keyword.

## Two hundred bouts

One seed proves little about dice. This test in `mash.rs` stages two hundred:

```rust
        for seed in 0..200 {
            let bout = fight(&mut Rng::new(seed), a, b);
            assert_eq!(bout.rounds.len(), 3);
            // ...
            let (won, lost) = bout.score();
            assert!(won > lost && won + lost == 3);
            for r in &bout.rounds {
                let higher = u8::from(r.roll_b > r.roll_a);
                assert!(r.roll_a == r.roll_b || r.winner == higher, "{r:?}");
                // ...
                assert!(!r.narration.contains('{'), "{}", r.narration);
            }
        }
```

It never says who should win. It says what must be true whoever wins: three rounds on three different stats, the higher roll takes the round, the bout's winner took more rounds, and no `{W}` was left unfilled. A bug in a random process usually shows on some seed, and two hundred take a moment.

## Run it

```bash
TERMINAL_OF_TINSEL_SEED=25 cargo run -- mash krampus yule-cat --fast --no-color
```

With seed 25, after the two stat cards:

```
Round 1 · Strength
Krampus rolled 10 · The Yule Cat rolled 13
Krampus and the Yule Cat locked arms on the frozen pond. The Yule Cat hurled
Krampus clean into a snowdrift!
Signature move from the Yule Cat: Spotting old clothes at a glance!

Round 2 · Dread
Krampus rolled 13 · The Yule Cat rolled 11
A staring contest over the punch bowl. The Yule Cat blinked first, and Krampus
let out a laugh that rattled the baubles.
Signature move from Krampus: A sack big enough for a child!
```

The Yule Cat took the third round and the bout, 2 to 1, and "Krampus trudged off into the snow, muttering: “Genuinely good children...”". And with seed 7, `mash "yule lads" tomte --fast` opens:

```
Round 1 · Speed
The Yule Lads rolled 11 · The Tomte rolled 10
A chase across the rooftops! The Yule Lads vanished between the chimneys, and
the Tomte gave up, puffing, by the weathervane.
Signature move from the Yule Lads: Thirteen nights of mischief!
```

Thirteen brothers, and not a verb out of place.

## Try it

- Run `cargo run -- mash` with no names, then with one. The empty corners are filled from the creatures not already chosen.
- Run `cargo run -- mash krampus krampus` and read the error and its hint.
- Run a showdown without `--fast`, then the same command with `| cat` on the end. Only the first one waits.
- Rewrite one narration line in the present tense and stage the Yule Lads against anyone.

## 🎄 Festive fact

A closure that captures no variables can be converted to a plain function pointer, which is how `|s| s.strength` fits a field of type `fn(&Stats) -> u8`. That conversion was added in Rust 1.19, in 2017. A closure that does capture something, like tonight's `roll` with its borrowed `rng`, cannot be converted: it is more than a function, since it carries data.

## 🕯️ Before morning

Write a test that a creature claiming a strength of 255 never rolls above 16. Hint: `Creature` derives `Clone`, so take one from `Vault::load()`, clone it, set `stats.strength`, `speed`, `cunning` and `dread` to 255, and loop over seeds as the test above does, asserting `r.roll_a <= 16` for every round.

> 📺 *"The pond has been swept and the loser has been given a mince pie. You've now played everything we have, and tomorrow night we find out what you have to show for it. Somebody has been keeping a stocking with your name on."*

[← Night 18](night-18.md) · [Index](README.md) · [Night 20 →](night-20.md)
