# Night 4 · A Fact from the Cracker

> 📺 *"Good evening. Every cracker holds a paper hat, a joke and something small that breaks by Boxing Day. Ours will hold a fact. Tonight we build the machine that chooses which one, and I can promise it's fair, because we're going to take it apart first."*

**Tonight you'll learn**
- `u64` and why arithmetic has to say what happens on overflow
- Writing a small random number generator, SplitMix64, by hand
- `&mut self`: a method that changes the thing it's called on
- `Option` properly: `Some`, `None`, and the methods that go with them
- Seeding from an environment variable or the clock, so a run can be repeated
- A `random` command

**Where we are:** three hard-coded creatures and a `list` command.

## Why write our own?

Rust's standard library has no stable way to make random numbers. Most programs add the `rand` crate. We won't, for two reasons. The first is that the generator we need is ten lines long, and writing it teaches integers, mutation and `Option` in one go. The second is the real one: later in the course the program shows a Fact of the Night that's the same for everybody all day. That only works if the numbers never change between releases, and the way to be certain is to own the code that makes them.

## The generator

A random number generator is a number (the **state**) and a recipe for scrambling it. Each time you ask, it moves the state on a step and hands you a scrambled copy. The same starting state, called the **seed**, always gives the same sequence.

Add a line at the very top of `src/main.rs`:

```rust
use std::time::{SystemTime, UNIX_EPOCH};
```

Then add this after the `Command` enum:

```rust
/// Fixes every random choice, so a run can be repeated exactly.
const SEED_ENV: &str = "TERMINAL_OF_TINSEL_SEED";

/// SplitMix64: fast, tiny, and good enough for a Christmas special.
#[derive(Debug, Clone)]
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// A seed from `TERMINAL_OF_TINSEL_SEED` if set, otherwise from the clock.
    fn from_env_or_clock() -> Self {
        let from_env: Option<u64> = std::env::var(SEED_ENV)
            .ok()
            .and_then(|text| text.trim().parse().ok());
        let seed = match from_env {
            Some(seed) => seed,
            None => SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0),
        };
        Self::new(seed)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A number in `0..n`. Panics if `n` is zero.
    fn below(&mut self, n: usize) -> usize {
        assert!(n > 0, "Rng::below(0)");
        (self.next_u64() % n as u64) as usize
    }
}
```

**`struct Rng(u64);`** is a *tuple struct*: a struct whose fields have positions instead of names. It has one field, a `u64`, reached as `self.0`. Wrapping a number in a struct gives it a type of its own, so nobody can mistake a generator for a count.

**`u64`** is an unsigned 64-bit integer: a whole number from 0 to about 18 quintillion. Rust's integer types say their size and sign in their names: `u8`, `i32`, `u64`, and `usize`, which is whatever size the machine uses to index memory. Lists are indexed and counted with `usize`.

**`next_u64`** takes `&mut self`, a *mutable borrow*: it may change the generator, and it must, since the state has to move on. The recipe is SplitMix64. It adds a large odd constant to the state, then mixes the bits with three rounds of "shift right, XOR, multiply" (`>>` shifts bits, `^` is XOR). The constants are the published ones; the underscores in `0x9E37_79B9_7F4A_7C15` are only there to make the hexadecimal readable.

Adding a huge constant to a `u64` will soon pass the largest value it can hold. Rust won't guess what you meant: in a debug build, a plain `+` that overflows stops the program with a panic. `wrapping_add` and `wrapping_mul` say "wrap round to zero and carry on", which is exactly what the recipe wants.

**`below`** turns a huge number into one from 0 to `n - 1` with `%`, the remainder. `as` converts between number types: `n as u64` widens `n` for the sum, and `as usize` turns the answer back into something that can index a list. `assert!` stops the program if its condition is false: asking for a number below zero is a mistake in the code, not something to carry on from.

`let mut z` is the first `mut` on a variable. Variables can't be changed unless you say so.

## Option, properly

Look at `from_env_or_clock`:

```rust
        let from_env: Option<u64> = std::env::var(SEED_ENV)
            .ok()
            .and_then(|text| text.trim().parse().ok());
        let seed = match from_env {
            Some(seed) => seed,
            None => SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0),
        };
        Self::new(seed)
```

On Night 2 `Option` let a subcommand be absent. Here it's a seed that may be absent. Read the first statement as a pipeline:

- `std::env::var(SEED_ENV)` asks for the environment variable. It can fail (the variable may not be set), so it returns a `Result`, which is Night 7's subject.
- `.ok()` turns that into an `Option<String>`: `Some(text)` if it was set, `None` if not.
- `.and_then(...)` runs only if there is a value, and its answer is itself an `Option`. Inside it, `|text| text.trim().parse().ok()` is a **closure**: a small function written in place, with its parameter between the bars. It tries to read the text as a number and gives `None` if it isn't one.

How does `parse()` know to produce a `u64`? Because we wrote `Option<u64>` on the left. Rust works out the types in between.

Then `match` handles both cases. `Some(seed) => seed` takes the number out. `None` reads the clock instead: the time since 1970 in nanoseconds, cut down to a `u64`. Even the clock can fail (it may be set before 1970), so `unwrap_or(0)` gives a fallback.

There's no way to use the seed without saying what happens when there isn't one. That's `Option` earning its keep.

## The random command

Add a variant to `Command`:

```rust
enum Command {
    /// List every creature
    List,

    /// A random creature fact
    Random,
}
```

An arm to the `match` in `main`:

```rust
fn main() {
    let cli = Cli::parse();
    let guests = creatures();
    match cli.command {
        None => welcome(),
        Some(Command::List) => list(&guests),
        Some(Command::Random) => random(&guests),
    }
}
```

And two functions at the end of the file:

```rust
/// A random fact from a random creature, if there is one to tell.
fn random_fact(creatures: &[Creature], rng: &mut Rng) -> Option<(usize, usize)> {
    if creatures.is_empty() {
        return None;
    }
    let who = rng.below(creatures.len());
    if creatures[who].facts.is_empty() {
        return None;
    }
    Some((who, rng.below(creatures[who].facts.len())))
}

fn random(creatures: &[Creature]) {
    let mut rng = Rng::from_env_or_clock();
    match random_fact(creatures, &mut rng) {
        Some((who, which)) => {
            let c = &creatures[who];
            println!("🎄 RANDOM FESTIVE FACT");
            println!();
            println!("{} {}", c.emoji, c.name);
            println!("{}", c.facts[which]);
        }
        None => println!("No facts tonight. The crackers were empty."),
    }
}
```

`random_fact` chooses *positions*: which creature, and which of its facts. It returns `Option<(usize, usize)>`. The parentheses make a **tuple**, two values travelling together. If there are no creatures, or the chosen one has no facts, there's nothing to choose, so it returns `None` early with `return`.

It takes `rng: &mut Rng` because choosing changes the generator. The caller has to agree in writing, with `&mut rng`, and has to have declared `let mut rng`. You can always see at the call site what a function is allowed to change.

In `random`, the pattern `Some((who, which))` unpacks the option and the tuple in one go. `creatures[who]` indexes the list, and `&creatures[who]` borrows that creature instead of trying to move it out.

## Run it

```bash
cargo run -q -- random
```

```
🎄 RANDOM FESTIVE FACT

🐈 The Yule Cat
Farm workers who finished the autumn wool were rewarded with new clothes
```

Yours will differ, and will differ again each time, because the seed is the clock. Now fix the seed:

```bash
TERMINAL_OF_TINSEL_SEED=25 cargo run -q -- random
```

```
🎄 RANDOM FESTIVE FACT

😈 Krampus
Austrians sent each other Krampus greeting cards in the 1800s
```

Run that as often as you like: with the seed set to 25 it's always this fact, on your machine and mine. (In PowerShell, set it first with `$env:TERMINAL_OF_TINSEL_SEED = "25"`, and remove it afterwards with `Remove-Item Env:TERMINAL_OF_TINSEL_SEED`.) From now on, whenever a lesson shows output that depends on chance, it sets the seed, so yours can match.

```bash
git add .
git commit -m "feat: a seeded random fact"
```

## Try it

- Try seeds 1 to 5 and see which creatures come up.
- Set the seed to `tinsel`. It isn't a number, so `parse` gives `None` and the clock is used. The finished program prints a warning here; you'll see how on Night 7.
- Change `wrapping_add` to a plain `+` and run it a few times. In a debug build it panics with "attempt to add with overflow" as soon as the state passes the top. Put it back.
- Print the first three numbers from `Rng::new(0)`. The first should be 16294208416658607535, which is `0xE220_A839_7B1D_CDAF`. On Night 8 a test will pin that.

## 🎄 Festive fact

SplitMix64 comes from a 2014 paper by Guy Steele, Doug Lea and Christine Flood, and it's the generator behind Java's `SplittableRandom`. The constant it adds each step, `0x9E3779B97F4A7C15`, is 2 to the power 64 divided by the golden ratio, rounded to an odd number: adding it over and over visits every one of the 2^64 states before repeating. The finished generator is in [`src/rng.rs`](../src/rng.rs), with a shuffle and a die we'll need for the games.

## 🕯️ Before morning

Add a method `die` to `Rng` that rolls a six-sided die, returning 1 to 6 as a `u8`, and print three rolls at the end of `random`.

Hint: `self.below(6)` gives 0 to 5. Convert with `as u8` and add one. The finished program has exactly this method, and uses it on Night 19 for the Snowball Showdown. Take the printing out again afterwards; you can leave `die` in, though the compiler will warn that nothing uses it yet.

> 📺 *"A fair machine, and it chose the cat. It usually does. Tomorrow the guests stop living in the source code and move somewhere with more room: under the tree, in JSON."*

[← Night 3](night-03.md) · [Index](README.md) · [Night 5 →](night-05.md)
