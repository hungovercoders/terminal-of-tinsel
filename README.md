<div align="center">

```
▀█▀ █▀▀ █▀█ █▀▄▀█ █ █▄ █ ▄▀█ █     █▀█ █▀▀   ▀█▀ █ █▄ █ █▀ █▀▀ █
 █  ██▄ █▀▄ █ ▀ █ █ █ ▀█ █▀█ █▄▄   █▄█ █▀     █  █ █ ▀█ ▄█ ██▄ █▄▄
```

### 📺 Channel 25's Midwinter Special, live in your terminal

![Creatures](https://img.shields.io/badge/creatures-14-52B788)
![Made with Rust](https://img.shields.io/badge/made%20with-rust-E63946)
![Advent](https://img.shields.io/badge/advent%20doors-24-E9C46A)

</div>

> 📺 *"Good evening, and a very merry midwinter. Welcome to the Channel 25 Midwinter Special. I'm your host, Auntie Aerial, broadcasting from the top of the tree."*

**Terminal of Tinsel** is the Christmas sister of [Terminal of Terror](https://github.com/hungovercoders/terminal-of-terror). It brings the creatures of Christmas folklore to your terminal: St Nicholas's grim companions, the Yule Cat and the thirteen Yule Lads, the Mari Lwyd singing at the door, and the spirits of *A Christmas Carol*. Learn the real stories behind them, open the advent calendar and count the sleeps until Christmas.

Terror is written in Go. Tinsel is written in Rust, to see how the two compare. See [docs/go-to-rust.md](docs/go-to-rust.md) for the side-by-side.

> [!WARNING]
> **Festive discretion is advised.** Side effects may include checking your jumper is new on Christmas Eve, leaving butter on the porridge, and practising rhymes in case a horse's skull knocks.

## 🎁 Unwrap it

**With Homebrew** (macOS or Linux):

```bash
brew install hungovercoders/tap/terminal-of-tinsel
```

**With the install script**, on macOS or Linux:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/hungovercoders/terminal-of-tinsel/releases/latest/download/terminal-of-tinsel-installer.sh | sh
```

**On Windows**, in PowerShell:

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/hungovercoders/terminal-of-tinsel/releases/latest/download/terminal-of-tinsel-installer.ps1 | iex"
```

**With Rust 1.88+**, from source:

```bash
cargo install --git https://github.com/hungovercoders/terminal-of-tinsel
```

## 🌟 Your first night

```bash
terminal-of-tinsel                   # Auntie Aerial says hello
terminal-of-tinsel creature          # tune in to the Midwinter Special
terminal-of-tinsel list              # meet tonight's guests
terminal-of-tinsel creature krampus  # read a creature's page
terminal-of-tinsel quiz              # pull a cracker: the Christmas Cracker Quiz
terminal-of-tinsel random --daily    # the Fact of the Night
terminal-of-tinsel tonight           # tonight's Christmas double bill
terminal-of-tinsel countdown         # sleeps until Christmas
terminal-of-tinsel advent            # open the advent calendar
```

> [!TIP]
> Add `terminal-of-tinsel random --daily` to your `~/.zshrc` or `~/.bashrc` and every new terminal opens with the Fact of the Night. From 1 to 24 December, it's about the guest behind that night's advent door.

## 📺 The Midwinter Special

```bash
terminal-of-tinsel creature
```

Channel 25 tunes in through a screen of static that settles into falling snow, Auntie Aerial says good evening, and then the guest list is yours. Every creature has its own colours, a portrait and a page of numbered sections:

| Section | What you'll learn |
|---------|-------------------|
| **Facts** | The essentials, the origin and the first appearance |
| **Legend** | The folklore and history behind the creature |
| **On Screen** | A classic film outing, where there is one |
| **Myth or Fact?** | True or false? Make your guesses, then press `r` for the verdicts |
| **From the Page** | Lines from public-domain sources, such as Dickens |
| **Stat Card** | Strength, speed, cunning and dread, plus powers and weaknesses |
| **Legacy** | Where the creature turns up today |

The spirits of *A Christmas Carol* are drawn in sepia, like the plates of a Victorian Christmas book, with snow drifting past the figure.

```bash
terminal-of-tinsel creature --all            # open on the gallery of every creature
terminal-of-tinsel creature --no-intro       # skip the static
terminal-of-tinsel creature marley --explore # open the explorer on one creature
terminal-of-tinsel creature --pack nordic    # only some packs
```

<details>
<summary><b>⌨️ Keys</b></summary>

| Key | Action |
|-----|--------|
| `←` `→` / `h` `l` / `p` `n` | Previous / next creature |
| `tab` `shift+tab` / `1`-`7` | Switch section |
| `↑` `↓` / `j` `k`, `space`, `pgup`/`pgdn`, `home`/`end` | Scroll |
| `r` | Reveal the Myth or Fact? verdicts |
| `/` | Search names, nicknames, facts, legends and quotes |
| `g` / `esc` | Back to the gallery |
| `?` | Full help |
| `q` / `Ctrl+C` | Quit |

The explorer fills the terminal and adapts to its size. From 100 columns up, the portrait sits beside the text.

</details>

The explorer needs a terminal. From a script, a pipe or CI it stops with the error code `no-terminal` and points you at `--json` instead.

## 🧝 The creature pages

Name a creature and its page is printed instead, which suits a quick look or a pipe:

```bash
terminal-of-tinsel creature yule cat
terminal-of-tinsel creature nisse            # nicknames work
terminal-of-tinsel creature gryla            # accents optional
terminal-of-tinsel creature krampus --reveal # show the Myth or Fact? verdicts
```

A near miss gets a suggestion: `creature krampas` asks whether you meant Krampus.

## 🎲 Games

### The Christmas Cracker Quiz

Questions built from the creature data: facts, lines from the page, first appearances, film credits and myth checks. Every answer explains itself, so you learn as you play. Finish with a rank from **Stocking Filler** to **Star of the Tree**.

```bash
terminal-of-tinsel quiz
terminal-of-tinsel quiz -n 5
terminal-of-tinsel quiz yule cat        # only about one creature
terminal-of-tinsel quiz --pack nordic
```

Answer with `1`-`4` or `a`-`d`, or move with `↑` `↓` and press `enter`.

### Guess the Guest

A portrait is hidden behind a frosted window. Press `space` to clear a little more frost (clues appear as it goes), but every clearing costs points: 100 through the thickest frost, down to 20 once the window is clear.

```bash
terminal-of-tinsel guess
terminal-of-tinsel guess --rounds 8
```

### The Snowball Showdown

Three rounds, no rules, no refunds. Two creatures settle it over strength, speed, cunning and dread, decided by their stat cards and the roll of a die.

```bash
terminal-of-tinsel mash
terminal-of-tinsel mash krampus
terminal-of-tinsel mash "yule cat" gryla --fast   # skip the dramatic pauses
```

The quiz and Guess the Guest need a terminal. With `--json` all three print their data instead: the questions with their answers, the rounds, or the whole showdown.

## 🧦 The Stocking

Answer three questions about a creature correctly, in the quiz or Guess the Guest, and it goes in your **stocking**. The stocking keeps your collection, your badges and your records.

```bash
terminal-of-tinsel stocking
terminal-of-tinsel stocking reset   # tip it out and start again (-y skips the question)
```

<details>
<summary><b>🏅 All 17 badges</b></summary>

| Badge | How to earn it |
|-------|----------------|
| 🎉 First Cracker | Finish your first quiz |
| 🏆 Top of the Tree | Score 100% on a quiz of 5 or more questions |
| 🧦 Stocking Filler | Put your first creature in the stocking |
| 🔍 Frost Reader | Name a guest in Guess the Guest through the thickest frost |
| 🥊 Snowball Referee | Stage your first Snowball Showdown |
| 📚 Fireside Scholar | Visit every creature's page in the explorer |
| 🦉 Up Past Bedtime | Play between midnight and 4am |
| 🌕 Cold Moon | Play a game on a full-moon night |
| 😈 Krampusnacht Survivor | Play a game on 5 December |
| 🎄 Christmas Spirit | Play a game on Christmas Day |
| 🌟 Twelfth Nighter | Play a game on Twelfth Night, 5 January |
| 🎬 Matinee Regular | Capture every creature with a classic film |
| 🎁 Saint's Shadow | Capture every companion of St Nicholas |
| 🌍 Midwinter Wanderer | Capture 5 from Midwinter Europe, or all it has |
| 🐈 Friend of the Yule Cat | Capture every creature of the Nordic Yule |
| 📖 Dickensian | Capture every spirit of *A Christmas Carol* |
| 👑 Keeper of the Stocking | Capture every creature |

</details>

Progress is saved as JSON in your config directory (`~/.config/terminal-of-tinsel/stocking.json` on Linux). Set `TERMINAL_OF_TINSEL_HOME` to keep it somewhere else. The badges count every creature, whatever `--pack` you play with, and the moon is worked out offline.

## 🌙 Nightly rituals

Everything here is worked out offline: the moon, the feasts, the film anniversaries and the sleeps until Christmas.

### 🎟️ Tonight's double bill

A cinema ticket for tonight's Channel 25 double bill. The line-up changes every night, and from 1 to 24 December the guest behind that night's advent door tops the bill.

```bash
terminal-of-tinsel tonight
terminal-of-tinsel tonight --shuffle           # a different bill
terminal-of-tinsel tonight --date 2026-12-24   # what's on for Christmas Eve?
```

### 🎄 The countdown

`countdown` draws the tree and the sleeps until Christmas, tonight's advent door, tonight's moon and the next special night in the diary, with notes for the nights that matter: Krampusnacht, St Nicholas Day, the arrival of each Yule Lad, the anniversary of *A Christmas Carol*, the winter solstice, a full moon, the Twelve Days and Twelfth Night.

```bash
terminal-of-tinsel countdown
terminal-of-tinsel countdown --date 2026-12-24
```

### 📅 The advent calendar

`advent` shows all 24 doors, opening as December goes on, with tonight's guest in full. The same guest tells the Fact of the Night and tops tonight's bill, so every ritual agrees on who's at the door.

```bash
terminal-of-tinsel advent
terminal-of-tinsel advent --date 2026-12-12      # pretend it's another day
```

### ✨ The Fact of the Night

The same fact for everyone all day, and a new one tomorrow. It notices the special nights too: on 31 October it'll remind you that *Scrooge* opened in 1951.

```bash
terminal-of-tinsel random --daily
terminal-of-tinsel random --date 2026-12-19
terminal-of-tinsel random                      # or just any old fact
```

## 📜 The roster

| Pack | Creatures |
|------|-----------|
| **St Nicholas's Companions** (`companions`) | 😈 Krampus · 🧥 Belsnickel · 🖤 Père Fouettard · 🌾 Hans Trapp |
| **Nordic Yule** (`nordic`) | 🐈 The Yule Cat · 👹 Grýla · 🥄 The Yule Lads · 🥣 The Tomte |
| **Midwinter Europe** (`midwinter`) | 🐴 The Mari Lwyd · 🧶 Frau Perchta · 🌳 The Kallikantzaroi · 🪵 Tió de Nadal |
| **A Christmas Carol** (`dickens`) | 👻 Marley's Ghost · ⌛ The Ghost of Christmas Yet to Come |

## 📦 Packs

Creatures come in packs. Use `--pack` with any command to choose which ones come out to play:

```bash
terminal-of-tinsel packs                        # every pack
terminal-of-tinsel quiz --pack nordic           # a Nordic-only quiz
terminal-of-tinsel guess --pack companions      # who's at the window? Probably Krampus
terminal-of-tinsel creature --pack nordic,dickens
```

### 🎁 Make your own

Build your own creatures without writing any Rust:

```bash
terminal-of-tinsel packs new office-party
```

This creates `~/.config/terminal-of-tinsel/packs/office-party/` (or your system's equivalent, or `$TERMINAL_OF_TINSEL_HOME/packs`) with an example creature to edit and a portrait to draw over. Every pack there loads automatically, and its creatures join the explorer, the quiz, Guess the Guest, the Showdown and everything else.

Only `name`, `description` and `facts` are required; everything else gets a sensible default. A file that's broken, a creature that's incomplete or an id that's already taken is skipped with a warning, and never stops the show. Portraits must be printable ASCII. Community creatures stay out of the advent calendar, whose doors are hand-picked.

## 🤖 For scripts

Every command takes `--json` and prints its data instead of drawing. Errors become one JSON object on stderr with a stable code (`unknown-creature`, `ambiguous-creature`, `bad-pack`, `bad-date`, `bad-flag`, `no-terminal`, `store`, `usage`) and exit 1.

```bash
terminal-of-tinsel countdown --json | jq .sleepsUntilChristmas
terminal-of-tinsel creature marley --json | jq '.quotes[].text'
terminal-of-tinsel creature --json | jq -r '.[].name'      # no name: every creature
terminal-of-tinsel creature tomte --json | jq -r .ascii    # the portrait
terminal-of-tinsel quiz -n 5 --json | jq '.[].prompt'
terminal-of-tinsel mash krampus gryla --json | jq -r .winner
terminal-of-tinsel stocking --json | jq '.captured | keys'
```

Output is plain (no colour, no emoji, ASCII borders) when piped, with `--no-color`, or when `NO_COLOR` is set. Set `TERMINAL_OF_TINSEL_SEED` to a number to make every random choice repeatable: the questions, the portraits, the showdowns, `random` and the host's lines.

**Tab completion** for commands, flags, creature names and pack ids, your own packs included:

```bash
source <(terminal-of-tinsel completions zsh)     # also bash, fish, powershell and elvish
terminal-of-tinsel creature kra<TAB>             # krampus  Krampus
```

Put that line in your shell's startup file to keep it. The script asks the program itself what to offer, so it never goes out of date. The release archives carry the same scripts under `completions/` and a man page per command under `man/` (`man -l man/terminal-of-tinsel.1`).

## 🛠️ Build it

```bash
cargo build --release      # target/release/terminal-of-tinsel
cargo test                 # the data tests check every creature, portrait and advent door
cargo clippy --all-targets -- -D warnings
```

Or with [Task](https://taskfile.dev): `task check`, `task run -- creature krampus`. The explorer is drawn with [ratatui](https://ratatui.rs); `DUMP=1 cargo test dump_screens -- --nocapture` prints sample screens without a terminal.

### Releasing

The man pages and completion scripts in `man/` and `completions/` are generated from the command tree and committed, because cargo-dist copies files into the archives but can't generate them. `cargo test` fails if they're stale; `UPDATE_SHELLDOCS=1 cargo test shell_docs` rewrites them. Do that after changing a command or a flag, and when the version changes.

Releases are built by [cargo-dist](https://opensource.axo.dev/cargo-dist/) for macOS, Linux and Windows on Intel and ARM. Pushing a `v*` tag runs `.github/workflows/release.yml`, which builds the archives and installers, creates the GitHub release and pushes a formula to `hungovercoders/homebrew-tap`. That last step needs a `HOMEBREW_TAP_TOKEN` secret with write access to the tap. Run `dist plan` to preview a release.

## 📜 Licence

[CC0 1.0](LICENSE), like Terminal of Terror. Quotes come only from public-domain sources.
