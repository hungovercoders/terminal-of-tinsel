# From Terror to Tinsel: Go to Rust

Terminal of Terror is a Go CLI. Terminal of Tinsel is the same idea, rebuilt in Rust. This is the map
between the two, and what changed on the way.

## The stack

| Job | Terror (Go) | Tinsel (Rust) |
|-----|-------------|---------------|
| Commands and flags | Cobra + pflag | clap 4, derive API (`src/cli.rs`) |
| Shell completions | Cobra's built-in generator | clap_complete |
| Embedded data | `//go:embed packs` | `include_dir!` (`src/creatures.rs`) |
| JSON | `encoding/json` with struct tags | serde + serde_json with `#[serde(...)]` |
| Colours and boxes | Lip Gloss + termenv | owo-colors, plus a small box drawer (`src/render.rs`) |
| Wrapping | Lip Gloss widths | textwrap + unicode-width |
| Dates | `time` | jiff (`src/calendar.rs`) |
| Randomness | `math/rand` | a 20-line SplitMix64 (`src/rng.rs`), so the Fact of the Night never changes between releases |
| Errors | `codedError` + `errors.As` | a `TinselError` with a `Code` enum, returned as `Result<T, TinselError>` |
| Tests | `go test` | `cargo test`: unit tests in each module, end-to-end in `tests/cli.rs` with assert_cmd |
| Releases | GoReleaser + Homebrew cask | cargo-dist + Homebrew formula |
| Interactive UI | Bubble Tea | not yet: the next step would be ratatui |

## The same ideas

- **Packs on disk, compiled in.** Both read `packs/<pack>/pack.json` for the order and one JSON file per
  creature, and both bake the data into the binary.
- **`--json` everywhere, coded errors on stderr.** Same codes, same shape:
  `{"error":{"code":"unknown-creature","message":"...","hint":"..."}}`.
- **Plain output when piped.** `NO_COLOR`, `--no-color` or a pipe gives no colour, no emoji and ASCII
  borders.
- **A daily seed from the date.** `YYYYMMDD` as a number, so the Fact of the Night is the same for everyone
  all day.
- **The data is tested.** Every creature needs 5+ facts, 3+ myths, stats from 1 to 10, valid colours and an
  emoji that's two columns wide. Every advent door must point at a real creature.

## What felt different

- **clap's derive API is the command tree.** One annotated struct gives parsing, validation, `--help`,
  possible values for `--pack` and completions. In Cobra those are separate registrations.
- **Errors are values the compiler tracks.** `?` threads `TinselError` through every command, and
  `main` decides once how to print it. There's no `RunE` returning an untyped `error`.
- **Borrowing shapes the code.** The JSON output structs borrow from the vault (`&'a str`) instead of
  copying, which is free at runtime but adds lifetimes to the types.
- **Compile times are longer, the binary is about the same.** A clean release build takes about 30
  seconds; the stripped binary is about 1.7 MB.
- **Dependencies need a closer look.** textwrap's default features pull in the ICU line-breaking
  data. Tinsel turns them off and keeps the dependency tree to about 40 crates.
