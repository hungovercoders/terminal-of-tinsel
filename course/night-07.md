# Night 7 · Speak Its Name

> 📺 *"Good evening. The switchboard is busy tonight. One viewer wants 'the cat one', another has asked for Grýla without the accent, and a gentleman in Leeds has typed Krampus with the letters in a new and exciting order. A good host knows who they mean. A good programme says so politely when it doesn't."*

**Tonight you'll learn**
- `Result` and the `?` operator: errors as values
- An error type of our own, `TinselError`, with stable codes
- Implementing a trait by hand: `Display`
- Finding a creature by id, name, alias or part of one, with accents folded
- Edit distance, for "did you mean"
- Exit codes, and a `creature <name>` command

**Where we are:** 22 creatures in four packs, with `list` and `random`. Nothing can be looked up by name.

## Errors are values

Rust has no exceptions. A function that can fail returns a **`Result`**, which is an enum like `Option`:

- `Ok(value)` when it worked
- `Err(error)` when it didn't

You've met three already (`env::var`, `parse`, `from_str`) and dodged each with `.ok()` or `.expect()`. Tonight we make our own.

Create `src/error.rs`:

```rust
//! Errors with stable codes, so a script can branch on them.

use std::fmt;

/// The error codes a script can match on. They are stable; the messages are not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Code {
    UnknownCreature,
    AmbiguousCreature,
    Usage,
}

impl Code {
    /// What the program exits with: 2 for a misused command, as clap does,
    /// and 1 for everything else.
    pub fn exit_code(self) -> u8 {
        match self {
            Code::Usage => 2,
            Code::UnknownCreature | Code::AmbiguousCreature => 1,
        }
    }
}

/// An error a player can act on: what went wrong and, sometimes, what to try.
#[derive(Debug, Clone)]
pub struct TinselError {
    pub code: Code,
    pub message: String,
    pub hint: Option<String>,
}

impl TinselError {
    pub fn new(code: Code, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            hint: None,
        }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

impl fmt::Display for TinselError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.hint {
            Some(hint) => write!(f, "{}; {}", self.message, hint),
            None => f.write_str(&self.message),
        }
    }
}

impl std::error::Error for TinselError {}

pub type Result<T> = std::result::Result<T, TinselError>;
```

**`Code`** is an enum of the things that can go wrong. A script can't be expected to read our prose, and we'd like to be free to reword it, so each failure gets a code that will never change. The finished program has eight; we need three.

**`TinselError`** carries a code, a message, and sometimes a hint about what to try.

**`impl Into<String>`** in `new` means "anything that can be turned into a `String`". Callers may pass a `&str` literal or a `String` from `format!`, and `.into()` does the conversion. `with_hint` takes `mut self` and hands itself back, the same chaining trick as `fact` on Night 3.

**`impl fmt::Display for TinselError`** is the first trait we've implemented by hand instead of deriving. `Display` is the ability to be printed with `{}`, and it can't be derived, because only we know how our error should read. The trait requires one method, `fmt`, and `write!` is `println!` aimed at a destination. `impl std::error::Error` then marks the type as a proper error; it needs no methods of its own.

**`pub type Result<T> = ...`** is an alias. Every fallible function here fails with a `TinselError`, so we write `Result<&Creature>` instead of spelling out both halves each time.

## Finding a creature

In `src/creatures.rs`, add an import under the serde one:

```rust
use crate::error::{Code, Result, TinselError};
```

Add one field to `Creature`, after `origin`:

```rust
    pub legend: String,
```

Now two methods. They go inside `impl Vault`, after `by_pack`:

```rust
    /// Finds a creature by id, name, alias or part of one.
    pub fn find(&self, query: &str) -> Result<&Creature> {
        let key = normalize(query);
        if key.is_empty() {
            return Err(unknown(query, &[]));
        }
        let names = |c: &Creature| {
            let mut v = vec![normalize(&c.id), normalize(&c.name)];
            v.extend(c.aliases.iter().map(|a| normalize(a)));
            v
        };
        // An exact match on any name wins outright.
        if let Some(c) = self.creatures.iter().find(|c| names(c).contains(&key)) {
            return Ok(c);
        }
        // Otherwise, every word of the query must appear in one of its names,
        // so "ghost future" finds the Ghost of Christmas Future.
        let words: Vec<&str> = key.split(' ').collect();
        let matches: Vec<&Creature> = self
            .creatures
            .iter()
            .filter(|c| names(c).iter().any(|n| words.iter().all(|w| n.contains(w))))
            .collect();
        match matches.as_slice() {
            [one] => Ok(one),
            [] => Err(unknown(query, &self.suggest(&key))),
            many => {
                let names: Vec<&str> = many.iter().map(|c| c.name.as_str()).collect();
                Err(TinselError::new(
                    Code::AmbiguousCreature,
                    format!("{query:?} could be any of: {}", names.join(", ")),
                ))
            }
        }
    }

    /// Up to three creature names within a few edits of a query that matched
    /// nothing, nearest first.
    fn suggest(&self, key: &str) -> Vec<&str> {
        let mut scored: Vec<(usize, &str)> = self
            .creatures
            .iter()
            .filter_map(|c| {
                std::iter::once(&c.id)
                    .chain(std::iter::once(&c.name))
                    .chain(&c.aliases)
                    .map(|n| levenshtein(&normalize(n), key))
                    .min()
                    .map(|d| (d, c.name.as_str()))
            })
            .filter(|(d, _)| *d <= 3.max(key.len() / 3))
            .collect();
        scored.sort_by_key(|(d, _)| *d);
        scored.into_iter().take(3).map(|(_, name)| name).collect()
    }
```

`find` returns `Result<&Creature>`: a borrowed creature, or an error saying why not. It works in three steps.

First, **normalise** the query (below), so that case, accents and punctuation don't matter. `names` is a closure that lists everything a creature answers to, normalised the same way: its id, its name and its aliases.

Second, an **exact match** on any of those wins at once. `if let Some(c) = ...` is a one-armed `match`: it runs the block only when the pattern fits.

Third, a **partial match**: every word of the query must appear in one of the creature's names. `any` and `all` are iterator methods that answer yes or no. Then a `match` on a slice sorts out what we found: `[one]` fits a list of exactly one item, `[]` an empty list, and a bare name such as `many` fits anything else.

`suggest` is only reached when nothing matched. It scores every creature by the smallest edit distance from the query to any of its names, keeps those within three edits (more for a long query) and takes the nearest three.

The helpers go at the end of the file, outside the `impl`:

```rust
fn unknown(query: &str, suggestions: &[&str]) -> TinselError {
    let mut msg = format!("no creature called {query:?} is out tonight");
    match suggestions {
        [] => {}
        [one] => msg.push_str(&format!(". Did you mean {one}?")),
        [first @ .., last] => {
            msg.push_str(&format!(". Did you mean {} or {last}?", first.join(", ")));
        }
    }
    TinselError::new(Code::UnknownCreature, msg)
        .with_hint("try 'terminal-of-tinsel list' for tonight's guests")
}

/// Lowercases, folds common accents, drops a leading "the" and punctuation,
/// so "the Yule-Cat" and "yule cat" match, and so do "Grýla" and "gryla".
pub fn normalize(s: &str) -> String {
    let folded: String = s.trim().chars().map(fold).collect();
    let folded = folded.strip_prefix("the ").unwrap_or(&folded);
    let cleaned: String = folded
        .chars()
        .filter_map(|ch| match ch {
            'a'..='z' | '0'..='9' => Some(ch),
            ' ' | '-' | '_' | '\'' => Some(' '),
            _ => None,
        })
        .collect();
    cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Lowercases one character and folds common accents.
pub fn fold(ch: char) -> char {
    match ch.to_lowercase().next().unwrap_or(ch) {
        'á' | 'à' | 'â' | 'ä' => 'a',
        'é' | 'è' | 'ê' | 'ë' => 'e',
        'í' | 'ì' | 'î' | 'ï' => 'i',
        'ý' | 'ÿ' => 'y',
        'ó' | 'ò' | 'ô' | 'ö' | 'ø' => 'o',
        'ú' | 'ù' | 'û' | 'ü' => 'u',
        'ð' => 'd',
        'þ' => 't',
        'æ' => 'a',
        'č' => 'c',
        c => c,
    }
}

/// Edit distance, for "did you mean".
fn levenshtein(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut cur = vec![i + 1; b.len() + 1];
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != *cb);
            cur[j + 1] = (prev[j] + cost).min(prev[j + 1] + 1).min(cur[j] + 1);
        }
        prev = cur;
    }
    prev[b.len()]
}
```

**`normalize`** lowercases and folds accents with `fold`, drops a leading "the", keeps letters and digits, and turns separators into single spaces. So "The Yule-Cat" becomes "yule cat", and "Grýla" becomes "gryla". Its `match` uses ranges (`'a'..='z'`) and alternatives (`|`) as patterns.

**`fold`** is a plain table of the accents that appear in our folklore, such as Icelandic `ð` and `þ` and the Czech `č` in Čert.

**`levenshtein`** counts the fewest single-letter insertions, deletions and substitutions that turn one string into another. "krampsu" is two edits from "krampus".

**`unknown`** builds the error. Its slice patterns include `[first @ .., last]`: "at least one item; call the last one `last` and the rest `first`". That's how three suggestions become "A, B or C". `{query:?}` prints the query in quotes.

## The command

In `src/cli.rs`, add a variant to `Command`, between `List` and `Random`:

```rust
    /// Read one creature's page
    Creature {
        /// The creature's name, nickname or part of one, e.g. krampus, yule cat, nisse
        #[arg(num_args = 1.., required = true, value_name = "NAME")]
        name: Vec<String>,
    },
```

This variant has a field. `Vec<String>` with `num_args = 1..` collects one or more words, so `creature yule cat` works without quotes.

In `src/main.rs`, add `mod error;` to the `mod` lines (after `mod creatures;`) and change the imports to:

```rust
use std::process::ExitCode;

use clap::Parser;

use cli::{Cli, Command};
use creatures::{Creature, Vault};
use error::{Code, Result, TinselError};
use rng::Rng;
```

Replace `main` with these two functions:

```rust
fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {}", e.message);
            if let Some(hint) = &e.hint {
                eprintln!("hint: {hint}");
            }
            ExitCode::from(e.code.exit_code())
        }
    }
}

fn run(cli: Cli) -> Result<()> {
    let vault = Vault::load();
    match cli.command {
        None => welcome(),
        Some(Command::List) => list(&vault),
        Some(Command::Creature { name }) => creature(&vault, &name.join(" "))?,
        Some(Command::Random) => random(&vault)?,
    }
    Ok(())
}
```

`run` does the work and returns `Result<()>`, where `()` is the empty value: "it worked, and there's nothing to hand back". `main` is the only place that prints an error. Errors go to **stderr** with `eprintln!`, so that a script reading the program's output never mistakes a complaint for data.

Then `main` returns an **exit code**. A misused command exits with 2, as clap's own errors do, and our other errors with 1. (The finished program is plainer still: it exits with 1 for every error of its own.)

Now the `?` at the end of two arms. It means: if this is an `Err`, return it from the current function straight away; otherwise carry on with the value inside. It's the whole of error handling in most Rust code, and it only works in a function that itself returns a `Result`.

Last, replace `random` and add `creature`, at the end of the file:

```rust
fn random(vault: &Vault) -> Result<()> {
    let mut rng = Rng::from_env_or_clock();
    let (who, which) = random_fact(&vault.creatures, &mut rng)
        .ok_or_else(|| TinselError::new(Code::Usage, "no facts to tell tonight"))?;
    let c = &vault.creatures[who];
    println!("🎄 RANDOM FESTIVE FACT");
    println!();
    println!("{} {}", c.emoji, c.name);
    println!("{}", c.facts[which]);
    println!();
    println!("📺 {}: “{}”", host::NAME, host::intro(c));
    Ok(())
}

fn creature(vault: &Vault, name: &str) -> Result<()> {
    let c = vault.find(name)?;
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
```

In `creature`, `vault.find(name)?` either gives us the creature or leaves the function with the error. In `random`, `ok_or_else` turns an `Option` into a `Result` by supplying the error for the `None` case, and the `let (who, which) = ...` pattern unpacks the tuple.

## Run it

```bash
cargo run -q -- creature yule cat
```

```
🐈 THE YULE CAT
The giant Icelandic cat who eats anyone without new clothes by Christmas Eve
Icelandic folklore · pack: nordic

📺 Auntie Aerial: “Check your jumper, viewers. If it's last year's, I'd stay away from the windows tonight.”
...
```

All of these find their creature: `creature Grýla`, `creature gryla`, `creature nisse` (an alias of the Tomte), `creature ghost future`. And when it can't:

```bash
cargo run -q -- creature krampsu
```

```
error: no creature called "krampsu" is out tonight. Did you mean Krampus?
hint: try 'terminal-of-tinsel list' for tonight's guests
```

Afterwards `echo $?` prints `1`. Try `creature ghost` too: four guests answer to that, and the error names them all.

```bash
git add .
git commit -m "feat: find a creature by name, with errors and exit codes"
```

## Try it

- Run `cargo run -q -- creature krampsu > out.txt`. The error still appears on screen and `out.txt` is empty: stdout was redirected, stderr was not. Delete the file.
- Remove the `?` after `creature(&vault, &name.join(" "))`. The compiler complains that the `match` arms have different types: `()` from the others, a `Result` from this one. Put it back.
- Add a fourth variant to `Code` and build. The `match` in `exit_code` no longer covers everything, and the compiler says so.

## 🎄 Festive fact

The `?` operator arrived in Rust 1.13, in November 2016. Before that the same job was done by a macro, written `try!(expression)`, and old code is full of it. When the 2018 edition reserved `try` as a keyword for future use, the macro had to be called by its raw name, `r#try!`, which was a strong hint to stop using it.

## 🕯️ Before morning

Run `creature the`. The program says "the" could be any of three guests, and none of them is The Yule Cat, The Tomte or anyone else whose name starts with "The". Work out why those three, by reading `normalize` and `find` together.

Hint: `normalize` removes a leading "the " from every name before anything is compared, and the partial match looks inside aliases as well as names. Run `list` and read the aliases of the three it named.

> 📺 *"The gentleman in Leeds has been put through to Krampus, and good luck to him. Tomorrow we do what a certain bearded colleague does with his list: we check it, and then we check it again."*

[← Night 6](night-06.md) · [Index](README.md) · [Night 8 →](night-08.md)
