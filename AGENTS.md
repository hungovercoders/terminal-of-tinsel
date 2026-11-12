# AGENTS.md

Terminal of Tinsel is a Rust CLI about the creatures of Christmas folklore, presented by a fictional
Christmas TV host, Auntie Aerial, on Channel 25's Midwinter Special. It's the Rust sister project of
Terminal of Terror (Go). It uses **clap** (commands), **serde** (data), **owo-colors** and **textwrap**
(drawing) and **jiff** (dates).

## Layout

```
src/main.rs        Entry point: parses args, runs the command, prints JSON or text, maps errors to exit codes
src/cli.rs         The clap command tree
src/creatures.rs   Creature, Pack and advent types, the embedded loader, name search and "did you mean"
src/calendar.rs    Sleeps until Christmas, advent doors, the Twelve Days, Yule Lads, seasonal notes
src/render.rs      Everything drawn: colours, boxes, pages, the tree and the big digits
src/host.rs        Auntie Aerial's lines
src/rng.rs         SplitMix64, pinned so daily facts never change between releases
src/error.rs       TinselError and its stable codes
packs/             Built-in data: <pack>/pack.json, <pack>/<id>.json, advent.json
tests/cli.rs       End-to-end tests of the binary
```

## Rules

- **Quotes must come from public-domain sources** (such as Dickens). No lines from copyrighted films,
  songs or modern poems, and no copyrighted characters (Rudolph, Frosty, the Grinch).
- Facts must be accurate. Hedge folklore that varies ("in some tellings").
- Avoid emoji that need a variation selector: terminals disagree on their width and break boxes. The tests
  check every creature's emoji is two columns wide.
- Never push a `v*` tag without the maintainer's go-ahead: it publishes a release.
- Keep dependencies few, and check what their default features pull in.
- Return `Result<_, TinselError>` with a message a player can act on; don't panic on player input.
- British English, no em dashes.

## Build and test

```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
```

## Common tasks

- **Adding a creature:** add `packs/<pack>/<id>.json` with every field (`film` only where there's a
  classic screen outing), 5+ accurate facts and 3+ myths, add the id to `order` in that pack's
  `pack.json`, then `cargo test`. Update the roster table and badge count in the README.
- **Adding a pack:** create `packs/<pack>/pack.json` and add the id to `PACK_ORDER` in `src/creatures.rs`.
- **Adding a command:** add a variant to `Command` in `src/cli.rs`, handle it in `run` in `src/main.rs`
  (with `--json` output), draw it in `src/render.rs`, and add it to the smoke test in `ci.yml` and to
  `every_command_runs` in `tests/cli.rs`.
