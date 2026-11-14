# Night 20 · What the Stocking Remembers

> 📺 *"You've pulled the crackers, wiped the window and refereed a fight on a pond. And when you closed the terminal, I forgot every bit of it. Tonight we hang a stocking by the fire, so that what you've earned is still there in the morning."*

**Tonight you'll learn**
- Where a program may keep its files, with the `directories` crate and an override
- A save file that old and new releases can both live with: defaults and a version
- Why a save from a newer release is refused, and why the file is written twice
- A rule that outranks the feature: saving must never stop a game
- Progress as a function: an `Outcome` goes in, `Unlocks` come out
- A `const` array of badges with its length in its type

**Where we are:** three games and an explorer, and nothing is remembered between runs.

## Where to keep it

Each operating system has a place for a program's settings, and it differs on all three. The `directories` crate knows them. [`src/store.rs`](../src/store.rs) asks it, after checking for an override:

```rust
pub fn dir() -> Result<PathBuf> {
    if let Some(home) = std::env::var_os(HOME_ENV).filter(|h| !h.is_empty()) {
        return Ok(PathBuf::from(home));
    }
    directories::ProjectDirs::from("", "", "terminal-of-tinsel")
        .map(|dirs| dirs.config_dir().to_path_buf())
        .ok_or_else(|| {
            TinselError::new(
                Code::Store,
                "couldn't work out where your home directory is",
            )
            .with_hint(format!("set {HOME_ENV} to a directory to keep progress in"))
        })
}
```

On Linux that is `~/.config/terminal-of-tinsel`, on macOS `~/Library/Application Support/terminal-of-tinsel`, and on Windows a folder under `AppData\Roaming`. `HOME_ENV` is `TERMINAL_OF_TINSEL_HOME`. The override exists for tests first: the end-to-end tests set it to a scratch directory on every command, so `cargo test` never touches your real stocking.

`ProjectDirs::from` returns an `Option`, because a machine may have no home directory, and `ok_or_else` turns `None` into an error with a hint.

## What is kept

```rust
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Progress {
    pub version: u32,
    /// Correct answers per creature id.
    pub knowledge: BTreeMap<String, u32>,
    /// Creature id to when it went in the stocking.
    pub captured: BTreeMap<String, String>,
    /// Badge id to when it was earned.
    pub badges: BTreeMap<String, String>,
    /// Creature pages visited in the explorer.
    pub seen: BTreeMap<String, bool>,

    pub quizzes_played: u32,
    // ...
}
```

Three choices here are about the future. `#[serde(default)]` on the struct means a field missing from the file takes its default, so a file written before a field existed still loads. Serde ignores fields it does not know, so the reverse works too. The maps are `BTreeMap`, not `HashMap`: a `BTreeMap` keeps its keys in order, so the file comes out in the same order every time and a player can read it. And there is a `version`.

## Loading, and the newer file

```rust
pub fn load(dir: &Path) -> Result<Progress> {
    let path = path_in(dir);
    let mut progress = match fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str(&text).map_err(|e| {
            store_error(&path, "read", e).with_hint(
                "fix the file, or run 'terminal-of-tinsel stocking reset' to start again",
            )
        })?,
        Err(e) if e.kind() == io::ErrorKind::NotFound => Progress::default(),
        Err(e) => return Err(store_error(&path, "read", e)),
    };
    // A file from a later release may hold things this one would drop on
    // saving, so it is left alone rather than rewritten as version 1.
    if progress.version > VERSION {
        return Err(TinselError::new(
            // ...
        )
        .with_hint("upgrade terminal-of-tinsel to use it"));
    }
    progress.version = VERSION;
    Ok(progress)
}
```

The `match` has three outcomes. The file was read: parse it, and if it is not valid JSON, say so with a way out. The file is not there: that is a new player, not an error, so the answer is an empty `Progress`. Anything else, such as no permission to read, is an error.

Then the version check. Suppose a later release saves format 2 with a new map of trophies, and you open that file with today's program. It would load: serde ignores the unknown field. Then you play one showdown, the program saves, and it writes the struct it knows, which has no trophies. Loading was harmless and saving destroyed data. Refusing to load is the only place to stop that, so a newer file is an error and the file is left untouched. An older file with no `version` at all reads as 0, passes, and is stamped as 1.

## Saving, in two steps

```rust
pub fn save(dir: &Path, progress: &Progress) -> Result<()> {
    let path = path_in(dir);
    let failed = |e: io::Error| store_error(&path, "save", e);
    fs::create_dir_all(dir).map_err(failed)?;
    let text = serde_json::to_string_pretty(progress).map_err(|e| store_error(&path, "save", e))?;
    let temp = dir.join(format!("{FILE}.{}.tmp", std::process::id()));
    let written = fs::File::create(&temp)
        .and_then(|mut f| {
            f.write_all(text.as_bytes())?;
            f.write_all(b"\n")?;
            f.sync_all()
        })
        .and_then(|()| fs::rename(&temp, &path));
    if written.is_err() {
        let _ = fs::remove_file(&temp);
    }
    written.map_err(failed)
}
```

Writing straight to `stocking.json` has a bad moment: the file is emptied, then filled. A crash or a full disk in between leaves half a file. So the new text goes to a temporary file beside the real one, is flushed to disk with `sync_all`, and is then renamed over it. The player has either the old stocking or the new one, never a torn one. The process id in the temporary name keeps two copies of the program from sharing it.

## Never stop a game

Every game ends by calling one method in [`src/main.rs`](../src/main.rs):

```rust
    /// Saves an outcome to the player's stocking and announces anything new.
    /// Trouble with the save file is reported but never stops the fun.
    fn record(&self, outcome: &Outcome) {
        let saved = store::dir().and_then(|dir| {
            let mut progress = store::load(&dir)?;
            let unlocks = stocking::apply(&mut progress, &self.everything.creatures, outcome);
            store::save(&dir, &progress)?;
            Ok(unlocks)
        });
        match saved {
            Ok(unlocks) => print(&render::unlocks(&self.painter, &unlocks)),
            // With the hint, so a broken save file comes with its way out.
            Err(e) => eprintln!("warning: this session wasn't saved: {e}"),
        }
    }
```

Look at the return type: there is none. The command methods on `Show` return `Result<()>`, and `record` deliberately does not. The player came to play. If the disk is full or the file is damaged, the game they finished still happened, the output they saw was still right, and the exit code should still be success. So inside the closure `?` gathers any failure, and outside it the failure becomes a warning on stderr. The `stocking` command is different: looking in the stocking is its whole job, so there a broken file is an error.

## From play to progress

`record` passes an `Outcome` to [`src/stocking.rs`](../src/stocking.rs). An `Outcome` describes one session: its `kind` (quiz, guess, mash or explore), the correct answers per creature, the score, whether it was finished, and `now`, the time play began.

```rust
pub fn apply<'a>(p: &mut Progress, all: &'a [Creature], o: &Outcome) -> Unlocks<'a> {
    let mut awards = Awards {
        when: o.now.strftime("%Y-%m-%dT%H:%M:%S").to_string(),
        unlocks: Unlocks::default(),
    };
    game_badges(p, o, &mut awards);
    // Playing at a memorable time. Reading in the explorer isn't playing.
    if o.kind != Kind::Explore {
        night_badges(p, o, &mut awards);
    }
    learn(p, all, o, &mut awards);
    collection_badges(p, all, &mut awards);
    awards.unlocks
}
```

`apply` reads no clock and no file. The time comes in with the outcome, which is why a test can award the Christmas Day badge in November by passing a date. It changes the `Progress` and returns only what is new, the `Unlocks`, for `main` to announce.

A creature goes in the stocking after three right answers about it, from either game:

```rust
    for (id, n) in &o.correct {
        *p.knowledge.entry(id.clone()).or_default() += n;
    }
    for c in all {
        let known = p.knowledge.get(&c.id).copied().unwrap_or(0);
        if known >= CAPTURE_AT && !p.captured.contains_key(&c.id) {
            p.captured.insert(c.id.clone(), awards.when.clone());
            awards.unlocks.captured.push(c);
        }
    }
```

`entry(...).or_default()` gives a mutable reference to the count for that id, inserting 0 first if there was none. The `!p.captured.contains_key` is what stops a creature being announced twice.

## Badges

```rust
const fn badge(
    id: &'static str,
    emoji: &'static str,
    name: &'static str,
    description: &'static str,
) -> Badge {
    // ...
}

/// Every badge, in the order they are shown.
pub const BADGES: [Badge; 17] = [
    badge(
        "first-cracker",
        "🎉",
        "First Cracker",
        "Finish your first quiz",
    ),
    // ...
];
```

`badge` is a `const fn`, a function the compiler can run while compiling, so the whole table is built into the program. The type is `[Badge; 17]`, an array, and an array's length is part of its type. Add an eighteenth badge without changing the 17 and the program does not compile. A slice, `&[Badge]`, would accept any number silently. The 17 is a small ceremony: the count appears in the README's badge table and in "1 of 17 badges", and the compiler makes you stop and look at it when it changes. The stocking page itself prints `BADGES.len()`, so it cannot be out of date.

Awarding goes through one method, so no badge can be given twice:

```rust
    fn give(&mut self, p: &mut Progress, earned: bool, id: &str) {
        if earned
            && !p.badges.contains_key(id)
            && let Some(b) = BADGES.iter().find(|b| b.id == id)
        {
            p.badges.insert(id.to_string(), self.when.clone());
            self.unlocks.badges.push(*b);
        }
    }
```

That `&& let` is a let chain: two tests and a pattern match in one `if`.

One last detail. `record` passes `self.everything.creatures`, not `self.vault.creatures`. `--pack nordic` narrows the creatures in play, and the badges count every built-in creature whatever was in play, so that playing with one pack cannot earn "Keeper of the Stocking" by capturing six creatures.

## Run it

Use a scratch directory. With seed 25:

```bash
export TERMINAL_OF_TINSEL_HOME=night20 TERMINAL_OF_TINSEL_SEED=25
cargo run -- mash krampus yule-cat --fast
cat night20/stocking.json
```

The end of the showdown, then the file:

```
BADGE: Snowball Referee · Stage your first Snowball Showdown
See your collection: terminal-of-tinsel stocking
{
  "version": 1,
  "knowledge": {},
  "captured": {},
  "badges": {
    "snowball-referee": "2026-11-14T21:28:45"
  },
  "seen": {},
  "quizzesPlayed": 0,
  "bestQuizScore": 0,
  "guessesPlayed": 0,
  "bestGuessScore": 0,
  "mashesPlayed": 1
}
```

Now edit the file so that `"version"` is 2, and run both commands again:

```
$ cargo run -- stocking
error: your progress in night20/stocking.json was saved by a newer terminal-of-tinsel (format 2)
hint: upgrade terminal-of-tinsel to use it
$ cargo run -- mash krampus yule-cat --fast
...
WINNER: THE YULE CAT (2-1)
Krampus trudged off into the snow, muttering: “Genuinely good children...”
warning: this session wasn't saved: your progress in night20/stocking.json was saved by a newer terminal-of-tinsel (format 2); upgrade terminal-of-tinsel to use it
```

The same failure is an error for `stocking` and a warning for `mash`, and the file still says 2.

## Try it

- Replace the file's contents with `{ not json` and run a showdown, then `cargo run -- stocking`. Follow the hint.
- Run `cargo run -- stocking --json` in a fresh scratch directory, then list the directory. Looking creates nothing.
- Read `filling_packs_and_then_the_whole_stocking` in `stocking.rs`: it fills the stocking pack by pack and names the badges each step earns.

## 🎄 Festive fact

Arrays of any length have not always been full citizens. Before Rust 1.47, in 2020, the standard library implemented traits such as `Debug` and `PartialEq` only for arrays of up to 32 elements, because each length is a different type and there was no way to write "for every length". Const generics fixed that, and Rust 1.51 let everyone write `[T; N]` code of their own.

## 🕯️ Before morning

Add a badge, "Hat Trick", for finishing three quizzes. Hint: follow "Adding a badge" in `AGENTS.md`. Add it to `BADGES` (and change the 17), award it in the `Kind::Quiz` arm of `game_badges`, and test it in `stocking.rs`. Work out `p.quizzes_played >= 3` into a variable before calling `give`, as `collection_badges` does with its conditions, or the borrow checker will object to reading `p` while it is lent out. One existing test plays three quizzes and will need to hear about your badge.

> 📺 *"There. Whatever you catch from now on stays caught. Tomorrow is the longest night of the year, and we shall spend it counting: the sleeps until Christmas, the phase of the moon, and which door is next."*

[← Night 19](night-19.md) · [Index](README.md) · [Night 21 →](night-21.md)
