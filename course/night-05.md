# Night 5 · Data under the Tree

> 📺 *"Good evening. Our three guests have been living inside the source code, which is cramped, and every time one of them remembers a new fact somebody has to recompile the building. Tonight they move out into files of their own. Wrapped, labelled and left under the tree."*

**Tonight you'll learn**
- Why data belongs in files, and what JSON looks like
- serde and serde_json: turning text into structs
- Derive macros, and what `#[derive(Deserialize)]` writes for you
- `#[serde(default)]` and `rename_all = "camelCase"`
- `include_str!`: compiling a file into the program

**Where we are:** three creatures typed into `main.rs`, with `list` and `random` commands.

## Fetch the data

The finished program keeps each creature in a JSON file. Rather than type folklore for an hour, borrow the files. Clone the finished repository *beside* your project, under another name, and copy three files across:

```bash
cd ..
git clone https://github.com/hungovercoders/terminal-of-tinsel finished
cd terminal-of-tinsel
mkdir -p packs/companions packs/nordic packs/midwinter
cp ../finished/packs/companions/krampus.json packs/companions/
cp ../finished/packs/nordic/yule-cat.json packs/nordic/
cp ../finished/packs/midwinter/mari-lwyd.json packs/midwinter/
```

On Windows, make the three folders and copy the files with File Explorer if you prefer. You should end up with a `packs` folder next to `src`.

Open `packs/companions/krampus.json`. Here is the start of it:

```json
{
  "id": "krampus",
  "name": "Krampus",
  "aliases": ["The Krampus", "Krampusz"],
  "emoji": "😈",
  "description": "The horned, chain-rattling demon who deals with the naughty while St Nicholas rewards the good",
  "origin": "Alpine folklore of Austria, Bavaria and their neighbours",
```

JSON is text in a fixed shape: objects in braces with named fields, lists in square brackets, strings in double quotes. The file has more than our struct knows about: a legend, myths, stats, colours. That's fine, and you'll see why shortly.

## Add serde

```bash
cargo add serde@1.0.229 --features derive
cargo add serde_json@1.0.154
```

**serde** is the framework: it knows how to walk a Rust type. **serde_json** is one format for it. The split means the same struct could be read from TOML or YAML by swapping the second crate.

## The struct learns to read

In `src/main.rs`, add `use serde::Deserialize;` under the clap import. Then delete everything from the `Creature` struct down to the end of the `creatures` function: the struct, its `impl` block and the hand-typed list. Three blocks go in their place. Here is the first, and the other two follow below:

```rust
/// A single creature and what we know about it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct Creature {
    id: String,
    name: String,
    aliases: Vec<String>,
    emoji: String,
    description: String,
    origin: String,
    facts: Vec<String>,
    host_intro: String,
}

impl Creature {
    /// The creature as one line of the guest list.
    fn line(&self) -> String {
        format!("  {} {:<14} {}", self.emoji, self.name, self.origin)
    }
}
```

`new` and `fact` have gone: serde will build the creatures now. `line` stays.

The top of the file now reads:

```rust
use std::time::{SystemTime, UNIX_EPOCH};

use clap::{Parser, Subcommand};
use serde::Deserialize;
```

## What a derive is

You've been writing `#[derive(Debug)]` since Night 2. Time to say what it does.

A **trait** is a set of abilities a type can have. `Debug` is the ability to be printed with `{:?}`. `Clone` is the ability to be copied with `.clone()`. `Default` is the ability to make an empty value: an empty string, an empty list, zero. `Deserialize` is serde's: the ability to be built from data.

To give a type an ability you write an `impl` of the trait for it, which for a struct of eight fields is a page of dull, predictable code. A **derive macro** writes that page for you, at compile time, by looking at the struct's fields. `#[derive(Debug, Clone, Default, Deserialize)]` is four pages you didn't type. Nothing is looked up while the program runs: the generated code is as fast as if you'd written it by hand.

Derives only work if every field has the ability too. `String` and `Vec<String>` can already do all four, so `Creature` can.

## The serde attributes

```rust
#[serde(rename_all = "camelCase", default)]
```

This line configures the `Deserialize` derive.

**`rename_all = "camelCase"`** deals with a difference in manners. Rust fields are written `host_intro`. JSON fields are usually written `hostIntro`. With this attribute serde translates, so the struct stays Rust and the file stays JSON.

**`default`** says: if a field is missing from the JSON, don't fail, use the type's `Default` value. That's why `Default` is in the derive list. Tonight every file has every field, but on Night 22 people will write their own creatures, and a creature three lines long should still load.

And the fields the struct *doesn't* have? serde ignores JSON fields it has no place for. So we can grow `Creature` a field at a time as we need them, which is how the next five nights go.

## Compiling the files in

The second block:

```rust
/// The built-in creatures, as JSON text compiled into the program.
const BUILT_IN: [&str; 3] = [
    include_str!("../packs/companions/krampus.json"),
    include_str!("../packs/nordic/yule-cat.json"),
    include_str!("../packs/midwinter/mari-lwyd.json"),
];
```

`include_str!` is a macro that reads a file *while compiling* and puts its contents into the program as a `&str`. The path is relative to the source file, so from `src/main.rs` it starts with `../`. After that the JSON files aren't needed to run the program: they're inside it. Hand somebody the binary alone and it still knows Krampus.

`[&str; 3]` is an **array**: a list whose length, 3, is part of its type and fixed at compile time. Change a file and Cargo rebuilds; mistype a path and it won't compile at all.

And the third:

```rust
/// Tonight's guests, read from the JSON.
fn creatures() -> Vec<Creature> {
    let mut all = Vec::new();
    for text in BUILT_IN {
        let c: Creature = serde_json::from_str(text).expect("invalid creature JSON");
        all.push(c);
    }
    all
}
```

`serde_json::from_str(text)` does the work. The `: Creature` on the left tells it what to build, the same way `Option<u64>` told `parse` last night.

Parsing can fail, so `from_str` returns a `Result`. `.expect("...")` says: give me the value, and if there's an error, stop the program with this message. Stopping the program is called a **panic**. It's the wrong answer to a mistake a *user* makes, and Night 7 does better. But this data is compiled in, so if it's broken the program is broken, and it should say so loudly the first time anyone runs it.

## Use the new fields

Two small changes so that `aliases` and `host_intro` appear. In `list`:

```rust
fn list(creatures: &[Creature]) {
    println!("🎄 TONIGHT'S GUESTS · {} creatures", creatures.len());
    println!();
    for c in creatures {
        println!("{}", c.line());
        println!("     {}", c.description);
        if !c.aliases.is_empty() {
            println!("     Also known as: {}", c.aliases.join(", "));
        }
        println!("     {} facts on file, id {}", c.facts.len(), c.id);
    }
}
```

`join(", ")` glues a list of strings together with a separator. `!` is "not".

And in `random`, two more lines after the fact:

```rust
            println!("{}", c.facts[which]);
            println!();
            println!("📺 Auntie Aerial: “{}”", c.host_intro);
```

```bash
cargo run -q -- list
```

```
🎄 TONIGHT'S GUESTS · 3 creatures

  😈 Krampus        Alpine folklore of Austria, Bavaria and their neighbours
     The horned, chain-rattling demon who deals with the naughty while St Nicholas rewards the good
     Also known as: The Krampus, Krampusz
     7 facts on file, id krampus
  🐈 The Yule Cat   Icelandic folklore
     The giant Icelandic cat who eats anyone without new clothes by Christmas Eve
     Also known as: Jólakötturinn, Jolakotturinn, Christmas Cat
     7 facts on file, id yule-cat
  🐴 The Mari Lwyd  Welsh folklore, especially Glamorgan and Gwent
     A horse's skull on a pole, draped in white, that comes singing to Welsh doors at midwinter
     Also known as: Mari Lwyd, Grey Mare, Y Fari Lwyd
     7 facts on file, id mari-lwyd
```

```bash
TERMINAL_OF_TINSEL_SEED=25 cargo run -q -- random
```

```
🎄 RANDOM FESTIVE FACT

😈 Krampus
His night is 5 December, Krampusnacht, the eve of St Nicholas Day

📺 Auntie Aerial: “First guest of the season, and he's brought his own chains. He's on a list of his own, and he's checking yours twice.”
```

Seed 25 still picks Krampus, but a different fact from last night: he has seven to choose from now, not three.

```bash
git add .
git commit -m "feat: creatures from JSON"
```

## Try it

- Break the data: delete a comma in `krampus.json` and run the program. It builds, then panics with the message from `expect` and serde's own report of the line and column. Mend it.
- Remove `default` from the serde attribute, then delete the `"aliases"` line from `yule-cat.json`. Now the error is "missing field `aliases`". Put both back.
- Remove `rename_all = "camelCase"` and run `random`. No error, but Auntie Aerial has nothing to say: the struct is now looking for `host_intro`, the file says `hostIntro`, and `default` quietly filled in an empty string. Defaults are kind to authors and can hide typos, which is why Night 8 adds tests that hold the built-in data to the full set.

## 🎄 Festive fact

The name serde is short for **ser**ialize and **de**serialize. Its derives are *procedural macros*: small Rust programs that the compiler runs on your source code while compiling it. That's why the `derive` feature has to be switched on, and why the first build after adding it takes a little longer: Cargo is compiling the macro and the parser it's built on before it can compile you.

## 🕯️ Before morning

Add the Yule Lads. Copy `yule-lads.json` from the finished repository's `packs/nordic` folder into yours, and add a fourth `include_str!`.

Hint: the array's type says how many items it holds, so the compiler will have a view on `[&str; 3]`. Read its error before you fix it. Then run `list`: the Yule Lads are Grýla's thirteen sons, and they have more nicknames than anyone else on the bill.

> 📺 *"Wrapped and labelled. Tomorrow the other nineteen guests arrive all at once, and we shall need somewhere to put them: packs, a vault, and a few more rooms in the house."*

[← Night 4](night-04.md) · [Index](README.md) · [Night 6 →](night-06.md)
