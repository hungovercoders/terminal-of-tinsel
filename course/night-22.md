# Night 22 · Open the Doors

> 📺 *"So far this has been a private party: my guests, my studio, and you on the sofa. Tonight we open two doors. One is at the back, for scripts and other programs that want our facts without the tinsel. The other is at the front, for guests you've invented yourselves. Wipe your feet."*

**Tonight you'll learn**
- `--json` on every command, and why `main` looks for it before clap does
- Errors as data: one JSON object on stderr with a stable code
- Output structs that borrow, and `skip_serializing_if`
- A trait with two implementations: `PackFiles` over embedded and on-disk files
- Trait objects: what `&dyn PackFiles` means
- Lenient loading: validation, defaults and warnings instead of failures

**Where we are:** every command works and the calendar knows the date. Tonight the program learns to talk to other programs, and to load creatures it wasn't compiled with.

## Data out

On Night 10 you added `--json`. In the finished program it is still one global flag, declared on `Cli` in [`src/cli.rs`](../src/cli.rs) with `#[arg(long, global = true)]`, and every method on `Show` in [`src/main.rs`](../src/main.rs) checks it. The pattern is always the same: work out the values, then either print them as data or draw them.

```rust
    fn countdown(&self, date: Option<&str>) -> Result<()> {
        let today = calendar::parse_date(date)?;
        if !self.json {
            print(&render::countdown(&self.painter, &self.vault, today));
            return Ok(());
        }
        let guest = calendar::advent_guest(today, &self.vault);
        print_json(&CountdownJson {
            date: today.to_string(),
            christmas: calendar::is_christmas(today),
            sleeps_until_christmas: calendar::sleeps_until_christmas(today),
            // ...
        })
    }
```

Last night's pure functions pay a second time here: the drawing and the JSON call the same functions in `calendar.rs`, so they can't disagree.

The shape of the output is a struct of its own:

```rust
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CountdownJson<'a> {
    date: String,
    christmas: bool,
    sleeps_until_christmas: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    advent: Option<AdventJson<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    twelve_days: Option<u8>,
    yule_lads_in_town: u8,
    moon: MoonJson,
    // ...
    #[serde(skip_serializing_if = "Vec::is_empty")]
    notes: Vec<String>,
}
```

Three things to notice. `rename_all = "camelCase"` turns `sleeps_until_christmas` into `sleepsUntilChristmas`, as it did for the creature files on Night 5. `skip_serializing_if` names a function; when it returns true the field is left out altogether, so on a night outside the Twelve Days there is no `"twelveDays": null` to trip over. And the `'a` is a lifetime: `AdventJson<'a>` holds `&'a str` fields that point into the vault instead of copying the strings. The struct lives for one function call, the vault outlives it, and the compiler checks that.

Why not serialise the calendar's own types directly? Because the JSON is a promise to other people's scripts, and a separate struct lets `calendar::Moon` change inside without changing what a script sees.

```bash
terminal-of-tinsel tonight --date 2026-12-21 --json
```

```
{
  "date": "2026-12-21",
  "features": [
    {
      "time": "7:00 PM",
      "creature": "wild-hunt",
      "name": "The Wild Hunt"
    },
    {
      "time": "8:45 PM",
      "creature": "hans-trapp",
      "name": "Hans Trapp"
    }
  ],
  "notes": [
    "The winter solstice is here, or near enough: the longest night of the year.",
    "10 Yule Lads are in town.",
    "4 sleeps until Christmas."
  ]
}
```

## Errors are data too

A script that asks for JSON needs its failures in JSON as well. The top of `main` handles that:

```rust
    let args: Vec<String> = std::env::args().collect();
    let wants_json = args.iter().any(|a| a == "--json");
    let cli = match Cli::try_parse_from(&args) {
        Ok(cli) => cli,
        Err(e) if wants_json && !is_help(&e) => {
            eprintln!("{}", clap_error(&e).to_json());
            return ExitCode::FAILURE;
        }
        Err(e) => e.exit(),
    };
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            if wants_json {
                eprintln!("{}", e.to_json());
            } else {
                eprintln!("error: {}", e.message);
                if let Some(hint) = &e.hint {
                    eprintln!("hint: {hint}");
                }
            }
            ExitCode::FAILURE
        }
    }
```

`wants_json` is found by looking through the raw arguments, before clap has parsed anything. It has to be: if the arguments are wrong (`--questions many`), parsing fails and there is no `cli.json` to read, yet the error must still come out as JSON. `Err(e) if wants_json && !is_help(&e)` is a match arm with a guard; `--help` and `--version` arrive as "errors" from clap and are left to print normally.

The object itself comes from [`src/error.rs`](../src/error.rs):

```rust
/// The error codes a script can match on. They are stable; the messages are not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Code {
    UnknownCreature,
    AmbiguousCreature,
    BadPack,
    // ...
}
```

```rust
    /// The error as `--json` reports it: one object on stderr.
    pub fn to_json(&self) -> String {
        #[derive(Serialize)]
        struct Wrapped<'a> {
            error: &'a TinselError,
        }
        serde_json::to_string(&Wrapped { error: self }).unwrap_or_default()
    }
```

A struct can be declared inside a function when nothing else needs it; `Wrapped` exists only to put the error under an `"error"` key. Ask for a guest who isn't on the roster (and, being somebody's property, never will be):

```bash
terminal-of-tinsel creature rudolph --json
```

```
{"error":{"code":"unknown-creature","message":"no creature called \"rudolph\" is out tonight","hint":"try 'terminal-of-tinsel list' for tonight's guests"}}
```

The rules a script can rely on: data goes to stdout, the error goes to stderr as one line, the exit status is 1, and the `code` never changes. The message may be reworded in any release.

## One loader, two places

Until now every creature was compiled into the binary with `include_dir!`. A community pack is a folder on the player's disk. The loader should not care which, so [`src/creatures.rs`](../src/creatures.rs) describes what it needs from "a place with files in it" as a trait:

```rust
/// Where a pack's files come from: compiled into the binary, or a directory
/// on disk. Go has `fs.FS` for this; here it's a two-method trait.
trait PackFiles {
    /// The text of a file in the pack, if it's there and it's UTF-8.
    fn read(&self, name: &str) -> Option<String>;
    /// The names of the pack's `.json` files, without the extension, sorted.
    fn json_stems(&self) -> Vec<String>;
}

struct Embedded<'a>(&'a Dir<'a>);

impl PackFiles for Embedded<'_> {
    fn read(&self, name: &str) -> Option<String> {
        let file = self.0.get_file(self.0.path().join(name))?;
        file.contents_utf8().map(str::to_string)
    }
    // ...
}

struct OnDisk(PathBuf);

impl PackFiles for OnDisk {
    fn read(&self, name: &str) -> Option<String> {
        std::fs::read_to_string(self.0.join(name)).ok()
    }
    // ...
}
```

`Embedded` and `OnDisk` are tuple structs, each wrapping one value, reached as `self.0`. The trait has only the two methods the loader calls.

The loader takes either:

```rust
fn read_pack(files: &dyn PackFiles, dir: &str, source: &str) -> (Pack, Vec<Creature>, Vec<String>) {
```

`&dyn PackFiles` is a **trait object**: a reference to some value that implements `PackFiles`, with the concrete type decided at run time. The call `files.read("pack.json")` looks up the right `read` through a small table of function pointers. The other way to write it, `files: &impl PackFiles`, would have the compiler make two copies of `read_pack`, one per type. Either works here; `dyn` keeps one copy of a long function, and the cost of the lookup is nothing next to reading a file.

Look at the return type too. `read_pack` doesn't return a `Result`. It returns the pack, the creatures it could read, and a list of problems as text. One broken file should not sink the whole pack, so problems are collected and the caller decides how serious they are. The two callers decide differently:

```rust
            let (pack, members, problems) = read_pack(&Embedded(dir), pack_id, BUILT_IN);
            assert!(problems.is_empty(), "packs/{pack_id}: {problems:?}");
```

For the built-in packs any problem is a bug in the repository, so `Vault::load` panics and the tests catch it. For community packs, `add_community` turns each problem into a warning and carries on.

## Strict for us, lenient for guests

On Night 5 every field of `Creature` got a serde default. That makes a three-line creature legal JSON for the parser, so something else has to decide whether it's playable:

```rust
/// What a community creature is missing to be playable.
fn validate(c: &Creature) -> Vec<&'static str> {
    let mut problems = Vec::new();
    if c.name.trim().is_empty() {
        problems.push("needs a name");
    }
    if c.description.trim().is_empty() {
        problems.push("needs a description");
    }
    if c.facts.iter().all(|f| f.trim().is_empty()) {
        problems.push("needs at least one fact");
    }
    if c.stats.labelled().iter().any(|(_, v)| *v > 10) {
        problems.push("stats must be between 0 and 10");
    }
    problems
}
```

A creature that passes is then filled in by `apply_defaults`: the legend falls back to the description, the emoji to a present, each missing stat to 5, the colours to the house gold and green. I put this file in a scratch packs folder:

```json
{"name": "The Sock Goblin", "description": "Takes one sock from every pair", "facts": ["It only ever takes the left one"]}
```

and `terminal-of-tinsel creature sock goblin --json` printed, in part:

```
{
  "id": "sock-goblin",
  "name": "The Sock Goblin",
  "emoji": "🎁",
  "description": "Takes one sock from every pair",
  "origin": "Unknown",
  ...
  "legend": "Takes one sock from every pair",
  "powers": [
    "Sheer festive spirit"
  ],
  ...
  "pack": "attic"
}
```

The id came from the file name and the pack from the folder. Two bad files beside it were skipped, each with a line on stderr:

```
warning: community pack: pack "attic": broken.json: EOF while parsing an object at line 2 column 0
warning: community pack: pack "attic": skipping "no-facts": needs at least one fact
```

Under `--json` those warnings are kept quiet, so that stderr holds nothing but the one error object a script might parse.

One check is about safety, not tidiness. An id becomes a file name (`<id>.txt` for the portrait), so `read_pack` refuses any id that `is_slug` rejects. Without that, an id of `"../../secret"` would make the program read a file outside the pack.

The built-in creatures get none of this leniency: the data tests from Night 8 hold them to the full set of fields.

## packs new

Nobody should have to write a pack from an empty folder. `packs new <id>` calls `scaffold` in [`src/packs.rs`](../src/packs.rs), which refuses a bad id, an id that is taken, and a folder that already exists, and only then writes three files:

```bash
terminal-of-tinsel packs new office-party
```

```
Created ~/.config/terminal-of-tinsel/packs/office-party

Edit office-party-creature.json (and office-party-creature.txt for its portrait), then try:
  terminal-of-tinsel creature office-party-creature
  terminal-of-tinsel quiz --pack office-party

Only name, description and facts are required.
```

(That is the path on Linux. With `TERMINAL_OF_TINSEL_HOME` set, the pack goes in its `packs` folder.)

Every check comes before the first write, and a test asserts that a refusal creates nothing. The example creature is written from a text template, not built with serde_json's `json!` macro, for a human reason: `json!` sorts keys alphabetically by default, which would put `aliases` first and bury `name`. The template keeps the fields in the order a person fills them in. [CONTRIBUTING.md](../CONTRIBUTING.md) describes every field.

## Try it

- Set `TERMINAL_OF_TINSEL_HOME` to a scratch folder, run `cargo run -- packs new office-party`, then `cargo run -- packs` and `cargo run -- quiz --pack office-party`.
- Delete the `facts` from your new creature and run `cargo run -- list`. Read the warning, then put them back.
- Run `cargo run -- countdown --date 2026-07-01 --json` and see which fields are missing.

## 🎄 Festive fact

serde does no reflection at run time. `#[derive(Serialize)]` is a procedural macro: while your crate compiles, it writes an ordinary `impl Serialize` for the struct, with one call per field. That is why `rename_all` and `skip_serializing_if` cost nothing when the program runs, and why a typo in a field attribute is a compile error.

## 🕯️ Before morning

Write a third implementation of `PackFiles` that holds its files in memory, as a `BTreeMap<String, String>` from file name to contents, and use it in a test of `read_pack`: a pack with one good creature and one whose JSON is broken should give back one creature and one problem.

*Hint:* the trait is private to `creatures.rs`, so put the struct and the test inside that file's `tests` module. `json_stems` can use `strip_suffix(".json")` on the map's keys, which a `BTreeMap` already keeps sorted.

> 📺 *"Both doors open, and nothing's blown out. Tomorrow is the night before the night before: we teach the shell to finish your sentences, write the manual, and let three different computers check our homework."*

[← Night 21](night-21.md) · [Index](README.md) · [Night 23 →](night-23.md)
