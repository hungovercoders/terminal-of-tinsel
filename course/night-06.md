# Night 6 · Packs of Creatures

> 📺 *"Good evening. Nineteen more guests have arrived at once and the hall is full of coats. A good house has rooms: one for the saint's companions, one for the Nordic contingent, one for Mr Dickens's ghosts. Tonight we sort the guests into packs, and the code into rooms of its own while we're at it."*

**Tonight you'll learn**
- Modules: splitting a program into files with `mod`, `pub` and `use`
- The whole `packs/` folder: one file per creature, one folder per pack
- `include_dir!`: compiling a whole directory into the program
- A `Pack` with an `order`, and a `Vault` with `load()` and `by_pack()`
- Iterator chains: `iter`, `map`, `filter`, `collect`

**Where we are:** one long `main.rs` that reads three JSON files.

## Fetch the rest

Replace your three files with the finished program's whole `packs` folder:

```bash
rm -r packs
cp -r ../finished/packs .
```

You now have four folders, `companions`, `nordic`, `midwinter` and `dickens`. Each holds a `pack.json`, one `<id>.json` per creature and, beside each of those, an `<id>.txt` with an ASCII portrait that Night 10 will use. There's also `advent.json`, which waits until Night 21.

A `pack.json` is short:

```json
{
  "id": "companions",
  "name": "St Nicholas's Companions",
  "description": "The saint brings the presents. These are the ones he brings along for everybody else.",
  "order": ["krampus", "belsnickel", "pere-fouettard", "hans-trapp", "knecht-ruprecht", "cert"]
}
```

`order` is the running order within the pack. Without it the creatures would appear alphabetically by file name, and Krampus would not open the show.

## Rooms for the code

A **module** is a named box of code. Declaring `mod rng;` in `main.rs` tells the compiler: there's a module called `rng`, and its contents are in `src/rng.rs`. Everything in a module is private to it unless marked `pub`. Other modules reach in with a path, `crate::rng::Rng`, where `crate` means the root of this program.

We'll make four modules. The program won't compile until all five files are in place.

### `src/cli.rs`

Move the `Cli` struct and the `Command` enum here, with `pub` on each and on the `command` field, which `main` needs to read:

```rust
//! The command line, defined with clap's derive API.

use clap::{Parser, Subcommand};

/// Channel 25's Midwinter Special: the creatures of Christmas folklore, live
/// in your terminal
#[derive(Debug, Parser)]
#[command(name = "terminal-of-tinsel", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// List every creature, grouped by pack
    List,

    /// A random creature fact
    Random,
}
```

A comment starting `//!` documents the thing it's *inside*, here the module.

### `src/rng.rs`

Move `SEED_ENV`, the `Rng` struct and its whole `impl` block here. The file starts like this:

```rust
//! A tiny, stable random number generator.

use std::time::{SystemTime, UNIX_EPOCH};

/// Fixes every random choice, so a run can be repeated exactly.
pub const SEED_ENV: &str = "TERMINAL_OF_TINSEL_SEED";

/// SplitMix64: fast, tiny, and good enough for a Christmas special.
#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }
```

The rest of the `impl` block is unchanged apart from `pub` in front of the other three functions: `pub fn from_env_or_clock`, `pub fn next_u64` and `pub fn below`.

### `src/host.rs`

Auntie Aerial gets a module of her own. The lines are hers, from the finished program:

```rust
//! Auntie Aerial, Terminal of Tinsel's host, broadcasting from the top of
//! the tree on Channel 25's Midwinter Special.

use crate::creatures::Creature;
use crate::rng::Rng;

/// The host's on-air name.
pub const NAME: &str = "Auntie Aerial";

/// The fictional station she broadcasts on.
pub const CHANNEL: &str = "Channel 25";

const GREETINGS: &[&str] = &[
    "Good evening, and a very merry midwinter. Welcome to the Channel 25 Midwinter Special. I'm your host, Auntie Aerial, broadcasting from the top of the tree.",
    "Pull up a chair by the fire, pour something warm and turn the lights down low. Tonight's guests have come a long way through the snow.",
    "Ah, you've tuned in. Lovely. Mind the tinsel, it's holding the aerial up.",
    "Welcome back, night owls and early stocking-checkers. The mince pies are warm and the visitors are at the door.",
];

/// A random opening line.
pub fn greeting(rng: &mut Rng) -> &'static str {
    GREETINGS[rng.below(GREETINGS.len())]
}

/// The host's introduction for a creature.
pub fn intro(c: &Creature) -> String {
    if c.host_intro.is_empty() {
        format!("Tonight's guest: {}. Wrap up warm.", c.name)
    } else {
        c.host_intro.clone()
    }
}
```

`&[&str]` is a slice of string slices. `GREETINGS[rng.below(GREETINGS.len())]` picks one by position. The return type `&'static str` has a **lifetime** in it: `'static` means "borrowed from something that lives for the whole run of the program", which text compiled into the binary does. Usually Rust works lifetimes out without being told. Here there's no borrowed argument the text could have come from, so we say.

`intro` returns an owned `String`, so it has to `.clone()` the creature's line instead of handing out the original.

### `src/creatures.rs`

This is tonight's real work. Add the crate that embeds directories:

```bash
cargo add include_dir@0.7.4
```

```rust
//! The creatures of Christmas folklore and their packs.
//!
//! The data lives in `packs/` as JSON and is compiled into the binary, so
//! the program needs nothing on disk to run.

use include_dir::{Dir, include_dir};
use serde::Deserialize;

static PACKS_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/packs");

/// The order packs appear in, from the saint's companions to Dickens.
pub const PACK_ORDER: &[&str] = &["companions", "nordic", "midwinter", "dickens"];

/// A single creature and what we know about it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Creature {
    pub id: String,
    pub name: String,
    pub aliases: Vec<String>,
    pub emoji: String,
    pub description: String,
    pub origin: String,
    pub facts: Vec<String>,
    pub host_intro: String,
    /// Filled in from the directory the creature was loaded from.
    pub pack: String,
}

/// A pack of creatures, such as the Nordic Yule.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Pack {
    pub id: String,
    pub name: String,
    pub description: String,
    pub order: Vec<String>,
}

/// Every creature and pack, loaded from the embedded data.
#[derive(Debug, Clone)]
pub struct Vault {
    pub packs: Vec<Pack>,
    pub creatures: Vec<Creature>,
}

impl Vault {
    /// Loads the built-in packs. The data is compiled in, so a failure here
    /// is a bug, not a player's mistake.
    pub fn load() -> Self {
        let mut packs = Vec::new();
        let mut creatures = Vec::new();
        for pack_id in PACK_ORDER {
            let pack: Pack = serde_json::from_str(read(&format!("{pack_id}/pack.json")))
                .unwrap_or_else(|e| panic!("packs/{pack_id}/pack.json: {e}"));
            for id in &pack.order {
                let mut c: Creature = serde_json::from_str(read(&format!("{pack_id}/{id}.json")))
                    .unwrap_or_else(|e| panic!("packs/{pack_id}/{id}.json: {e}"));
                c.pack = pack.id.clone();
                creatures.push(c);
            }
            packs.push(pack);
        }
        Self { packs, creatures }
    }

    /// The creatures grouped by pack, in pack order.
    pub fn by_pack(&self) -> Vec<(&Pack, Vec<&Creature>)> {
        self.packs
            .iter()
            .map(|p| {
                let members = self.creatures.iter().filter(|c| c.pack == p.id).collect();
                (p, members)
            })
            .collect()
    }
}

/// The text of a file under `packs/`.
fn read(path: &str) -> &'static str {
    PACKS_DIR
        .get_file(path)
        .and_then(|f| f.contents_utf8())
        .unwrap_or_else(|| panic!("missing packs/{path}"))
}
```

**`include_dir!`** is `include_str!` for a whole folder: at compile time it reads every file under `packs/` into the program. `$CARGO_MANIFEST_DIR` is the folder holding `Cargo.toml`. The result is kept in a `static`, a single value that lives for the whole run.

**`Creature`** has gained `pub` on every field and one new field, `pack`. No JSON file has a `pack` field: we fill it in from the folder the file was found in.

**`Vault`** is the name the finished program uses for "everything loaded". `load` walks `PACK_ORDER`. For each pack it reads `pack.json`, then one creature file per id in that pack's `order`, and stamps each creature with the pack's id. `pack.id.clone()` makes a copy because the pack still needs its own.

`unwrap_or_else(|e| panic!(...))` is `expect` with a message that can name the file: the closure receives the error `e` and only runs if there is one.

**`read`** asks the embedded directory for a file. `get_file` returns an `Option` (the file may not exist), and `contents_utf8` another (it may not be text). `and_then` chains them, as on Night 4.

**`by_pack`** is our first real iterator chain. Read it from the inside out:

- `self.creatures.iter()` visits each creature by reference.
- `.filter(|c| c.pack == p.id)` keeps the ones in this pack.
- `.collect()` gathers them into the `Vec<&Creature>` the return type asks for.

Around that, `self.packs.iter().map(...)` does it once per pack and pairs each pack with its members. The result borrows from the vault throughout: nothing is copied.

### `src/main.rs`

What's left is short. Replace the whole file:

```rust
mod cli;
mod creatures;
mod host;
mod rng;

use clap::Parser;

use cli::{Cli, Command};
use creatures::{Creature, Vault};
use rng::Rng;

fn main() {
    let cli = Cli::parse();
    let vault = Vault::load();
    match cli.command {
        None => welcome(),
        Some(Command::List) => list(&vault),
        Some(Command::Random) => random(&vault),
    }
}

fn welcome() {
    let mut rng = Rng::from_env_or_clock();
    println!("🎄 TERMINAL OF TINSEL");
    println!(
        "{}'s Midwinter Special, live in your terminal",
        host::CHANNEL
    );
    println!();
    println!("📺 {}: “{}”", host::NAME, host::greeting(&mut rng));
    println!();
    println!("Start here:  terminal-of-tinsel list");
}

fn list(vault: &Vault) {
    println!("🎄 TONIGHT'S GUESTS · {} creatures", vault.creatures.len());
    for (pack, creatures) in vault.by_pack() {
        println!();
        println!("── {} ({}) ──", pack.name.to_uppercase(), pack.id);
        println!("{}", pack.description);
        for c in creatures {
            println!("  {} {} · {}", c.emoji, c.name, c.origin);
            println!("     {}", c.description);
            if !c.aliases.is_empty() {
                println!("     Also known as: {}", c.aliases.join(", "));
            }
        }
    }
}

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

fn random(vault: &Vault) {
    let mut rng = Rng::from_env_or_clock();
    match random_fact(&vault.creatures, &mut rng) {
        Some((who, which)) => {
            let c = &vault.creatures[who];
            println!("🎄 RANDOM FESTIVE FACT");
            println!();
            println!("{} {}", c.emoji, c.name);
            println!("{}", c.facts[which]);
            println!();
            println!("📺 {}: “{}”", host::NAME, host::intro(c));
        }
        None => println!("No facts tonight. The crackers were empty."),
    }
}
```

The four `mod` lines are what make the other files part of the program: a file in `src/` that no `mod` names is not compiled. `for (pack, creatures) in vault.by_pack()` unpacks each pair as it goes.

## Run it

```bash
cargo run -q -- list
```

```
🎄 TONIGHT'S GUESTS · 22 creatures

── ST NICHOLAS'S COMPANIONS (companions) ──
The saint brings the presents. These are the ones he brings along for everybody else.
  😈 Krampus · Alpine folklore of Austria, Bavaria and their neighbours
     The horned, chain-rattling demon who deals with the naughty while St Nicholas rewards the good
     Also known as: The Krampus, Krampusz
  🧥 Belsnickel · The Palatinate in south-west Germany, carried to Pennsylvania by German settlers
     The ragged, fur-clad gift-giver who brings sweets in one hand and a switch in the other
     Also known as: Pelznickel, Belschnickel, Fur Nicholas
...
```

Twenty-two creatures in four packs (the output is cut short here), and every one of them is inside the binary. `random` now chooses among all 22, and the welcome draws on the generator too: run `cargo run -q` a few times and Auntie Aerial varies her greeting.

```bash
git add .
git commit -m "feat: packs, a vault and modules"
```

## Try it

- Remove `pub` from `pub fn load` and build. The error, "function `load` is private", names the function and where it's defined.
- Remove `"krampus"` from the `order` in `packs/companions/pack.json` and run `list`. He's gone: the order is the roster. Put him back.
- Swap two names in `PACK_ORDER` and run `list`.
- One caution with `include_dir`: Cargo notices when an embedded file is edited, but may not notice a brand-new file until something else changes. If a new file seems invisible, run `cargo clean -p terminal-of-tinsel` and build again.

## 🎄 Festive fact

Before the 2018 edition, a module with submodules had to live in a file called `mod.rs` inside a folder, so a large project had a dozen editor tabs all labelled `mod.rs`. Since Rust 1.30 you may write `src/explorer.rs` beside a folder `src/explorer/` instead. Both styles still work: the finished program's explorer uses the older one, [`src/explorer/mod.rs`](../src/explorer/mod.rs), which keeps all its files in one folder.

## 🕯️ Before morning

Add a method to `Vault` that finds a creature by its exact id:

```rust
pub fn get(&self, id: &str) -> Option<&Creature>
```

Hint: iterators have a `find` method that takes a closure and returns an `Option`. The finished one is a single line in [`src/creatures.rs`](../src/creatures.rs). Try it from `main` with `dbg!(vault.get("lussi"))`, then take both out again: tomorrow we write its cleverer sibling.

> 📺 *"Everybody has a room, and the coats are off the floor. Tomorrow a viewer rings in asking for 'the cat one', and we shall have to work out who they mean."*

[← Night 5](night-05.md) · [Index](README.md) · [Night 7 →](night-07.md)
