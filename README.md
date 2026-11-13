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
terminal-of-tinsel random --daily    # the Fact of the Night
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

## 📅 The advent calendar

`advent` shows all 24 doors, opening as December goes on, with tonight's guest in full. `countdown` draws the tree and the sleeps until Christmas, tonight's door, and notes for the special nights: Krampusnacht, St Nicholas Day, the arrival of each Yule Lad, the anniversary of *A Christmas Carol*, the Twelve Days and Twelfth Night.

```bash
terminal-of-tinsel advent --date 2026-12-12      # pretend it's another day
terminal-of-tinsel countdown --date 2026-12-24
```

## 📜 The roster

| Pack | Creatures |
|------|-----------|
| **St Nicholas's Companions** (`companions`) | 😈 Krampus · 🧥 Belsnickel · 🖤 Père Fouettard · 🌾 Hans Trapp |
| **Nordic Yule** (`nordic`) | 🐈 The Yule Cat · 👹 Grýla · 🥄 The Yule Lads · 🥣 The Tomte |
| **Midwinter Europe** (`midwinter`) | 🐴 The Mari Lwyd · 🧶 Frau Perchta · 🌳 The Kallikantzaroi · 🪵 Tió de Nadal |
| **A Christmas Carol** (`dickens`) | 👻 Marley's Ghost · ⌛ The Ghost of Christmas Yet to Come |

Use `--pack` with any command to keep to some packs: `terminal-of-tinsel list --pack nordic,dickens`.

## 🤖 For scripts

Every command takes `--json` and prints its data instead of drawing. Errors become one JSON object on stderr with a stable code (`unknown-creature`, `ambiguous-creature`, `bad-pack`, `bad-date`, `bad-flag`, `no-terminal`, `usage`) and exit 1.

```bash
terminal-of-tinsel countdown --json | jq .sleepsUntilChristmas
terminal-of-tinsel creature marley --json | jq '.quotes[].text'
terminal-of-tinsel creature --json | jq -r '.[].name'      # no name: every creature
terminal-of-tinsel creature tomte --json | jq -r .ascii    # the portrait
```

Output is plain (no colour, no emoji, ASCII borders) when piped, with `--no-color`, or when `NO_COLOR` is set. Set `TERMINAL_OF_TINSEL_SEED` to a number to make `random`, the host's lines and the explorer's snow repeatable.

**Tab completion:** `source <(terminal-of-tinsel completions zsh)` (also `bash`, `fish`, `powershell` and `elvish`).

## 🛠️ Build it

```bash
cargo build --release      # target/release/terminal-of-tinsel
cargo test                 # the data tests check every creature, portrait and advent door
cargo clippy --all-targets -- -D warnings
```

Or with [Task](https://taskfile.dev): `task check`, `task run -- creature krampus`. The explorer is drawn with [ratatui](https://ratatui.rs); `DUMP=1 cargo test dump_screens -- --nocapture` prints sample screens without a terminal.

### Releasing

Releases are built by [cargo-dist](https://opensource.axo.dev/cargo-dist/) for macOS, Linux and Windows on Intel and ARM. Pushing a `v*` tag runs `.github/workflows/release.yml`, which builds the archives and installers, creates the GitHub release and pushes a formula to `hungovercoders/homebrew-tap`. That last step needs a `HOMEBREW_TAP_TOKEN` secret with write access to the tap. Run `dist plan` to preview a release.

## 📜 Licence

[CC0 1.0](LICENSE), like Terminal of Terror. Quotes come only from public-domain sources.
