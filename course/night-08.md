# Night 8 · Check It Twice

> 📺 *"Good evening. A colleague of mine in the red suit keeps a list and, famously, checks it twice. He has the right idea. Tonight we write down what the programme is supposed to do, in a form the computer can check for us, every time, without getting bored or stopping for a mince pie."*

**Tonight you'll learn**
- `cargo test`, `#[test]` and the `assert!` family
- Unit tests in a `#[cfg(test)]` module beside the code
- Data tests: one loop that holds every creature to the same standard
- End-to-end tests that run the real program, with `assert_cmd`
- `cargo clippy` and `cargo fmt`

**Where we are:** `list`, `random` and `creature <name>`, with errors and exit codes. Nothing checks any of it except us, by eye.

## A first test

A test is a function marked `#[test]`. It passes if it returns and fails if it panics, and the `assert!` macros panic when their condition doesn't hold. Add this to the end of `src/rng.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = Rng::new(20261224);
        let mut b = Rng::new(20261224);
        for _ in 0..10 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn sequence_is_pinned() {
        // If this changes, every seeded run changes with it.
        assert_eq!(Rng::new(0).next_u64(), 0xE220_A839_7B1D_CDAF);
    }

    #[test]
    fn below_stays_in_range() {
        let mut r = Rng::new(7);
        for _ in 0..1000 {
            assert!(r.below(13) < 13);
        }
    }
}
```

**`#[cfg(test)]`** means "only compile this when testing". The module and everything in it are left out of the program you ship.

**`mod tests { use super::*; ... }`** is a module declared in place, with braces instead of a file. `super` is the module it sits inside, so `use super::*` brings in everything from `rng`, private items included. A child module can see its parent's private things, which is why unit tests live in the same file as the code.

The three tests each say something we care about:

- The same seed gives the same sequence. `assert_eq!` compares two values and, on failure, prints both.
- The sequence is *pinned*: seed 0 must give this exact number. If anybody "improves" the generator, this fails, and it should, because every seeded run would change.
- `below(13)` stays below 13, a thousand times over.

Run them:

```bash
cargo test
```

```
running 3 tests
test rng::tests::below_stays_in_range ... ok
test rng::tests::same_seed_same_sequence ... ok
test rng::tests::sequence_is_pinned ... ok
```

## Testing the lookup

At the end of `src/creatures.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_folds_case_accents_and_punctuation() {
        assert_eq!(normalize("  The Yule-Cat "), "yule cat");
        assert_eq!(normalize("Grýla"), "gryla");
        assert_eq!(normalize("Marley's Ghost"), "marley s ghost");
        assert_eq!(normalize("!!!"), "");
    }

    #[test]
    fn levenshtein_counts_edits() {
        assert_eq!(levenshtein("krampus", "krampus"), 0);
        assert_eq!(levenshtein("krampsu", "krampus"), 2);
        assert_eq!(levenshtein("", "cat"), 3);
    }

    #[test]
    fn find_takes_ids_names_aliases_and_parts() {
        let v = Vault::load();
        for (query, id) in [
            ("krampus", "krampus"),
            ("The Yule Cat", "yule-cat"),
            ("jolakotturinn", "yule-cat"),
            ("GRÝLA", "gryla"),
            ("nisse", "tomte"),
            ("ghost future", "ghost-yet-to-come"),
        ] {
            assert_eq!(v.find(query).unwrap().id, id, "{query}");
        }
    }

    #[test]
    fn find_explains_its_failures() {
        let v = Vault::load();
        let e = v.find("ghost").unwrap_err();
        assert_eq!(e.code, Code::AmbiguousCreature);
        let e = v.find("krampsu").unwrap_err();
        assert_eq!(e.code, Code::UnknownCreature);
        assert!(e.message.contains("Did you mean Krampus?"), "{e}");
        assert!(v.find("  ").is_err());
    }

    // ── Data tests: every built-in creature is held to the full set. ──

    #[test]
    fn every_pack_directory_is_in_the_pack_order() {
        for dir in PACKS_DIR.dirs() {
            let name = dir.path().to_str().unwrap();
            assert!(
                PACK_ORDER.contains(&name),
                "pack {name} isn't in PACK_ORDER"
            );
        }
    }

    #[test]
    fn every_creature_file_is_listed_in_its_pack() {
        let v = Vault::load();
        for dir in PACKS_DIR.dirs() {
            for f in dir.files() {
                let path = f.path();
                let stem = path.file_stem().unwrap().to_str().unwrap();
                if path.extension().unwrap() == "json" && stem != "pack" {
                    assert!(
                        v.creatures.iter().any(|c| c.id == stem),
                        "{} isn't in its pack's order",
                        path.display()
                    );
                }
            }
        }
    }

    #[test]
    fn every_creature_is_complete() {
        let v = Vault::load();
        assert!(v.creatures.len() >= 20);
        for c in &v.creatures {
            let id = &c.id;
            assert!(!c.name.is_empty(), "{id}: name");
            assert!(!c.emoji.is_empty(), "{id}: emoji");
            assert!(!c.description.is_empty(), "{id}: description");
            assert!(!c.origin.is_empty(), "{id}: origin");
            assert!(!c.legend.is_empty(), "{id}: legend");
            assert!(!c.host_intro.is_empty(), "{id}: hostIntro");
            assert!(c.facts.len() >= 5, "{id}: needs 5+ facts");
        }
    }

    #[test]
    fn every_creature_can_be_found_by_its_own_names() {
        let v = Vault::load();
        for c in &v.creatures {
            assert_eq!(v.find(&c.id).unwrap().id, c.id);
            assert_eq!(v.find(&c.name).unwrap().id, c.id);
            for alias in &c.aliases {
                assert_eq!(v.find(alias).unwrap().id, c.id, "{alias}");
            }
        }
    }
}
```

The first four are unit tests of last night's work. Notice that they call `levenshtein`, which is private: the tests are inside the module.

`find_takes_ids_names_aliases_and_parts` is a **table test**: a list of (query, expected id) pairs and one loop. Adding a case is adding a line. The third argument to `assert_eq!` is a message, shown on failure, so you know *which* query broke. `.unwrap()` takes the value out of an `Ok` and panics on an `Err`, which in a test is exactly right, and `unwrap_err()` is its opposite.

## Data tests

The last four tests don't test code. They test the *data*:

- every folder under `packs/` is named in `PACK_ORDER`, so a new pack can't be silently ignored
- every creature file is listed in its pack's `order`
- every creature has a name, an emoji, a description, an origin, a legend, a host line and at least five facts
- every creature can be found by its own id, name and each of its aliases

Remember Night 5: `#[serde(default)]` means a missing field loads as an empty one. That's kind to people writing their own packs, and it would let a typo in ours through. So the parser stays lenient and these tests are strict. Add a 23rd creature with four facts and `cargo test` will name it.

The last test is the one that earns its keep. An alias that also matches another creature would make `find` ambiguous, and nobody would notice until a player typed it. With 22 creatures and 60-odd aliases, a loop notices.

Add one more, at the end of `src/error.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_appends_the_hint() {
        let e = TinselError::new(Code::Usage, "bad date").with_hint("use YYYY-MM-DD");
        assert_eq!(e.to_string(), "bad date; use YYYY-MM-DD");
    }
}
```

`to_string()` exists on anything that implements `Display`, so this checks the `impl` we wrote by hand.

## End-to-end tests

Unit tests call functions. An **end-to-end** test runs the compiled program, as a player would, and looks at what comes out. Two crates help, and they're only needed for testing, so they go in a separate section of `Cargo.toml`:

```bash
cargo add --dev assert_cmd@2.2.2 predicates@3.1.4
```

```toml
[dev-dependencies]
assert_cmd = "2.2.2"
predicates = "3.1.4"
```

Create a new folder, `tests`, beside `src`, and in it `tests/cli.rs`:

```rust
//! End-to-end tests: run the real binary the way a player or a script would.

use assert_cmd::Command;
use predicates::prelude::*;

fn tinsel() -> Command {
    let mut cmd = Command::cargo_bin("terminal-of-tinsel").unwrap();
    cmd.env("TERMINAL_OF_TINSEL_SEED", "25");
    cmd
}

#[test]
fn every_command_runs() {
    for args in [
        vec![],
        vec!["list"],
        vec!["creature", "krampus"],
        vec!["creature", "mari", "lwyd"],
        vec!["random"],
        vec!["--help"],
        vec!["--version"],
    ] {
        tinsel().args(&args).assert().success();
    }
}

#[test]
fn list_groups_the_creatures_by_pack() {
    tinsel()
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("NORDIC YULE"))
        .stdout(predicate::str::contains("The Yule Cat"));
}

#[test]
fn the_same_seed_tells_the_same_fact() {
    let run = || tinsel().arg("random").output().unwrap().stdout;
    assert_eq!(run(), run());
}

#[test]
fn an_unknown_creature_fails_with_a_suggestion() {
    tinsel()
        .args(["creature", "krampsu"])
        .assert()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("Did you mean Krampus?"));
}

#[test]
fn a_misused_command_exits_with_two() {
    tinsel().arg("creature").assert().code(2);
}
```

Files in `tests/` are **integration tests**. Each is compiled as its own little crate, outside the program, so it can't see private functions. It doesn't need to: it runs the binary.

`Command::cargo_bin("terminal-of-tinsel")` finds the program Cargo built. Our helper `tinsel()` sets the seed on every command, so nothing in these tests depends on chance. `.assert()` runs it and returns something to make claims about: `.success()` for exit code 0, `.code(1)` for a particular one, and `.stdout(...)` or `.stderr(...)` with a **predicate**, a small rule such as "contains this text" or "is empty".

Look at what `an_unknown_creature_fails_with_a_suggestion` pins down: the exit code, that the suggestion appears, that it appears on stderr, and that stdout stays empty. That's the whole contract a script relies on.

`the_same_seed_tells_the_same_fact` uses a closure, `run`, to run the command twice and compare the raw output.

## Run everything

```bash
cargo test
```

```
     Running unittests src/main.rs (target/debug/deps/terminal_of_tinsel-f5e21fecf30c50d6)
running 12 tests
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/cli.rs (target/debug/deps/cli-b0a0c6ad0575d675)
running 5 tests
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

Two groups: the unit tests inside the program, then `tests/cli.rs`. To run a few, give part of a name: `cargo test find` runs every test with "find" in its name.

## Two more tools

**Clippy** is Rust's linter: a few hundred checks for code that compiles but could be clearer, faster or less surprising.

```bash
cargo clippy --all-targets -- -D warnings
```

`--all-targets` includes the tests, and `-D warnings` turns every warning into an error, so the command fails if anything is found. If you've typed the lessons as written, it prints nothing but "Finished". When it does object, it shows the line, explains why and usually offers the replacement.

**rustfmt** formats code in the one standard style:

```bash
cargo fmt
```

It rewrites your files in place. `cargo fmt --check` changes nothing and fails if anything *would* change, which is the form to use in automation. Every code block in this course has been through it, so if yours looks different after formatting, compare it with the lesson.

From tonight, the routine before each commit is the one the finished project uses:

```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
git add .
git commit -m "test: unit, data and end-to-end tests"
```

## Try it

- Break something on purpose: in `fold`, change `'ý' | 'ÿ' => 'y'` to give `'i'`, and run `cargo test`. A test fails, and the failure prints the file and line and both values, left and right. Mend it.
- Delete two facts from `packs/nordic/lussi.json`, leaving her with four. The data test names her. Restore the file with `git checkout packs`.
- Write `let x = vec.len() == 0;` somewhere (with any `Vec`) and run clippy. It suggests `is_empty()`.
- Run `cargo test -- --nocapture`. Normally the output of passing tests is hidden; this shows it.

## 🎄 Festive fact

`cargo test` runs the tests inside each test binary in parallel, on several threads, which is why they finish in a different order each time you run them. It's also why tests mustn't share anything they change, such as a file or an environment variable. If you ever need them one at a time, `cargo test -- --test-threads=1` does it. The finished program's end-to-end tests give each test that saves progress its own scratch folder for exactly this reason: see [`tests/cli.rs`](../tests/cli.rs).

## 🕯️ Before morning

Write a data test, `every_id_is_a_slug`, that checks each creature's id uses only lowercase letters, digits and dashes.

Hint: `c.id.chars().all(|ch| ...)` with `ch.is_ascii_lowercase()`, `ch.is_ascii_digit()` and `ch == '-'`. The finished program has this as a function, `is_slug`, in [`src/creatures.rs`](../src/creatures.rs), because on Night 22 it has to judge ids written by strangers.

> 📺 *"Checked twice, and nobody on the naughty list but a typo in Leeds. Tomorrow night we get the decorations down from the loft. I do hope you like green and gold."*

[← Night 7](night-07.md) · [Index](README.md) · [Night 9 →](night-09.md)
