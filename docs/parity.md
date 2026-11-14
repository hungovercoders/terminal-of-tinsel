# Parity with Terminal of Terror

A checklist of everything Terminal of Terror (Go, v1.1.0, read at its `main` of 9 November 2026) has, and
its Christmas equivalent in Terminal of Tinsel. Terror is the reference implementation.

- `[x]` Tinsel has it already (v0.1.0)
- `[ ]` to build, in the phase shown (P1 to P7)
- `[~]` deliberately different, with the reason
- `[?]` needs a decision: see [roadmap.md](roadmap.md)

## Commands

| | Terror | Tinsel | Phase |
|-|--------|--------|-------|
| [x] | bare `terminal-of-terror` prints help | bare command prints Auntie Aerial's welcome (Tinsel extra, kept) | done |
| [x] | `list` | `list` (alias `ls`) | done |
| [x] | `monster` with no name: the interactive explorer | `creature` with no name opens the Midwinter Special explorer | done (P1) |
| [x] | `monster <name>`: explorer opened on that monster | `creature <name>` prints the static page; `creature <name> --explore` (`-x`) opens the explorer there | done (P1) |
| [x] | `monster <name> --json` | `creature <name> --json` | done |
| [x] | `monster --json` with no name: every monster | `creature --json` with no name: every creature | done (P1) |
| [x] | `monster --all` / `-a`: open on the gallery | `creature --all` / `-a` | done (P1) |
| [x] | `monster --no-intro` | `creature --no-intro` | done (P1) |
| [x] | `random`, `--daily` / `-d`, `--date` | the same | done |
| [x] | `quiz [monster]`, `-n` / `--questions` (default 10) | `quiz [creature]`: the Christmas Cracker Quiz | done (P2) |
| [x] | `guess`, `-n` / `--rounds` (default 5) | `guess`: Guess the Guest, the portrait behind a frosted window | done (P2) |
| [x] | `mash [a] [b]`, `--fast` | `mash [a] [b]`, `--fast`: the Snowball Showdown | done (P2) |
| [x] | `crypt` | `stocking` | done (P3) |
| [x] | `crypt reset`, `-y` / `--yes` | `stocking reset`, `-y` / `--yes` | done (P3) |
| [x] | `tonight`, `--shuffle`, `--date` | `tonight`: tonight's Christmas double bill | done (P4) |
| [x] | `countdown`, `--date` | `countdown`, `--date` | done |
| [x] | `countdown` shows the moon and the next anniversary | add both to `countdown` | done (P4) |
| [x] | no equivalent | `advent`, `--date` (Tinsel extra, kept) | done |
| [x] | `packs` | `packs` | done (P5) |
| [x] | `packs new <id>` | `packs new <id>` | done (P5) |
| [~] | `completion <shell>` (Cobra's name) | `completions <shell>` (clap's convention), plus hidden alias `completion` | done (P5) |
| [x] | completes monster names, aliases and pack ids | dynamic completion with clap_complete's `CompleteEnv` | done (P5) |
| [x] | `--version`, `--help` | the same | done |

## Global flags and environment

| | Terror | Tinsel | Phase |
|-|--------|--------|-------|
| [x] | `--json` on every command | the same | done |
| [x] | `--pack a,b` | the same | done |
| [x] | `--pack` accepts community packs, with "did you mean" | today clap only accepts the four built-in ids; validate at run time instead | done (P5) |
| [x] | `--no-color`, `NO_COLOR`, plain when piped | the same | done |
| [x] | `TERMINAL_OF_TERROR_SEED` | `TERMINAL_OF_TINSEL_SEED` (only `random` and the welcome use it so far) | done |
| [x] | the seed fixes quiz, guess, mash, the explorer's host lines and `tonight --shuffle` | all but `tonight --shuffle` done (P1, P2) | done (P4) |
| [x] | a seed that isn't a number warns on stderr and is ignored | check and match | done (P2) |
| [x] | `TERMINAL_OF_TERROR_HOME` | `TERMINAL_OF_TINSEL_HOME`; otherwise the `directories` crate's config dir | done (P3) |

## Error codes (`--json` puts one object on stderr)

| | Terror | Tinsel | Phase |
|-|--------|--------|-------|
| [x] | `unknown-monster` | `unknown-creature` | done |
| [x] | `ambiguous-monster` | `ambiguous-creature` | done |
| [x] | `bad-pack`, `bad-date`, `bad-flag`, `usage` | the same | done |
| [x] | `no-terminal` (explorer, quiz, guess, `crypt reset` without `-y`), with a hint | `no-terminal`: the explorer (P1) and the games (P2) have it; `stocking reset` to come | done (P3) |
| [x] | `store` (progress can't be read or written) | `store` | done (P3) |

## The explorer (Bubble Tea becomes ratatui + crossterm)

| | Terror | Tinsel | Phase |
|-|--------|--------|-------|
| [x] | intro: TV static, big logo, the host's greeting typed out, tonight's notes; any key skips, `q` quits | Channel 25 tunes in through snow, then the same | done (P1) |
| [x] | special greeting on Halloween | special greeting on Christmas Eve and Christmas Day | done (P1) |
| [x] | gallery of every monster with a preview | gallery of every creature, grouped by pack | done (P1) |
| [x] | detail page with up to 7 numbered sections; empty ones are dropped | Facts, Legend, On Screen, Myth or Fact?, From the Page, Stat Card, Legacy | done (P1) |
| [x] | `tab` / `shift+tab` / `1`-`7` switch section | the same | done (P1) |
| [x] | `←` `→` / `h` `l` / `p` `n` previous and next | the same | done (P1) |
| [x] | `↑` `↓` / `j` `k`, `space`, `pgup` / `pgdn`, `home` / `end` scroll | the same | done (P1) |
| [x] | `r` reveals the myth verdicts | the same | done (P1) |
| [x] | `/` searches names, aliases, description, origin, film, facts, legend, myths and legacy, with a snippet | the same | done (P1) |
| [x] | `g` / `esc` gallery, `?` help, `q` / `ctrl+c` quit with a sign-off line | the same; Auntie Aerial needs sign-off lines | done (P1) |
| [x] | portrait beside the text on wide screens | the same | done (P1) |
| [x] | per-monster colours from `theme` | already used by the static page; reuse | done (P1) |
| [x] | silent-film monsters in black and white with intertitles and film grain | the Dickens pack as a Victorian engraving: sepia ink, an ENGRAVER'S INK badge and drifting snow, switched on by `"style": "engraving"` in `pack.json` | done (P1) |
| [x] | ASCII portraits, `<id>.txt` beside each JSON file | original portraits for all 14 creatures | done (P1) |
| [x] | returns the pages visited, for progress | the same | done (P1, P3) |
| [x] | tests render every page at several sizes; `DUMP=1` prints sample screens | ratatui `TestBackend` tests of every page and size | done (P1) |

## Games

| | Terror | Tinsel | Phase |
|-|--------|--------|-------|
| [x] | quiz kinds: `fact`, `quote`, `debut`, `myth`, `film` (star, director), `year` | the same kinds from the same fields | done (P2) |
| [x] | names masked in clues (`▒▒▒▒`), no kind over a third of the quiz, every answer explained | the same | done (P2) |
| [x] | ranks: Ghoul-in-Training, Creature Feature Fan, Monster Scholar, Master of Horror | Stocking Filler, Cracker Puller, Carol Scholar, Star of the Tree | done (P2) |
| [x] | `quiz --json`: the questions with answers | the same shape, `creatureId` for `monsterId` | done (P2) |
| [x] | guess: 5 fog stages (15% to 100%), 100 down to 20 points, clues at stages 3 and 4, four options | the same numbers, with frost on a window | done (P2) |
| [x] | `guess --json`: rounds with options and answer | the same | done (P2) |
| [x] | guess needs 3 creatures and one portrait, else `usage` with a hint | the same | done (P2) |
| [x] | mash: three of four stats, stat plus a d6, re-roll ties, signature move, a finale line | the same rules, with snowball-fight narration in the past tense so plural names read properly | done (P2) |
| [x] | `mash --json`: `a`, `b`, `rounds`, `winner`, `finale` | the same | done (P2) |
| [x] | a creature can't fight itself; empty corners filled at random | the same | done (P2) |
| [x] | dramatic pauses unless `--fast` or piped | the same | done (P2) |

## Progress (the Crypt becomes the Stocking)

| | Terror | Tinsel | Phase |
|-|--------|--------|-------|
| [x] | a progress file in the config dir, written atomically | the stocking's JSON file, written to a temp file and renamed | done (P3) |
| [x] | fields: `version`, `knowledge`, `captured`, `badges`, `seen`, games played and best scores | the same fields | done (P3) |
| [x] | a monster is captured at 3 correct answers (quiz or guess) | a creature goes in the stocking at 3 | done (P3) |
| [x] | games and the explorer record after play; unlocks are announced | the same | done (P3) |
| [x] | a broken save warns and never stops the game | the same | done (P3) |
| [x] | badges count every pack, whatever `--pack` says | the same | done (P3) |
| [x] | `crypt --json` prints the progress | `stocking --json` | done (P3) |
| [x] | `crypt` shows where progress is saved, with `~` | the same | done (P3) |
| [x] | reset asks for "yes"; without a terminal it needs `-y` | the same | done (P3) |

Badges (Terror has 17; Tinsel gets 17):

| | Terror | Tinsel | How |
|-|--------|--------|-----|
| [x] | 🎃 First Fright | 🎉 First Cracker | Finish your first quiz |
| [x] | 🏆 Flawless Fiend | 🏆 Top of the Tree | 100% on a quiz of 5 or more |
| [x] | 🦴 Grave Robber | 🧦 Stocking Filler | First creature in the stocking |
| [x] | 🔍 Eagle Eye | 🔍 Frost Reader | Name a guest through the thickest frost |
| [x] | 🥊 Fight Promoter | 🥊 Snowball Referee | Stage your first showdown |
| [x] | 📚 Midnight Scholar | 📚 Fireside Scholar | Visit every creature's page |
| [x] | 🦉 Night Owl | 🦉 Up Past Bedtime | Play between midnight and 4am |
| [x] | 🌕 Survived the Full Moon | 🌕 Cold Moon | Play on a full-moon night |
| [x] | 🐈 Unlucky for Some | 😈 Krampusnacht Survivor | Play on 5 December |
| [x] | 👻 Halloween Spirit | 🎄 Christmas Spirit | Play on Christmas Day |
| [x] | 🎬 Silent Era Scholar | 🎬 Matinee Regular | Capture every creature with a classic film |
| [x] | 📺 Monster Kid | 🎁 Saint's Shadow | Capture every companion of St Nicholas |
| [x] | 🌍 Folklorist | 🌍 Midwinter Wanderer | Capture 5 from Midwinter Europe (once it has 5; every one until then) |
| [x] | 🔭 Cryptozoologist | 🐈 Friend of the Yule Cat | Capture every Nordic creature |
| [x] | 📖 Bookworm | 📖 Dickensian | Capture every spirit of A Christmas Carol |
| [x] | 🏹 Hero of Legend | 🌟 Twelfth Nighter | Play on Twelfth Night (Terror's sixth pack has no Tinsel twin) |
| [x] | 👑 Master of the Crypt | 👑 Keeper of the Stocking | Capture every creature |

Badge emoji will be checked against the two-column rule before they land.

## Rituals and the calendar

| | Terror | Tinsel | Phase |
|-|--------|--------|-------|
| [x] | nights until Halloween | sleeps until Christmas | done |
| [x] | 31 Nights of Fright (October), one monster a night | 24 Nights of Advent, one guest a door | done |
| [x] | the night's monster also tells the Fact of the Night | the advent guest does | done |
| [x] | and headlines `tonight` | the advent guest headlines the double bill | done (P4) |
| [x] | daily seed `YYYYMMDD` | the same | done |
| [x] | notes: Halloween, Friday the 13th, anniversaries | notes: Krampusnacht, St Nicholas Day, the Tió, Carol anniversary, Christmas Eve, Boxing Day, Twelfth Night, Epiphany, Yule Lads, Twelve Days | done |
| [x] | moon phase (mean synodic month from the 6 January 2000 new moon; name, emoji, illumination) | the moon's age and the full-moon test arrived with the badges (P3); the name, emoji, illumination and `moon` in `countdown --json` to come | done (P4) |
| [x] | full-moon note | full-moon note (December's is the Cold Moon) | done (P4) |
| [x] | "N nights until Halloween" note within 60 nights | "N sleeps until Christmas" note within 60 | done (P4) |
| [x] | film anniversaries from `film.releaseDate`: today's and the next | the "next special night": the nearer of the next film anniversary and the next dated feast | done (P4) |
| [x] | `tonight`: a ticket, two features at 9:00 PM and 10:45 PM, the host's intro, notes | a cinema ticket for Channel 25's double bill at 7:00 PM and 8:45 PM | done (P4) |
| [x] | `tonight --json`: `date`, `features[]`, `notes` | the same | done (P4) |
| [x] | `tonight` needs two creatures, else `usage` | the same | done (P4) |
| [x] | the explorer's intro shows tonight's notes | the same | done (P1) |
| [~] | Friday the 13th | no Christmas twin; Krampusnacht takes its badge | - |
| [x] | no equivalent | winter solstice note on 21 December (a Tinsel extra, cheap with the moon work) | done (P4) |

## Data

| | Terror | Tinsel | Phase |
|-|--------|--------|-------|
| [x] | `id`, `name`, `aliases`, `emoji`, `description`, `origin`, `debut`, `legend`, `myths`, `quotes`, `powers`, `weaknesses`, `stats`, `legacy`, `facts`, `hostIntro`, `theme`, `pack` | all present, same JSON names | done |
| [x] | `film.title`, `year`, `director`, `star`, `note` | present | done |
| [x] | `film.releaseDate`, `studio`, `makeup`, `silent` | add all four as optional | done (P4) |
| [x] | `ascii` from `<id>.txt`, with `\r\n` normalised | add; shown in `--json` as `ascii` | done (P1) |
| [x] | `Pack.Source`: "built-in" or a path | add | done (P5) |
| [x] | lookup by id, name, alias, partial; ambiguity listed; "did you mean" by edit distance | present, and it folds accents too | done |
| [x] | "did you mean" lists up to three names | Tinsel suggests one; match Terror | done (P1) |
| [x] | data tests: every field, 5+ facts, stats 1 to 10, colours, emoji width, unique ids | present | done |
| [x] | data test: portraits are single-width characters | add with the portraits | done (P1) |
| [x] | data test: `film` only where the pack allows | Tinsel allows a film in any pack, so the test checks the credits are complete and `releaseDate` agrees with `year` | done (P4) |
| [~] | 55 monsters in 6 packs | 22 in 4 after P6 (14 before); it grows as creatures can be researched properly, not to hit a number | P6, ongoing |

## Community packs

| | Terror | Tinsel | Phase |
|-|--------|--------|-------|
| [x] | packs load from `<config>/packs/<id>/` on every run | the same; the embedded loader and the disk loader share one code path | done (P5) |
| [x] | only `name`, `description` and `facts` are required; defaults fill the rest | the same (`serde(default)` on a separate lenient type) | done (P5) |
| [x] | broken files, bad stats and duplicate ids warn on stderr and are skipped | the same | done (P5) |
| [x] | `pack.json` is optional for a community pack | the same | done (P5) |
| [x] | `packs new` writes `pack.json`, an example creature and a portrait; refuses built-in ids and bad ids | the same | done (P5) |
| [x] | `packs --json`: `id`, `name`, `description`, `source`, creature ids | the same | done (P5) |
| [~] | community creatures join the 31 Nights rotation | advent doors are hand-written in `advent.json`, so community creatures stay out of advent | - |

## Shipping

| | Terror | Tinsel | Phase |
|-|--------|--------|-------|
| [x] | GoReleaser: Linux, macOS, Windows on Intel and ARM | cargo-dist, same targets | done |
| [x] | Homebrew cask in `hungovercoders/homebrew-tap` | Homebrew formula, same tap | done |
| [x] | a man page per command, from the command tree | clap_mangen | done (P5) |
| [x] | man pages and completion scripts in every archive | generated `man/` and `completions/` committed, a test that fails when they're stale, and cargo-dist `include` | done (P5) |
| [~] | Homebrew installs man pages and completions | cargo-dist's generated formula can't; the README will say where they are in the archive | done (P5) |
| [~] | a Release button (`workflow_dispatch`) that tags and publishes | cargo-dist owns `release.yml` and is tag-driven; left as it is | - |
| [x] | CI: format, lint, tests on three systems, smoke test | present, plus MSRV 1.88 | done |
| [x] | smoke test covers every command, `--json`, plain output, error codes, `packs new`, no-terminal refusals | the same | done (P1 to P5) |
| [x] | CI checks the release config (`goreleaser check`) | cargo-dist's own `plan` job already runs `dist plan` on every pull request | done |
| [x] | CI runs with the home variable pointed at a scratch directory | the same from P3 | done (P3) |

## Repo extras

| | Terror | Tinsel | Phase |
|-|--------|--------|-------|
| [x] | `AGENTS.md`, `CLAUDE.md`, `CHANGELOG.md`, `LICENSE` (CC0), `Taskfile.yml`, `mise.toml` | present | done |
| [~] | README says MIT; the LICENSE file is CC0 | Tinsel says CC0 in both (Terror's badge looks like the slip, worth fixing there) | - |
| [x] | `CONTRIBUTING.md`: creature fields, community packs, demos, releasing, commits | write it | done (P7) |
| [x] | `.editorconfig`, `.gitattributes` | add, with Rust in place of Go | done (P7) |
| [x] | `.github/dependabot.yml`: gomod, npm, github-actions | cargo and github-actions | done (P7) |
| [x] | `cliff.toml` and the git-cliff release step | the same | done (P7) |
| [ ] | VHS tapes, a render script and a theme; hero, quiz, guess and mash GIFs; list, random, countdown, tonight and crypt stills | the tapes, the render script and the theme are in `docs/demos/` (P7); the GIFs and stills are not recorded yet, because that needs ttyd 1.7.7 or newer | P7, recording to do |
| [x] | `docs/README.md` routing table, `docs/ui.md` | the same | done (P7) |
| [ ] | README: programme table, keys, games, badges, rituals, roster by pack, command reference | grow with each phase | P1 to P7 |
| [ ] | slopstopper: `.slopstopper.yml` (profile `ui`), `Taskfile.ss.yml`, `ss-*.yml`, `.githooks/pre-push`, the `slopstopper-cli` pin, two Claude skills | the same installer: profile `library` in P7, switched to `ui` when the website lands | P7, P8 |
| [ ] | `package.json`, `.markdownlint.json`, `.zap/`, `public/_headers`, `wrangler.jsonc` | website tooling, with the website | P8 |
| [ ] | website: a Go generator, a page per monster, the course, Cloudflare Workers Builds | a Rust generator, a page per creature, Cloudflare: see [website-plan.md](website-plan.md) | P8 |
| [ ] | Count Cathode's Night School, 31 nights | Auntie Aerial's Night School, 24 nights; stretch goal; ask first | stretch |
| [ ] | `TUTORIAL.md` pointing at the course | only with the course | stretch |
| [x] | no equivalent | `docs/go-to-rust.md`, updated every phase | ongoing |

The decisions taken, the dependencies vetted and the phases are in [roadmap.md](roadmap.md).
