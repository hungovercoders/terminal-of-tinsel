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
| Interactive UI | Bubble Tea + Lip Gloss | ratatui with its crossterm backend (`src/explorer/`) |
| Terminal check | `charmbracelet/x/term` | `std::io::IsTerminal`, in the standard library since Rust 1.70 |

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
  data. Tinsel turns them off and kept the dependency tree to about 40 crates, until the explorer arrived
  (see below).

## Phase 1: the explorer (Bubble Tea to ratatui)

Terror's explorer is a Bubble Tea program. Tinsel's is the same screens and the same keys on ratatui.
The two libraries disagree about almost everything, which makes this the most interesting port so far.

- **Who owns the loop.** Bubble Tea is the Elm architecture: you hand it a model with `Init`, `Update`
  and `View`, and it runs the loop, calls you with messages and draws the string you return. ratatui is a
  drawing library and nothing more. The loop in `src/explorer/mod.rs` is ours: draw, wait for a key or a
  tick, update, go round again. It's 30 lines, and nothing is hidden in it.
- **Ticks are a timeout, not a message.** Terror asks for animation frames with `tea.Tick`, which sends a
  `tickMsg` later, and has to remember whether a tick is already in flight. In Tinsel the loop polls for
  a key with a timeout: if the screen is animated (`Model::tick_every` returns `Some`), the timeout is
  the time to the next frame; if it's still, the loop blocks on the keyboard and uses no CPU at all.
- **Update mutates.** Bubble Tea's `Update` takes the model by value and returns a new one, so Terror's
  methods are a mix of value receivers and pointer receivers that have to be kept straight. In Rust
  `on_key(&mut self, ...)` changes the model in place, and the borrow checker guarantees nothing is
  drawing from it at the same time.
- **Strings against cells.** Lip Gloss builds a screen as one big string: style a piece, measure it,
  join pieces side by side. ratatui draws styled cells into a buffer through widgets and layout
  rectangles. Tinsel meets it halfway: each page is built as a `Vec<Line>` of styled spans by plain
  functions (`view.rs`), with small helpers for a box and for two columns side by side, and only the last
  step lays those lines on the frame. That keeps a page a pure function of the model, as it is in Go.
- **No global colour switch.** Lip Gloss has a process-wide colour profile that Terror flips for plain
  mode. ratatui has no globals, so the model carries `colour: bool` and every style goes through one
  function. `NO_COLOR` therefore works inside Tinsel's explorer too, which it doesn't in Terror's.
- **Keys are an enum.** Bubble Tea hands over keys as strings (`"shift+tab"`, `"pgdown"`), so a typo in
  a `case` compiles and never matches. crossterm's `KeyCode` is an enum: `KeyCode::BackTab` either
  exists or the build fails, and `KeyCode::Char('1'..='9')` is a pattern.
- **Testing a screen.** Terror calls `View()` and searches the string. Tinsel draws into ratatui's
  `TestBackend`, an in-memory terminal, and reads the cells back, so the tests see exactly what a real
  terminal would, including what got clipped at the edge. `tests.rs` draws every page of every creature
  at six sizes, down to one cell by one.
- **Leaving the terminal tidy.** Both libraries switch to the alternate screen and raw mode.
  `ratatui::try_init` also installs a panic hook that restores the terminal first, so a bug in the
  explorer prints its message on a working terminal instead of a garbled one.
- **Stateless snow.** Terror keeps the intro's static in the model and regenerates it on each tick.
  Tinsel's snow is a pure function of `(seed, column, row, frame)`, so there is no flake state to keep,
  it survives a resize, and a seeded run draws the same snow every time.
- **Embedding text files.** `//go:embed packs` picks up `<id>.txt` portraits for free, and so does
  `include_dir!`. Both normalise `\r\n`, because a checkout on Windows can add carriage returns.
- **What it cost.** ratatui 0.30 needs Rust 1.88, exactly Tinsel's minimum. With default features off
  and only the crossterm backend, it took the tree from about 40 crates to about 85 and the stripped release
  binary from about 1.7 MB to 2.1 MB.

## Phase 2: the games

The quiz, Guess the Guest and the Snowball Showdown are ports of Terror's Midnight Quiz, Guess the
Monster and Monster Mash: the same rules and numbers, with Christmas words.

- **A trait where Go has an interface, but checked at the call.** Terror's three Bubble Tea programs
  each implement `tea.Model`. Tinsel's explorer, quiz and guess each implement `App` (`src/tui.rs`), and
  one `tui::run(&mut impl App)` drives them all. The difference is `impl App`: the loop is compiled once
  per screen with the calls resolved, where Go dispatches through the interface at run time.
- **A question kind is an enum, not a string.** Terror's `Question.Kind` is `"fact"`, `"myth"` and so
  on, and the "no kind over a third" rule counts them in a `map[string]int`. In Tinsel `Kind` is an enum
  that serde writes as the same lowercase strings, and the count is an array indexed by the variant. A
  misspelt kind can't exist.
- **`Option<usize>` for "not answered yet".** Terror marks an unanswered round with `Chosen: -1`.
  Tinsel's `chosen: Option<usize>` makes the compiler ask what happens before the guess, everywhere.
- **Shuffling by hand.** Go has `rand.Shuffle` and `rand.Perm`. Tinsel keeps its own SplitMix64 so that
  seeded games never change between releases, which meant writing an eight-line Fisher-Yates shuffle.
  The rand crate would have done it, at the cost of a sequence that can change with its version.
- **No regex.** Terror hides a monster's name in a quiz clue with `regexp`. Go's is in the standard
  library; Rust's is a crate, and a big one. Tinsel's `mask` walks the characters instead: 50 lines,
  and it folds accents too, so "Grýla" is hidden however the fact spells it.
- **Borrowed rounds.** A guess round holds `&Creature`, borrowed from the vault for the length of the
  game, where Go copies the whole `Monster` struct into each round. The lifetime (`Round<'a>`) is the
  price, and it spreads to everything that holds a round.
- **Sleeping is the same.** The showdown's dramatic pauses are `time.Sleep` in Go and
  `std::thread::sleep` in Rust, skipped in both when stdout isn't a terminal.
- **Grammar is a data problem.** Terror's fight lines are in the present tense ("hurls"), which reads
  wrongly for a plural name. Tinsel has the Yule Lads, so its lines are in the past tense, and the
  loser's parting mutter is quoted rather than woven into the sentence.
