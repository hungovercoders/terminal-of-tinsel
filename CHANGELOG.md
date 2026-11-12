# Changelog

## [Unreleased]

### Added

- **The Midwinter Special.** `creature` with no name opens an interactive explorer: Channel 25 tunes in
  through static and snow, then a gallery of every creature and a page each, with numbered sections
  (`tab` or `1`-`7`), `h`/`l` and the arrows to move between creatures, `j`/`k` to scroll, `r` to reveal
  the Myth or Fact? verdicts, `/` to search names and lore, `g` or `esc` for the gallery, `?` for help and
  `q` to quit.
- `creature --all` opens on the gallery, `--no-intro` skips the opening and `creature <name> --explore`
  opens the explorer on one creature.
- An original ASCII portrait for every creature (`packs/<pack>/<id>.txt`), shown beside the text on wide
  screens and included in `--json` as `ascii`.
- The spirits of *A Christmas Carol* are drawn as engravings: sepia ink and drifting snow.
- `creature --json` with no name prints every creature.
- A `no-terminal` error code when a screen that needs a terminal is run from a script, a pipe or CI, with
  a hint to use `--json`.
- **The Christmas Cracker Quiz.** `quiz [creature]` asks multiple-choice questions built from the creature
  data (`-n` sets how many), explains every answer and ends with a rank from Stocking Filler to Star of
  the Tree.
- **Guess the Guest.** `guess` hides a portrait behind a frosted window: `space` clears more frost and
  brings clues, at the cost of points (`-n` sets the rounds).
- **The Snowball Showdown.** `mash [creature] [creature]` stages three rounds between two creatures,
  decided by their stats and a die; `--fast` skips the pauses.
- All three games take `--json`, and `TERMINAL_OF_TINSEL_SEED` now fixes every random choice. A seed that
  isn't a whole number is ignored with a warning.
- **The Stocking.** Three correct answers about a creature, in the quiz or Guess the Guest, put it in
  your stocking. `stocking` shows what you've caught, 17 badges and your records; `stocking reset` (`-y`
  to skip the question) starts again. The games and the explorer record what was seen and won.
- Progress is saved as `stocking.json` in your config directory, or in `TERMINAL_OF_TINSEL_HOME` if set.
  A broken save file never stops a game: it warns and carries on.
- A `store` error code when saved progress can't be read or written.
- **Tonight's double bill.** `tonight` prints a cinema ticket for the night's two features, the same all
  day and different tomorrow; `--shuffle` deals another, `--date` looks ahead. In advent, the guest
  behind the night's door tops the bill.
- `countdown` shows tonight's moon and the next special night: the nearer of the next feast of the
  season and the next film anniversary. `countdown --json` gains `moon` and `nextSpecialNight`.
- New notes: the winter solstice, Christmas Day, a full moon (the Cold Moon in December), film
  anniversaries, and the sleeps until Christmas when there are 60 or fewer.
- Films can carry `releaseDate`, `studio`, `makeup` and `silent`. *Scrooge* (1951) has its release date.
- Auntie Aerial has sign-off lines, and a special greeting on Christmas Eve and Christmas Day.

### Changed

- "Did you mean" now offers up to three names, nearest first, and matches ids as well as names.
- `--no-color` and `NO_COLOR` also switch the explorer to the terminal's own colours.

## [0.1.0] - Unreleased

The first broadcast of Channel 25's Midwinter Special.

### Added

- 14 creatures of Christmas folklore in four packs: St Nicholas's Companions, Nordic Yule, Midwinter Europe
  and A Christmas Carol.
- `list`, `creature` (with `--reveal`), `random` (with `--daily` and `--date`), `countdown`, `advent` and
  `completions`.
- A 24-door advent calendar, the Twelve Days of Christmas, the Yule Lads' comings and goings, and notes for
  the season's special nights.
- `--json` on every command with stable error codes, `--pack` filtering, and plain output for pipes and
  `NO_COLOR`.
- Release builds for macOS, Linux and Windows with cargo-dist, plus shell, PowerShell and Homebrew
  installers.
