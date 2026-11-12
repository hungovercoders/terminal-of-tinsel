# AGENTS.md

Terminal of Tinsel is a Rust CLI about the creatures of Christmas folklore, presented by a fictional
Christmas TV host, Auntie Aerial, on Channel 25's Midwinter Special. It's the Rust sister project of
Terminal of Terror (Go). It uses **clap** (commands), **serde** (data), **ratatui** with its crossterm backend (the
interactive explorer), **owo-colors** and **textwrap** (printed output) and **jiff** (dates).
[docs/parity.md](docs/parity.md) is the checklist of what Terror has and where Tinsel has got to.

## Layout

```
src/main.rs        Entry point: parses args, runs the command, prints JSON or text, maps errors to exit codes
src/cli.rs         The clap command tree
src/creatures.rs   Creature, Pack and advent types, the embedded loader, name lookup and "did you mean"
src/search.rs      Searching names and lore, for the explorer's /
src/explorer/      The interactive explorer: mod.rs (entry point), model.rs (state and keys), view.rs (gallery and pages), intro.rs (Channel 25
                   tuning in), tests.rs (every screen drawn into ratatui's TestBackend)
src/calendar.rs    Sleeps until Christmas, advent doors, the Twelve Days, Yule Lads, seasonal notes
src/tui.rs         What full-screen pages share: the App trait and event loop, the house styles (Ink)
                   and helpers for boxes and columns of styled lines
src/quiz.rs        Quiz questions generated from the creature data, name masking, ranks
src/guess.rs       Guess the Guest: rounds, the frost and its points
src/mash.rs        The Snowball Showdown: the three-round fight simulation
src/games.rs       The quiz and guess screens (ratatui), and their results
src/stocking.rs    Captures and badges: what a session of play earns
src/store.rs       Saved progress: stocking.json in the config dir (TERMINAL_OF_TINSEL_HOME overrides)
src/render.rs      Everything printed: colours, boxes, pages, the tree and the big digits
src/host.rs        Auntie Aerial's lines
src/rng.rs         SplitMix64, pinned so daily facts never change between releases
src/error.rs       TinselError and its stable codes
packs/             Built-in data: <pack>/pack.json, <pack>/<id>.json, <pack>/<id>.txt (portrait), advent.json
tests/cli.rs       End-to-end tests of the binary
```

## Rules

- **Quotes must come from public-domain sources** (such as Dickens). No lines from copyrighted films,
  songs or modern poems, and no copyrighted characters (Rudolph, Frosty, the Grinch).
- Facts must be accurate. Hedge folklore that varies ("in some tellings").
- Avoid emoji that need a variation selector: terminals disagree on their width and break boxes. The tests
  check every creature's emoji is two columns wide.
- Portraits are original ASCII art in printable ASCII only, at most 46 columns by 20 lines, with no
  trailing spaces. The tests check it: a double-width character would misalign the portrait.
- Explorer keys stay consistent with Terror's (h/l and arrows, j/k, tab and 1-7, r, /, g, ?, q). Screens
  that take over the terminal call `require_terminal` first, and fail with the `no-terminal` code.
- Every random choice goes through `Rng` (`src/rng.rs`), seeded by `TERMINAL_OF_TINSEL_SEED`, so a run
  can be repeated. Don't add another source of randomness. Set the seed when testing.
- Set `TERMINAL_OF_TINSEL_HOME` to a scratch directory when testing, so real progress isn't touched. The
  end-to-end tests do. Saving must never stop a game: `record` warns and carries on.
- Game text must read the same for one creature or several (the Yule Lads are plural), so narration is
  in the past tense.
- Never push a `v*` tag without the maintainer's go-ahead: it publishes a release.
- Keep dependencies few, and check what their default features pull in.
- Return `Result<_, TinselError>` with a message a player can act on; don't panic on player input.
- British English, no em dashes.

## Build and test

```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
DUMP=1 cargo test dump_ -- --nocapture   # print sample explorer and game screens
```

## Common tasks

- **Adding a creature:** add `packs/<pack>/<id>.json` with every field (`film` only where there's a
  classic screen outing, with a `releaseDate` you have checked), 5+ accurate facts and 3+ myths, draw `packs/<pack>/<id>.txt`, add the id to
  `order` in that pack's `pack.json`, then `cargo test`. Update the roster table and badge count in the README.
- **Adding a pack:** create `packs/<pack>/pack.json` and add the id to `PACK_ORDER` in `src/creatures.rs`.
- **Changing the explorer:** state and keys go in `src/explorer/model.rs`, drawing in `view.rs`. Pages
  are built as `Vec<Line>` by plain functions of the model, so test them in `tests.rs` by pressing keys
  and reading the screen back, at small and large sizes. Then try it in a real terminal.
- **Adding a badge:** add it to `BADGES` in `src/stocking.rs` (the array's length is in its type), award
  it in `apply`, test it there, and add it to the README's badge table. Its emoji must be two columns wide.
- **Adding a command:** add a variant to `Command` in `src/cli.rs`, handle it in `run` in `src/main.rs`
  (with `--json` output), draw it in `src/render.rs`, and add it to the smoke test in `ci.yml` and to
  `every_command_runs` in `tests/cli.rs`.
