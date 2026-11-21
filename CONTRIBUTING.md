# Contributing to Terminal of Tinsel

Thank you for wanting to help. A new creature, a whole pack of them, a fix or a better fact are all welcome.

> 📺 *"Every guest on this programme was once a stranger at the door. Come in, and mind the tinsel."* Auntie Aerial

## Getting started

You need [Rust 1.88 or newer](https://rustup.rs). Then:

```bash
git clone https://github.com/hungovercoders/terminal-of-tinsel.git
cd terminal-of-tinsel
cargo run -- creature        # the explorer
cargo run -- list
```

Before you open a pull request, run the three checks CI runs:

```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
```

`cargo test` checks the data as well as the code: every creature's fields, portrait and emoji, every advent door, and that the man pages are up to date. Two things make testing by hand safer and repeatable:

- `TERMINAL_OF_TINSEL_HOME=/some/scratch/dir` keeps your real stocking out of it.
- `TERMINAL_OF_TINSEL_SEED=25` fixes every random choice, so a quiz or a showdown can be repeated exactly. It's also the way to reproduce a bug report.

If you use [mise](https://mise.jdx.dev), `mise install` gets the pinned tools and `task check` runs the three checks.

## Adding a creature

Creatures live in **packs** under `packs/<pack-id>/`. The built-in packs are `companions` (St Nicholas's Companions), `nordic` (Nordic Yule), `midwinter` (Midwinter Europe) and `dickens` (A Christmas Carol). Each pack has:

- `pack.json`: the pack's `id`, `name`, `description` and the display `order` of creature ids
- `<creature-id>.json`: one file per creature
- `<creature-id>.txt`: the creature's portrait. The built-in ones share a style with Terminal of Terror's: a bold picture stamped out of the letter `x` (and spaces), at most 44 columns by 20 lines, usually a face or figure seen from the front. A community pack may use any printable ASCII

A creature file looks like this (see `packs/dickens/marley.json` for a full example):

```json
{
  "id": "your-creature",
  "name": "Your Creature",
  "aliases": ["Nickname"],
  "emoji": "🎁",
  "description": "A one-line description",
  "origin": "Where the tale comes from",
  "debut": {"medium": "folklore", "title": "Where it first appeared", "era": "when, if there's no single year"},
  "film": {"title": "Film Title", "year": 1951, "releaseDate": "1951-10-31", "studio": "Studio",
           "director": "Director", "star": "Who played the creature", "note": "Anything worth adding"},
  "legend": "A paragraph on the real history or folklore behind the creature.",
  "myths": [{"claim": "A popular belief.", "true": false, "explanation": "What's really going on."}],
  "quotes": [{"text": "A line from a public-domain source.", "speaker": "Who", "source": "Where"}],
  "powers": ["Power one", "Power two"],
  "weaknesses": ["Weakness one", "Weakness two"],
  "stats": {"strength": 5, "speed": 5, "cunning": 5, "dread": 5},
  "legacy": ["Where the creature turns up today"],
  "facts": ["Fact 1", "Fact 2", "Fact 3", "Fact 4", "Fact 5"],
  "hostIntro": "What Auntie Aerial says to introduce this creature.",
  "theme": {"primary": "#RRGGBB", "accent": "#RRGGBB"}
}
```

Then add the id to `order` in the pack's `pack.json`, run `cargo test`, and add the creature to the roster table and the badge count in the README.

Guidelines:

- **Accuracy matters.** This is a teaching tool, so check facts, dates and credits against more than one source. Where folklore varies, say so ("in some tellings"); where sources disagree on a date, leave it out or hedge it.
- **Quotes must come from public-domain sources**, such as Dickens, and must be word for word. Check them against the text. No lines from copyrighted films, songs or modern poems, and no copyrighted characters.
- `debut` needs a `year` or an `era`. Its `medium` should say `folklore`, `novella`, `film` and so on: the quiz asks about it.
- `film` is only for a creature with a classic screen outing. Give a `releaseDate` only if you have checked it: it drives the anniversary notes.
- Built-in creatures need 5 or more facts and 3 or more myths. Keep every fact about the creature or its story.
- Every fact, myth, quote and film credit becomes quiz material, so write facts that make good questions. Don't give two creatures of the same tradition a fact that fits them both.
- Myth claims are shown with the creature's name beside them, so "He only comes on Christmas Eve" is fine.
- Stats are 1 to 10 and just for fun. They decide the Snowball Showdown.
- Choose an emoji that is two columns wide and needs no variation selector, or boxes will misalign.
- **Portraits** are original ASCII art: printable ASCII only, at most 46 columns by 20 lines, no trailing spaces.
- Treat folklore from living traditions with respect: explain where it comes from and what it means to the people who tell it.

To put a creature behind an advent door, edit `packs/advent.json`. There are 24 doors, and each needs a creature and a note for the night.

## Community packs

You can try a pack without touching the repository:

```bash
terminal-of-tinsel packs new my-pack
```

This creates a pack in your config directory (`$TERMINAL_OF_TINSEL_HOME/packs` if that's set). Community creatures only need `name`, `description` and `facts`; everything else gets a sensible default. Broken files are skipped with a warning. When your pack is polished, it can be moved into `packs/`, added to `PACK_ORDER` in `src/creatures.rs` and contributed as a built-in pack. Built-in creatures need every field.

## Changing the code

- Read [AGENTS.md](AGENTS.md) for the layout and the house rules, and [docs/ui.md](docs/ui.md) before changing a full-screen page.
- Every command takes `--json`, returns a `TinselError` with a stable code when it fails, and has a line in the smoke test in `.github/workflows/ci.yml` and a case in `every_command_runs` in `tests/cli.rs`.
- After changing a command, a flag or its help text, run `task shelldocs` (or `UPDATE_SHELLDOCS=1 cargo test shell_docs`) to regenerate `man/` and `completions/`, and commit the result.
- Keep dependencies few. Check a new crate's default features and what it brings with it, and that `cargo +1.88 check` still passes.
- British English, and no em dashes.

## Recording the demos

The README's GIFs and screenshots are recorded from scripts in [`docs/demos/`](docs/demos): one `.tape` file per demo, written in the language of [VHS](https://github.com/charmbracelet/vhs), and `docs/demos/render.sh` to play them. VHS itself isn't used: `pty_record.py` plays a tape into a real shell, and `record.cjs` draws the frames with xterm.js in a headless browser, which gets emoji widths right where older terminals don't.

```bash
docs/demos/render.sh              # everything
docs/demos/render.sh hero quiz    # just some
```

You'll need `cargo`, `python3`, `node` with [Playwright](https://playwright.dev) and its Chromium, `ffmpeg`, the **JetBrains Mono** font and **Noto Color Emoji**. Record on Linux: the recordings draw emoji with the system's font, and Noto's licence lets its glyphs ship in the GIFs while Apple's and Microsoft's don't. The script builds the program, uses a scratch home directory, sets `TERMINAL_OF_TINSEL_SEED` to 25 so the keys a tape presses always get the same result, and writes to `docs/assets/`. Look at the results before committing them: the quiz tape's answers move when the roster changes. The recorder understands only the tape commands the demos use (`Type`, `Enter`, `Tab`, `Space`, the arrows, `Sleep` and the size settings).

If you change the seed or the creature data, check that each tape still shows what it should.

## Releasing

Releases are built by [cargo-dist](https://opensource.axo.dev/cargo-dist/) when a `v*` tag is pushed. `.github/workflows/release.yml`, which cargo-dist generates, builds archives and installers for Linux, macOS and Windows on Intel and ARM, publishes a GitHub release and pushes a Homebrew formula to `hungovercoders/homebrew-tap`. That last step needs the `HOMEBREW_TAP_TOKEN` secret.

**Never push a `v*` tag without the maintainer's go-ahead.**

1. **Pick the version** by [semantic versioning](https://semver.org) and set it in `Cargo.toml`. Run `cargo check` so `Cargo.lock` follows.
2. **Regenerate the man pages**, which carry the version: `task shelldocs`.
3. **Update CHANGELOG.md.** [git-cliff](https://git-cliff.org) turns the conventional commits since the last tag into a new section:

   ```bash
   git cliff --unreleased --tag v0.2.0 --strip all
   ```

   CHANGELOG.md is written by hand, so don't let git-cliff edit it. Rename its `## [Unreleased]` heading to the new version and date, check the list git-cliff printed for anything the section has missed, and make sure it reads well for players, not only for developers.
4. **Commit** (`chore(release): prepare for v0.2.0`) and merge it into `main`.
5. **Tag** from an up-to-date `main`:

   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```

Run `dist plan` to see what a release would build without publishing anything. The `plan` job runs it on every pull request.

## Commits and pull requests

Use [conventional commits](https://www.conventionalcommits.org): `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`. Keep a pull request to one piece of work, say what you tested, and say what you didn't. The slopstopper checks (`ss-*.yml`) run on every pull request; `task ss:hygiene:test` runs the static ones locally, and they are also the pre-push hook (`git config core.hooksPath .githooks` to turn it on).

## Reporting issues

Open an issue with what you ran, what you expected and what happened. `--json` output and the value of `TERMINAL_OF_TINSEL_SEED` make a bug much easier to reproduce.

## Licence

By contributing you agree that your contribution is released under [CC0 1.0](LICENSE), like the rest of the project.
