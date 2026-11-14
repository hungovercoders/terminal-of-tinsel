# Night 23 · The Night Before the Night Before

> 📺 *"Two sleeps. The presents are wrapped, more or less, and tonight is for the jobs nobody sees: the labels, the instructions, and a second pair of eyes on the lot. We shall teach the shell to finish your sentences, write a manual nobody asked for, and have three computers check our work while we sleep."*

**Tonight you'll learn**
- Tab completion that asks the program what to offer
- `ArgValueCandidates`: attaching a list of names to an argument
- Why one dependency is pinned with `=`
- Man pages from the command tree with clap_mangen
- Generated files that are committed, and a test that fails when they go stale
- CI on three operating systems, and a job for the minimum Rust version

**Where we are:** the program takes community packs and speaks JSON. Tonight nothing new appears on screen. This is the work around the program.

## Completion that asks the program

A shell completes a command by running a small script when you press tab. The older way to make that script is to generate it from the command tree: it knows every subcommand and flag, and nothing else. It could never offer `sock-goblin`, because last night's community pack didn't exist when the script was written.

So the script in this project knows almost nothing. Here is the whole of [`completions/terminal-of-tinsel.fish`](../completions/terminal-of-tinsel.fish):

```
complete --keep-order --exclusive --command terminal-of-tinsel --arguments "(COMPLETE=fish terminal-of-tinsel -- (commandline --current-process --tokenize --cut-at-cursor) (commandline --current-token))"
```

When you press tab, fish runs the program itself with the environment variable `COMPLETE=fish` and the words typed so far, and offers whatever comes back. You can play the shell's part by hand:

```bash
COMPLETE=fish terminal-of-tinsel -- terminal-of-tinsel creature kra
```

```
krampus	Krampus
krampusz	Krampus
```

The other end of that conversation is the first statement in `main`, in [`src/main.rs`](../src/main.rs):

```rust
fn main() -> ExitCode {
    // When the shell asks for completions (COMPLETE=bash and so on), answer
    // and leave: nothing else runs.
    clap_complete::CompleteEnv::with_factory(Cli::command).complete();
    // ...
```

If `COMPLETE` is set, `complete()` prints the candidates and exits the process. If it isn't, it returns and the program carries on as usual. `Cli::command` is the function clap's derive wrote for you on Night 2; it builds the command tree, so commands and flags complete with no further work.

Creature names need telling. In [`src/cli.rs`](../src/cli.rs) the name argument carries one extra attribute:

```rust
    Creature {
        /// The creature's name, nickname or part of one, e.g. krampus, "yule cat", nisse
        #[arg(num_args = 0.., value_name = "NAME", add = ArgValueCandidates::new(complete::creatures))]
        name: Vec<String>,
        // ...
```

`ArgValueCandidates::new` takes a function that returns the candidates. It is called while the shell waits, so it runs against the vault as it is now, community packs included. The function is in [`src/complete.rs`](../src/complete.rs):

```rust
pub fn creatures() -> Vec<CompletionCandidate> {
    let mut seen = Vec::new();
    let mut out = Vec::new();
    for c in &vault().creatures {
        let keys = std::iter::once(c.id.clone()).chain(c.aliases.iter().map(|a| a.to_lowercase()));
        for key in keys.filter(|k| !k.contains(' ')) {
            if !seen.contains(&key) {
                seen.push(key.clone());
                out.push(CompletionCandidate::new(key).help(Some(c.name.clone().into())));
            }
        }
    }
    out
}
```

`std::iter::once(x)` is an iterator of one item, and `chain` joins it to the aliases, so the loop sees the id first and then each nickname. Names of several words are left out, because a shell would need them quoted and the id always works. `.help(...)` is the text shown beside each candidate. The module's `vault()` helper throws away the warnings about broken packs: a completion must never print to the terminal, or the warning lands in the middle of the line you are typing.

`terminal-of-tinsel completions <shell>` prints the registration script for a shell, which is what a player puts in their startup file.

## A pin, and why

Look at how that dependency is declared in [`Cargo.toml`](../Cargo.toml):

```toml
clap = { version = "4.6.7", features = ["derive", "wrap_help"] }
# Pinned exactly: dynamic completion is behind an unstable feature, so a
# newer release may change it. Dependabot proposes each bump.
clap_complete = { version = "=4.6.11", features = ["unstable-dynamic"] }
```

In Cargo, `"4.6.7"` does not mean exactly 4.6.7. It means "4.6.7 or any later release that semantic versioning calls compatible", which is anything below 5.0.0. That is normally what you want: bug fixes arrive without asking.

`CompleteEnv` lives behind a feature called `unstable-dynamic`, and the name is a warning from its authors: this part of the crate may change in any release, even a patch release. The promise that makes `"4.6.11"` safe doesn't cover it. `"=4.6.11"` means that version and no other. An upgrade then only happens as a pull request from Dependabot, where the tests run and any change to the generated scripts shows up in the diff.

## The manual

A man page is the manual a Unix user expects from `man terminal-of-tinsel`. The **clap_mangen** crate writes them from the same command tree that makes `--help`. It is listed under `[dev-dependencies]`, so it is compiled for tests and never reaches the shipped binary. The generator is two lines, in [`src/shelldocs.rs`](../src/shelldocs.rs):

```rust
fn write_man_pages(dir: &Path) {
    fs::create_dir_all(dir).unwrap();
    clap_mangen::generate_to(Cli::command(), dir).unwrap();
}
```

That module only exists in test builds: `main.rs` declares it as `#[cfg(test)] mod shelldocs;`. The `unwrap()` calls are fine for the same reason. This is test code, and a panic is how a test fails.

The results are in [`man/`](../man/), one page for the program and one for each command, and in [`completions/`](../completions/), one script for each shell. Read one with `man ./man/terminal-of-tinsel-countdown.1`.

## Generated, committed and checked

Generated files are usually kept out of git, because they can be made again. These are committed, for a practical reason given at the top of `shelldocs.rs`: cargo-dist, which builds the releases (tomorrow's subject), copies `man/` and `completions/` into each archive but has no step that could generate them first. They have to be in the repository already when the release is built.

A committed generated file can go stale. Somebody changes a flag's help text, forgets the man page, and the manual is now wrong. So there is a test:

```rust
#[test]
fn shell_docs_are_up_to_date() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    // ...
    for (name, generate) in generators {
        let committed = root.join(name);
        if std::env::var_os("UPDATE_SHELLDOCS").is_some() {
            let _ = fs::remove_dir_all(&committed);
            generate(&committed);
            continue;
        }
        let fresh_dir = scratch.join(name);
        generate(&fresh_dir);
        let (fresh, on_disk) = (files_in(&fresh_dir), files_in(&committed));
        // ...
        for (file, text) in &fresh {
            assert!(
                on_disk[file] == *text,
                "{name}/{file} is out of date: run UPDATE_SHELLDOCS=1 cargo test shell_docs"
            );
        }
    }
    let _ = fs::remove_dir_all(&scratch);
}
```

Without the variable, the test generates everything afresh into a scratch directory and compares it, file by file, with what is committed. Any difference fails the test, and the message is the command that fixes it. With `UPDATE_SHELLDOCS` set, the same test rewrites the committed files instead. One function is both the check and the tool, so the two can't drift apart.

`env!("CARGO_MANIFEST_DIR")` is read at compile time and gives the folder holding `Cargo.toml`, which is how a test finds the repository's files wherever it is run from. `generators` is an array of `(&str, fn(&Path))` pairs: a function name without brackets is a value you can store and call later.

The man pages carry the version number, so a release has to regenerate them. That is one of tomorrow's steps. [`Taskfile.yml`](../Taskfile.yml) gives the command a shorter name, `task shelldocs`, beside `task check`, which runs the formatter, clippy and every test.

## Three computers and an old compiler

All of this would be good intentions without something to enforce it. [`.github/workflows/ci.yml`](../.github/workflows/ci.yml) runs on every pull request and every push to `main`, in three jobs.

**lint** runs `cargo fmt --all --check` and `cargo clippy --workspace --all-targets --locked -- -D warnings`. `-D warnings` turns every clippy warning into an error, so nothing can be waved through.

**test** runs on three operating systems:

```
  test:
    name: Test (${{ matrix.os }})
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
```

A matrix runs the same job once per value. `fail-fast: false` lets all three finish when one fails, so you learn whether a failure belongs to Windows alone. The differences are real: the config directory from Night 20 is in a different place on each system, and a Windows checkout can add carriage returns to text files, which is why `files_in` in `shelldocs.rs` strips `\r` before comparing.

After `cargo test --workspace --locked`, the job runs a smoke test: a shell script that runs the built binary through each of its commands, tonight's completion among them.

```
          COMPLETE=fish $bin -- terminal-of-tinsel creature kra | grep -q '^krampus'
```

`--locked` appears throughout. It tells cargo to use exactly the versions in `Cargo.lock` and to fail if the lock file would need changing, so CI tests the same dependencies you did.

**msrv** is the third job:

```
  msrv:
    name: Minimum Rust version
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
      - uses: dtolnay/rust-toolchain@1.88
      - uses: Swatinem/rust-cache@v2
      - run: cargo check --workspace --locked
```

`Cargo.toml` says `rust-version = "1.88"`: a promise that the program builds with Rust 1.88 or newer. The other jobs use the latest stable compiler, so nothing in them would notice if you used a feature from a later release, or a dependency raised its own minimum. This job installs 1.88 itself and checks that the promise still holds. 1.88 isn't arbitrary: it is what ratatui 0.30 needs.

## Try it

- Add completion to your own shell: for zsh, put `source <(terminal-of-tinsel completions zsh)` in `~/.zshrc`, open a new shell and type `terminal-of-tinsel creature kra` then tab.
- Run `COMPLETE=fish cargo run -q -- -- terminal-of-tinsel quiz --pack ''` to see every pack id, yours included.
- Run `cargo test shell_docs`, then `task check` if you have [Task](https://taskfile.dev) installed, or the three commands it runs if you haven't.
- Read the smoke test in `ci.yml` from top to bottom. It is a tour of the whole program.

## 🎄 Festive fact

Cargo's `rust-version` field does more than document. Edition 2024, which this project uses, makes dependency resolver 3 the default, and that resolver takes `rust-version` into account: when it chooses versions for `Cargo.lock`, it prefers releases of your dependencies that say they support your minimum Rust, and falls back to newer ones only when there is no such release.

## 🕯️ Before morning

Make the staleness test fail, then mend it. On a branch of your own, change the help text of `--fast` on `mash` in `src/cli.rs` (the `///` comment above the field) and run `cargo test`. Read the failure. Then run the command it names, and look at what changed with `git diff --stat`.

*Hint:* a doc comment on a clap field is its help text, so one edit reaches `--help`, a man page and nothing else. When you've seen it work, `git checkout .` puts everything back.

> 📺 *"Labels on, instructions folded, and three computers nodding along. Nearly there, my dears. Tomorrow is Christmas Eve: we wrap the whole thing up, put a number on it and send it out into the snow."*

[← Night 22](night-22.md) · [Index](README.md) · [Night 24 →](night-24.md)
