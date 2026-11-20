# Night 10 · Every Creature in Its Colours

> 📺 *"Good evening, and welcome to the last night of typing. Each of tonight's guests has sent ahead a portrait and a note of their colours: Krampus in red and ember orange, the Mari Lwyd in bone white. We shall hang them properly. And for the viewers at home who are not people at all but scripts, we have something special: the plain facts, in JSON."*

**Tonight you'll learn**
- Nested structs in JSON: a `Theme` inside each creature
- Loading an optional file beside each creature: the ASCII portrait
- A printed creature page built from last night's `Painter`
- `#[derive(Serialize)]`: structs back out to JSON
- `--json` as a global flag, for output and for errors
- Structs that borrow: a first lifetime annotation

**Where we are:** colour, boxes and plain mode for `list`, `random` and the welcome. The creature page is still bare `println!`, and every creature is the same colour.

## More of each creature

Each JSON file has had more in it than we've read. Tonight we take two more fields. In `src/creatures.rs`, change the serde import to:

```rust
use serde::{Deserialize, Serialize};
```

Replace the `Creature` struct and add a struct under it:

```rust
/// A single creature and what we know about it.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Creature {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
    pub emoji: String,
    pub description: String,
    pub origin: String,
    pub legend: String,
    pub facts: Vec<String>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub host_intro: String,
    pub theme: Theme,
    /// The portrait, from `<id>.txt` beside the creature's JSON file.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub ascii: String,
    /// Filled in from the directory the creature was loaded from.
    pub pack: String,
}

/// A creature's colours, as `#RRGGBB`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Theme {
    pub primary: String,
    pub accent: String,
}
```

In the JSON, `"theme": {"primary": "#D00000", "accent": "#F48C06"}` is an object inside an object. In Rust it's a field whose type is another struct that also derives `Deserialize`. serde follows the shape down.

There are now `Serialize` derives, and some new attributes. We'll come back to both.

## The portrait

Beside every `krampus.json` there's a `krampus.txt`: an ASCII drawing, at most 46 columns by 20 lines. In `Vault::load`, add the lines between `c.pack = pack.id.clone();` and `creatures.push(c);`:

```rust
                c.pack = pack.id.clone();
                // The portrait sits beside the JSON file. Not every creature
                // has to have one.
                c.ascii = PACKS_DIR
                    .get_file(format!("{pack_id}/{id}.txt"))
                    .and_then(|f| f.contents_utf8())
                    .map(tidy_art)
                    .unwrap_or_default();
                creatures.push(c);
```

Unlike the JSON, a portrait is optional, so this doesn't use `read`, which panics. `get_file` gives an `Option`, `map(tidy_art)` tidies the text if there is some, and `unwrap_or_default()` settles for an empty string if there isn't. A function's name can be passed wherever a closure is expected.

Add a method to `impl Vault`, before `by_pack`:

```rust
    /// The pack a creature belongs to.
    pub fn pack_of(&self, c: &Creature) -> Option<&Pack> {
        self.packs.iter().find(|p| p.id == c.pack)
    }
```

And the helper, above `fn unknown`:

```rust
/// A portrait without its trailing newline and with Windows line endings
/// normalised: a stray `\r` would garble the layout.
fn tidy_art(art: &str) -> String {
    art.replace("\r\n", "\n").trim_end_matches('\n').to_string()
}
```

Git on Windows may check text files out with `\r\n` line endings, and a stray `\r` in a drawing sends the cursor back to the start of the line.

New fields need new checks, or `#[serde(default)]` will hide a gap. In the tests at the end of the file, add three lines to the loop in `every_creature_is_complete`, after the line that counts the facts:

```rust
            for hex in [&c.theme.primary, &c.theme.accent] {
                assert!(crate::render::parse_hex(hex).is_some(), "{id}: {hex}");
            }
```

And add a test after it:

```rust
    #[test]
    fn every_portrait_is_printable_ascii_and_fits() {
        // The most a portrait may measure.
        const ART_COLUMNS: usize = 46;
        const ART_LINES: usize = 20;
        for c in &Vault::load().creatures {
            let id = &c.id;
            assert!(!c.ascii.trim().is_empty(), "{id}: no portrait");
            assert!(c.ascii.lines().count() <= ART_LINES, "{id}: too tall");
            for line in c.ascii.lines() {
                assert!(line.len() <= ART_COLUMNS, "{id}: too wide: {line}");
                assert!(!line.ends_with(' '), "{id}: trailing space");
                assert!(
                    line.chars().all(|ch| (' '..='~').contains(&ch)),
                    "{id}: not printable ASCII: {line}"
                );
            }
        }
    }
```

`(' '..='~')` is the range of printable ASCII characters. Anything else risks a character two columns wide, which would push the rest of its line out of true.

## The creature page

In `src/render.rs`, add a method to `impl Painter`, after `boxed`:

```rust
    /// Wraps text under a bullet, with the later lines indented to match.
    fn bullet(&self, text: &str, width: usize) -> Vec<String> {
        let marker = if self.plain { " - " } else { " • " };
        let options = textwrap::Options::new(width.max(10))
            .initial_indent(marker)
            .subsequent_indent("   ");
        textwrap::wrap(text, options)
            .iter()
            .map(|line| line.to_string())
            .collect()
    }
```

textwrap can indent for us: the first line of each fact starts with the bullet, and the lines after it with three spaces, so the text hangs neatly.

Now the page. Add it above `fact_card`:

```rust
/// A creature's full page.
pub fn creature(p: &Painter, c: &Creature, pack_name: &str) -> String {
    let w = p.text_width();
    let (primary, accent) = (c.theme.primary.as_str(), c.theme.accent.as_str());
    let mut out = vec![p.bold(
        &format!("{}{}", p.glyph(&c.emoji), c.name.to_uppercase()),
        primary,
    )];
    out.extend(p.wrap(&c.description, w, |l| p.italic(l, SNOW)));
    let meta = format!("{} · pack: {pack_name}", c.origin);
    out.extend(p.wrap(&meta, w, |l| p.paint(l, DIM)));
    out.push(String::new());
    if !c.ascii.is_empty() {
        out.extend(c.ascii.lines().map(|l| p.paint(l, primary)));
        out.push(String::new());
    }
    out.extend(p.wrap(
        &format!("{}{}: “{}”", p.glyph("📺"), host::NAME, host::intro(c)),
        w,
        |l| p.italic(l, GOLD),
    ));

    out.push(String::new());
    out.push(p.heading("FACTS", accent));
    for fact in &c.facts {
        out.extend(p.bullet(fact, w));
    }

    out.push(String::new());
    out.push(p.heading("THE LEGEND", accent));
    out.extend(p.wrap(&c.legend, w, |l| l.to_string()));

    out.join("\n") + "\n"
}
```

There's nothing new in it: lines pushed onto a `Vec`, each wrapped and painted by the `Painter`. The difference from last night is the colours. The title and the portrait use the creature's `primary`, the headings its `accent`. The portrait's lines are painted but never wrapped: a drawing has to keep its shape.

The other pages should wear the creature's colours too. In `list`, change `p.bold(&name, SNOW)` to:

```rust
                p.bold(&name, &c.theme.primary),
```

And replace `fact_card`:

```rust
/// A boxed fact, introduced by a title and signed off by the host.
pub fn fact_card(p: &Painter, title: &str, c: &Creature, fact: &str, quip: &str) -> String {
    let outer = p.width.saturating_sub(2).min(66);
    let inner = outer.saturating_sub(4).max(20);
    let mut lines = vec![
        p.bold(&format!("{}{title}", p.glyph("🎄")), GOLD),
        String::new(),
        p.bold(
            &format!("{}{}", p.glyph(&c.emoji), c.name),
            &c.theme.primary,
        ),
    ];
    lines.extend(p.wrap(fact, inner, |l| p.paint(l, SNOW)));
    lines.push(String::new());
    lines.extend(p.wrap(
        &format!("{}{}: “{quip}”", p.glyph("📺"), host::NAME),
        inner,
        |l| p.italic(l, GOLD),
    ));
    p.boxed(&lines, inner, &c.theme.accent)
}
```

## Speaking to machines

A person wants a page. A script wants data. `--json` gives every command a second output: the same information as JSON, with no decoration.

In `src/cli.rs`, add a flag as the first field of `Cli`:

```rust
    /// Print the command's data as JSON instead of drawing it (errors become a JSON object on stderr)
    #[arg(long, global = true)]
    pub json: bool,
```

Now those `Serialize` derives. **`Serialize`** is `Deserialize` backwards: the ability to be written out as data. With both, a `Creature` makes the round trip from JSON to struct and back. The attributes on its fields tidy the output:

- `rename_all = "camelCase"` works in both directions, so `host_intro` is written as `hostIntro`.
- `skip_serializing_if = "Vec::is_empty"` leaves a field out when the named function says so. A creature with no aliases has no `aliases` key, instead of an empty list.

Errors should be data too. In `src/error.rs`, add `use serde::Serialize;` under `use std::fmt;`, then change the derives on `Code` and `TinselError`:

```rust
/// The error codes a script can match on. They are stable; the messages are not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Code {
```

```rust
/// An error a player can act on: what went wrong and, sometimes, what to try.
#[derive(Debug, Clone, Serialize)]
pub struct TinselError {
    pub code: Code,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}
```

`rename_all = "kebab-case"` on an enum writes `UnknownCreature` as `"unknown-creature"`. That string is the stable code scripts will match on.

Add a method to `impl TinselError`, after `with_hint`, and a test in the tests module:

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

```rust
    #[test]
    fn json_error_has_kebab_case_code_and_skips_empty_hint() {
        let e = TinselError::new(Code::UnknownCreature, "no such creature");
        assert_eq!(
            e.to_json(),
            r#"{"error":{"code":"unknown-creature","message":"no such creature"}}"#
        );
    }
```

`Wrapped` is a struct declared inside a function, used once, to put the error under an `"error"` key. It holds a *reference*, and a struct that holds a reference must say how long the reference lives: `<'a>` names a lifetime, and `&'a TinselError` uses it. Read it as "a `Wrapped` can't outlive the error it points at". The compiler checks that; we only had to give it a name to check with. `r#"..."#` is a raw string, in which quotes need no escaping.

## main

In `src/main.rs`, add `use serde::Serialize;` under `use clap::Parser;`. Replace `main`:

```rust
fn main() -> ExitCode {
    let cli = Cli::parse();
    let wants_json = cli.json;
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
            ExitCode::from(e.code.exit_code())
        }
    }
}
```

Replace the `Show` struct, and add a struct below it:

```rust
/// What every command needs: how to draw, which creatures are in play, and
/// whether to print data instead.
struct Show {
    painter: Painter,
    vault: Vault,
    json: bool,
}

/// What `random --json` prints.
#[derive(Serialize)]
struct FactJson<'a> {
    id: &'a str,
    creature: &'a str,
    fact: &'a str,
}
```

In `run`, add `json: cli.json,` to the `Show { ... }` literal, after `vault`. Then replace the whole `impl Show` block and add `print_json` under it:

```rust
impl Show {
    /// Runs the command. One flat match, so that a new `Command` variant
    /// fails to compile until it has an arm here.
    fn dispatch(&self, command: Option<Command>) -> Result<()> {
        match command {
            None => self.welcome(),
            Some(Command::List) => self.list(),
            Some(Command::Creature { name }) => self.creature(&name.join(" ")),
            Some(Command::Random) => self.random(),
        }
    }

    fn welcome(&self) -> Result<()> {
        let mut rng = Rng::from_env_or_clock();
        let greeting = host::greeting(&mut rng);
        if self.json {
            return print_json(&serde_json::json!({
                "host": host::NAME,
                "channel": host::CHANNEL,
                "greeting": greeting,
            }));
        }
        print!("{}", render::welcome(&self.painter, greeting));
        Ok(())
    }

    fn list(&self) -> Result<()> {
        if self.json {
            return print_json(&self.vault.creatures);
        }
        print!("{}", render::list(&self.painter, &self.vault));
        Ok(())
    }

    fn creature(&self, name: &str) -> Result<()> {
        let c = self.vault.find(name)?;
        if self.json {
            return print_json(c);
        }
        let pack = self.vault.pack_of(c).map_or(c.pack.as_str(), |p| &p.name);
        print!("{}", render::creature(&self.painter, c, pack));
        Ok(())
    }

    fn random(&self) -> Result<()> {
        let mut rng = Rng::from_env_or_clock();
        let (who, which) = random_fact(&self.vault.creatures, &mut rng)
            .ok_or_else(|| TinselError::new(Code::Usage, "no facts to tell tonight"))?;
        let c = &self.vault.creatures[who];
        let fact = &c.facts[which];
        if self.json {
            return print_json(&FactJson {
                id: &c.id,
                creature: &c.name,
                fact,
            });
        }
        let quip = host::quip(&mut rng);
        print!(
            "{}",
            render::fact_card(&self.painter, "RANDOM FESTIVE FACT", c, fact, quip)
        );
        Ok(())
    }
}

fn print_json<T: Serialize + ?Sized>(value: &T) -> Result<()> {
    let text = serde_json::to_string_pretty(value)
        .map_err(|e| TinselError::new(Code::Usage, e.to_string()))?;
    println!("{text}");
    Ok(())
}
```

Every command now starts the same way: if `self.json`, print the data and return; otherwise draw.

- **`FactJson<'a>`** borrows its three strings, with the same lifetime annotation as `Wrapped`.
- **`serde_json::json!`** builds a one-off value from something that looks like JSON, for the welcome, which has no struct.
- **`print_json<T: Serialize + ?Sized>`** is a **generic** function: it works for any type `T` that can be serialised. `?Sized` relaxes the usual rule that a generic type has a size known at compile time, so a slice such as `[Creature]` is welcome too.
- **`map_err`** converts serde's error into ours, so that `?` can pass it on.
- In `creature`, `map_or(c.pack.as_str(), |p| &p.name)` gives the pack's display name, or its id if the pack can't be found.

## Test it

In `tests/cli.rs`, add `use serde_json::Value;` under the other imports, and a helper under `tinsel`:

```rust
fn json(args: &[&str]) -> Value {
    let out = tinsel().args(args).arg("--json").output().unwrap();
    assert!(
        out.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).unwrap()
}
```

Add two lines to the list in `every_command_runs`, after `vec!["random"],`:

```rust
        vec!["random", "--json"],
        vec!["list", "--no-color"],
```

And two tests at the end:

```rust
#[test]
fn json_output_is_the_data() {
    assert_eq!(json(&["list"]).as_array().unwrap().len(), 22);
    let cat = json(&["creature", "yule", "cat"]);
    assert_eq!(cat["id"], "yule-cat");
    assert_eq!(cat["pack"], "nordic");
    assert!(cat["ascii"].as_str().unwrap().contains('|'));
    assert_eq!(json(&["random"])["id"], json(&["random"])["id"]);
    assert_eq!(json(&[])["host"], "Auntie Aerial");
}

#[test]
fn errors_are_json_too() {
    let out = tinsel()
        .args(["creature", "krampsu", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    let error: Value = serde_json::from_slice(&out.stderr).unwrap();
    assert_eq!(error["error"]["code"], "unknown-creature");
}
```

`Value` is serde_json's type for "any JSON". It can be indexed like a map, `cat["id"]`, which suits a test that only cares about a few fields.

## Run it

```bash
cargo run -q -- creature krampus
```

Piped, so plain:

```
KRAMPUS
The horned, chain-rattling demon who deals with the naughty while St Nicholas
rewards the good
Alpine folklore of Austria, Bavaria and their neighbours · pack: St Nicholas's
Companions

      /\                /\
     /  \              /  \
    | /\ \            / /\ |
    | \ \ \__________/ / / |
     \ \_/            \_/ /
      \   .--.    .--.   /
      |  ( o  )  (  o )  |
      |   '--'    '--'   |
      |       /  \       |
       \     '----'     /
        \   \VVVVVV/   /
         \   |    |   /
          '-.| () |.-'
     o=o=o=o | () | o=o=o=o
              \  /
               \/

Auntie Aerial: “First guest of the season, and he's brought his own chains. He's
on a list of his own, and he's checking yours twice.”

-- FACTS -----------------------------------------------------------------------
...
```

```bash
TERMINAL_OF_TINSEL_SEED=25 cargo run -q -- random --json
```

```
{
  "id": "ghost-yet-to-come",
  "creature": "The Ghost of Christmas Yet to Come",
  "fact": "It never speaks, answering only by pointing"
}
```

```bash
cargo run -q -- creature krampsu --json
```

```
{"error":{"code":"unknown-creature","message":"no creature called \"krampsu\" is out tonight. Did you mean Krampus?","hint":"try 'terminal-of-tinsel list' for tonight's guests"}}
```

```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
git add .
git commit -m "feat: creature pages, portraits and --json"
```

## The end of the build-along

That's the last line you type from a lesson. What you have is a small, complete program: three commands and a welcome, 22 creatures in four embedded packs, a `Vault`, a seeded `Rng`, a `TinselError` with stable codes, lookup with aliases and "did you mean", a `Painter` with a plain mode, `--json` throughout, and 24 tests.

It's also a faithful miniature of the finished one. Your seven files have the same names, and most of your functions the same signatures, as [`src/cli.rs`](../src/cli.rs), [`src/creatures.rs`](../src/creatures.rs), [`src/error.rs`](../src/error.rs), [`src/host.rs`](../src/host.rs), [`src/render.rs`](../src/render.rs), [`src/rng.rs`](../src/rng.rs) and [`src/main.rs`](../src/main.rs). Open any of them beside yours. The finished `Creature` reads the fields we skipped (`debut`, `film`, `myths`, `quotes`, `stats`, `powers`, `weaknesses`, `legacy`), and its page has sections for them. One difference is deliberate: the finished page doesn't print the portrait, because the portrait's home is the full-screen explorer.

**From tomorrow the lessons change.** The program becomes too big to type in an evening, so Nights 11 to 24 teach each idea with the key code quoted from the finished repository, the clone you made on Night 5, and point at the file for the rest. Keep your own project: it's the best place to try things.

## Try it

- If you have `jq`: `cargo run -q -- list --json | jq '.[].name'`.
- Run `cargo run -q -- --json` and compare the order of the keys with the order in the `json!` call. They've been sorted: see tonight's fact.
- Change a `primary` colour in a creature's JSON to `"#12345"` and run `cargo test`. The data test names the creature and the bad value. Restore it with `git checkout packs`.

## 🎄 Festive fact

When serde serialises a struct, the keys come out in the order the fields are declared, which is why `id` is first in `creature --json`. A `json!` object is different: serde_json keeps it in a map sorted by key, so the welcome's keys come out alphabetically, `channel`, `greeting`, `host`, whatever order you wrote them in. (serde_json has an optional feature, `preserve_order`, that keeps insertion order instead.)

## 🕯️ Before morning

Write a test in `src/render.rs`, `pages_fit_the_width`, that draws every creature's page at widths 40, 60 and 100, plain and in colour, and checks that no line is wider than it should be.

Hint: loop over the widths, over `[true, false]` and over `Vault::load().creatures`; measure each line with `visible_width`. A line may be as wide as `p.text_width()`, or 46 columns if that's larger, because a portrait isn't wrapped. The finished program has a test of the same name.

> 📺 *"Hung, lit and labelled, and you built every inch of it. Tomorrow Channel 25 tunes in properly: the terminal goes dark, the picture rolls, and the programme stops printing and starts broadcasting. Bring a cushion."*

[← Night 9](night-09.md) · [Index](README.md) · [Night 11 →](night-11.md)
