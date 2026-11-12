# Parity with Terminal of Terror

A checklist of everything Terminal of Terror (Go, v1.1.0, read at its `main` of 9 November 2026) has, and
its Christmas equivalent in Terminal of Tinsel. Terror is the reference implementation.

- `[x]` Tinsel has it already (v0.1.0)
- `[ ]` to build, in the phase shown (P1 to P7)
- `[~]` deliberately different, with the reason
- `[?]` needs a decision: see [Open questions](#open-questions)

## Commands

| | Terror | Tinsel | Phase |
|-|--------|--------|-------|
| [x] | bare `terminal-of-terror` prints help | bare command prints Auntie Aerial's welcome (Tinsel extra, kept) | done |
| [x] | `list` | `list` (alias `ls`) | done |
| [ ] | `monster` with no name: the interactive explorer | `creature` with no name opens the Midwinter Special explorer | P1 |
| [?] | `monster <name>`: explorer opened on that monster | `creature <name>`: the static page stays (Q1) | P1 |
| [x] | `monster <name> --json` | `creature <name> --json` | done |
| [ ] | `monster --json` with no name: every monster | `creature --json` with no name: every creature | P1 |
| [ ] | `monster --all` / `-a`: open on the gallery | `creature --all` / `-a` | P1 |
| [ ] | `monster --no-intro` | `creature --no-intro` | P1 |
| [x] | `random`, `--daily` / `-d`, `--date` | the same | done |
| [ ] | `quiz [monster]`, `-n` / `--questions` (default 10) | `quiz [creature]`: the Christmas Cracker Quiz | P2 |
| [ ] | `guess`, `-n` / `--rounds` (default 5) | `guess`: Guess the Guest, the portrait behind a frosted window | P2 |
| [ ] | `mash [a] [b]`, `--fast` | `mash [a] [b]`, `--fast`: the Snowball Showdown | P2 |
| [ ] | `crypt` | `stocking` | P3 |
| [ ] | `crypt reset`, `-y` / `--yes` | `stocking reset`, `-y` / `--yes` | P3 |
| [ ] | `tonight`, `--shuffle`, `--date` | `tonight`: tonight's Christmas double bill | P4 |
| [x] | `countdown`, `--date` | `countdown`, `--date` | done |
| [ ] | `countdown` shows the moon and the next anniversary | add both to `countdown` | P4 |
| [x] | no equivalent | `advent`, `--date` (Tinsel extra, kept) | done |
| [ ] | `packs` | `packs` | P5 |
| [ ] | `packs new <id>` | `packs new <id>` | P5 |
| [~] | `completion <shell>` (Cobra's name) | `completions <shell>` (clap's convention), plus hidden alias `completion` | P5 |
| [ ] | completes monster names, aliases and pack ids | dynamic completion with clap_complete's `CompleteEnv` | P5 |
| [x] | `--version`, `--help` | the same | done |

## Global flags and environment

| | Terror | Tinsel | Phase |
|-|--------|--------|-------|
| [x] | `--json` on every command | the same | done |
| [x] | `--pack a,b` | the same | done |
| [ ] | `--pack` accepts community packs, with "did you mean" | today clap only accepts the four built-in ids; validate at run time instead | P5 |
| [x] | `--no-color`, `NO_COLOR`, plain when piped | the same | done |
| [x] | `TERMINAL_OF_TERROR_SEED` | `TERMINAL_OF_TINSEL_SEED` (only `random` and the welcome use it so far) | done |
| [ ] | the seed fixes quiz, guess, mash, the explorer's host lines and `tonight --shuffle` | the same | P1, P2, P4 |
| [ ] | a seed that isn't a number warns on stderr and is ignored | check and match | P2 |
| [ ] | `TERMINAL_OF_TERROR_HOME` | `TERMINAL_OF_TINSEL_HOME`; otherwise the `directories` crate's config dir | P3 |

## Error codes (`--json` puts one object on stderr)

| | Terror | Tinsel | Phase |
|-|--------|--------|-------|
| [x] | `unknown-monster` | `unknown-creature` | done |
| [x] | `ambiguous-monster` | `ambiguous-creature` | done |
| [x] | `bad-pack`, `bad-date`, `bad-flag`, `usage` | the same | done |
| [ ] | `no-terminal` (explorer, quiz, guess, `crypt reset` without `-y`), with a hint | `no-terminal`, same places | P1 to P3 |
| [ ] | `store` (progress can't be read or written) | `store` | P3 |

## The explorer (Bubble Tea becomes ratatui + crossterm)

| | Terror | Tinsel | Phase |
|-|--------|--------|-------|
| [ ] | intro: TV static, big logo, the host's greeting typed out, tonight's notes; any key skips, `q` quits | Channel 25 tunes in through snow, then the same | P1 |
| [ ] | special greeting on Halloween | special greeting on Christmas Eve and Christmas Day | P1 |
| [ ] | gallery of every monster with a preview | gallery of every creature, grouped by pack | P1 |
| [ ] | detail page with up to 7 numbered sections; empty ones are dropped | Facts, Legend, On Screen, Myth or Fact?, From the Page, Stat Card, Legacy | P1 |
| [ ] | `tab` / `shift+tab` / `1`-`7` switch section | the same | P1 |
| [ ] | `←` `→` / `h` `l` / `p` `n` previous and next | the same | P1 |
| [ ] | `↑` `↓` / `j` `k`, `space`, `pgup` / `pgdn`, `home` / `end` scroll | the same | P1 |
| [ ] | `r` reveals the myth verdicts | the same | P1 |
| [ ] | `/` searches names, aliases, description, origin, film, facts, legend, myths and legacy, with a snippet | the same | P1 |
| [ ] | `g` / `esc` gallery, `?` help, `q` / `ctrl+c` quit with a sign-off line | the same; Auntie Aerial needs sign-off lines | P1 |
| [ ] | portrait beside the text on wide screens | the same | P1 |
| [ ] | per-monster colours from `theme` | already used by the static page; reuse | P1 |
| [?] | silent-film monsters in black and white with intertitles and film grain | the Dickens pack as a Victorian engraving: sepia, with gently falling snow (Q2) | P1 |
| [ ] | ASCII portraits, `<id>.txt` beside each JSON file | original portraits for all 14 creatures | P1 |
| [ ] | returns the pages visited, for progress | the same (recorded from P3) | P1, P3 |
| [ ] | tests render every page at several sizes; `DUMP=1` prints sample screens | ratatui `TestBackend` tests of every page and size | P1 |

## Games

| | Terror | Tinsel | Phase |
|-|--------|--------|-------|
| [ ] | quiz kinds: `fact`, `quote`, `debut`, `myth`, `film` (star, director), `year` | the same kinds from the same fields | P2 |
| [ ] | names masked in clues (`▒▒▒▒`), no kind over a third of the quiz, every answer explained | the same | P2 |
| [ ] | ranks: Ghoul-in-Training, Creature Feature Fan, Monster Scholar, Master of Horror | Stocking Filler, Cracker Puller, Carol Scholar, Star of the Tree | P2 |
| [ ] | `quiz --json`: the questions with answers | the same shape, `creatureId` for `monsterId` | P2 |
| [ ] | guess: 5 fog stages (15% to 100%), 100 down to 20 points, clues at stages 3 and 4, four options | the same numbers, with frost on a window | P2 |
| [ ] | `guess --json`: rounds with options and answer | the same | P2 |
| [ ] | guess needs 3 creatures and one portrait, else `usage` with a hint | the same | P2 |
| [ ] | mash: three of four stats, stat plus a d6, re-roll ties, signature move, a finale line | the same rules, with snowball-fight narration | P2 |
| [ ] | `mash --json`: `a`, `b`, `rounds`, `winner`, `finale` | the same | P2 |
| [ ] | a creature can't fight itself; empty corners filled at random | the same | P2 |
| [ ] | dramatic pauses unless `--fast` or piped | the same | P2 |

## Progress (the Crypt becomes the Stocking)

| | Terror | Tinsel | Phase |
|-|--------|--------|-------|
| [ ] | `progress.json` in the config dir, written atomically | `stocking.json`, written to a temp file and renamed | P3 |
| [ ] | fields: `version`, `knowledge`, `captured`, `badges`, `seen`, games played and best scores | the same fields | P3 |
| [ ] | a monster is captured at 3 correct answers (quiz or guess) | a creature goes in the stocking at 3 | P3 |
| [ ] | games and the explorer record after play; unlocks are announced | the same | P3 |
| [ ] | a broken save warns and never stops the game | the same | P3 |
| [ ] | badges count every pack, whatever `--pack` says | the same | P3 |
| [ ] | `crypt --json` prints the progress | `stocking --json` | P3 |
| [ ] | `crypt` shows where progress is saved, with `~` | the same | P3 |
| [ ] | reset asks for "yes"; without a terminal it needs `-y` | the same | P3 |

Badges (Terror has 17; Tinsel gets 17):

| | Terror | Tinsel | How |
|-|--------|--------|-----|
| [ ] | 🎃 First Fright | 🎉 First Cracker | Finish your first quiz |
| [ ] | 🏆 Flawless Fiend | 🏆 Top of the Tree | 100% on a quiz of 5 or more |
| [ ] | 🦴 Grave Robber | 🧦 Stocking Filler | First creature in the stocking |
| [ ] | 🔍 Eagle Eye | 🔍 Frost Reader | Name a guest through the thickest frost |
| [ ] | 🥊 Fight Promoter | 🥊 Snowball Referee | Stage your first showdown |
| [ ] | 📚 Midnight Scholar | 📚 Fireside Scholar | Visit every creature's page |
| [ ] | 🦉 Night Owl | 🦉 Up Past Bedtime | Play between midnight and 4am |
| [ ] | 🌕 Survived the Full Moon | 🌕 Cold Moon | Play on a full-moon night |
| [ ] | 🐈 Unlucky for Some | 😈 Krampusnacht Survivor | Play on 5 December |
| [ ] | 👻 Halloween Spirit | 🎄 Christmas Spirit | Play on Christmas Day |
| [ ] | 🎬 Silent Era Scholar | 🎬 Matinee Regular | Capture every creature with a classic film |
| [ ] | 📺 Monster Kid | 🎁 Saint's Shadow | Capture every companion of St Nicholas |
| [ ] | 🌍 Folklorist | 🌍 Midwinter Wanderer | Capture 5 from Midwinter Europe (once it has 5; every one until then) |
| [ ] | 🔭 Cryptozoologist | 🐈 Friend of the Yule Cat | Capture every Nordic creature |
| [ ] | 📖 Bookworm | 📖 Dickensian | Capture every spirit of A Christmas Carol |
| [ ] | 🏹 Hero of Legend | 🌟 Twelfth Nighter | Play on Twelfth Night (Terror's sixth pack has no Tinsel twin) |
| [ ] | 👑 Master of the Crypt | 👑 Keeper of the Stocking | Capture every creature |

Badge emoji will be checked against the two-column rule before they land.

## Rituals and the calendar

| | Terror | Tinsel | Phase |
|-|--------|--------|-------|
| [x] | nights until Halloween | sleeps until Christmas | done |
| [x] | 31 Nights of Fright (October), one monster a night | 24 Nights of Advent, one guest a door | done |
| [x] | the night's monster also tells the Fact of the Night | the advent guest does | done |
| [ ] | and headlines `tonight` | the advent guest headlines the double bill | P4 |
| [x] | daily seed `YYYYMMDD` | the same | done |
| [x] | notes: Halloween, Friday the 13th, anniversaries | notes: Krampusnacht, St Nicholas Day, the Tió, Carol anniversary, Christmas Eve, Boxing Day, Twelfth Night, Epiphany, Yule Lads, Twelve Days | done |
| [ ] | moon phase (mean synodic month from the 6 January 2000 new moon; name, emoji, illumination) | the same sum with jiff; `moon` in `countdown --json` | P4 |
| [ ] | full-moon note | full-moon note (December's is the Cold Moon) | P4 |
| [ ] | "N nights until Halloween" note within 60 nights | "N sleeps until Christmas" note within 60 | P4 |
| [ ] | film anniversaries from `film.releaseDate`: today's and the next | the same, plus dated feasts (Q3) | P4 |
| [ ] | `tonight`: a ticket, two features at 9:00 PM and 10:45 PM, the host's intro, notes | a cinema ticket for Channel 25's double bill at 7:00 PM and 8:45 PM | P4 |
| [ ] | `tonight --json`: `date`, `features[]`, `notes` | the same | P4 |
| [ ] | `tonight` needs two creatures, else `usage` | the same | P4 |
| [ ] | the explorer's intro shows tonight's notes | the same | P1 |
| [~] | Friday the 13th | no Christmas twin; Krampusnacht takes its badge | - |
| [ ] | no equivalent | winter solstice note on 21 December (a Tinsel extra, cheap with the moon work) | P4 |

## Data

| | Terror | Tinsel | Phase |
|-|--------|--------|-------|
| [x] | `id`, `name`, `aliases`, `emoji`, `description`, `origin`, `debut`, `legend`, `myths`, `quotes`, `powers`, `weaknesses`, `stats`, `legacy`, `facts`, `hostIntro`, `theme`, `pack` | all present, same JSON names | done |
| [x] | `film.title`, `year`, `director`, `star`, `note` | present | done |
| [ ] | `film.releaseDate`, `studio`, `makeup`, `silent` | add all four as optional | P4 |
| [ ] | `ascii` from `<id>.txt`, with `\r\n` normalised | add; shown in `--json` as `ascii` | P1 |
| [ ] | `Pack.Source`: "built-in" or a path | add | P5 |
| [x] | lookup by id, name, alias, partial; ambiguity listed; "did you mean" by edit distance | present, and it folds accents too | done |
| [ ] | "did you mean" lists up to three names | Tinsel suggests one; match Terror | P1 |
| [x] | data tests: every field, 5+ facts, stats 1 to 10, colours, emoji width, unique ids | present | done |
| [ ] | data test: portraits are single-width characters | add with the portraits | P1 |
| [ ] | data test: `film` only where the pack allows | already a house rule; add the test | P4 |
| [ ] | 55 monsters in 6 packs | 14 in 4 today; grow towards it | P6 |

## Community packs

| | Terror | Tinsel | Phase |
|-|--------|--------|-------|
| [ ] | packs load from `<config>/packs/<id>/` on every run | the same; the embedded loader and the disk loader share one code path | P5 |
| [ ] | only `name`, `description` and `facts` are required; defaults fill the rest | the same (`serde(default)` on a separate lenient type) | P5 |
| [ ] | broken files, bad stats and duplicate ids warn on stderr and are skipped | the same | P5 |
| [ ] | `pack.json` is optional for a community pack | the same | P5 |
| [ ] | `packs new` writes `pack.json`, an example creature and a portrait; refuses built-in ids and bad ids | the same | P5 |
| [ ] | `packs --json`: `id`, `name`, `description`, `source`, creature ids | the same | P5 |
| [~] | community creatures join the 31 Nights rotation | advent doors are hand-written in `advent.json`, so community creatures stay out of advent | - |

## Shipping

| | Terror | Tinsel | Phase |
|-|--------|--------|-------|
| [x] | GoReleaser: Linux, macOS, Windows on Intel and ARM | cargo-dist, same targets | done |
| [x] | Homebrew cask in `hungovercoders/homebrew-tap` | Homebrew formula, same tap | done |
| [ ] | a man page per command, from the command tree | clap_mangen | P5 |
| [ ] | man pages and completion scripts in every archive | cargo-dist `include` (Q4) | P5 |
| [~] | Homebrew installs man pages and completions | cargo-dist's generated formula can't; the README will say where they are in the archive | P5 |
| [~] | a Release button (`workflow_dispatch`) that tags and publishes | cargo-dist owns `release.yml` and is tag-driven; left as it is | - |
| [x] | CI: format, lint, tests on three systems, smoke test | present, plus MSRV 1.88 | done |
| [ ] | smoke test covers every command, `--json`, plain output, error codes, `packs new`, no-terminal refusals | extend phase by phase | P1 to P5 |
| [ ] | CI checks the release config (`goreleaser check`) | `dist plan` in CI | P7 |
| [ ] | CI runs with the home variable pointed at a scratch directory | the same from P3 | P3 |

## Repo extras

| | Terror | Tinsel | Phase |
|-|--------|--------|-------|
| [x] | `AGENTS.md`, `CLAUDE.md`, `CHANGELOG.md`, `LICENSE` (CC0), `Taskfile.yml`, `mise.toml` | present | done |
| [~] | README says MIT; the LICENSE file is CC0 | Tinsel says CC0 in both (Terror's badge looks like the slip, worth fixing there) | - |
| [ ] | `CONTRIBUTING.md`: creature fields, community packs, demos, releasing, commits | write it | P7 |
| [ ] | `.editorconfig`, `.gitattributes` | add, with Rust in place of Go | P7 |
| [ ] | `.github/dependabot.yml`: gomod, npm, github-actions | cargo and github-actions | P7 |
| [ ] | `cliff.toml` and the git-cliff release step | the same | P7 |
| [ ] | VHS tapes, `render.sh`, `theme.json`; hero, quiz, guess and mash GIFs; list, random, countdown, tonight and crypt stills | the same set, plus advent and stocking; seed 25 | P7 |
| [ ] | `docs/README.md` routing table, `docs/ui.md` | the same | P7 |
| [ ] | README: programme table, keys, games, badges, rituals, roster by pack, command reference | grow with each phase | P1 to P7 |
| [?] | slopstopper: `.slopstopper.yml` (profile `ui`), `Taskfile.ss.yml`, `ss-*.yml`, `.githooks/pre-push`, the `slopstopper-cli` pin, two Claude skills | the same installer with profile `library` until there's a website (Q5) | P7 |
| [~] | `package.json`, `.markdownlint.json`, `.zap/`, `public/_headers`, `wrangler.jsonc` | website tooling; only with the website stretch goal | stretch |
| [ ] | website: a Go generator, a page per monster, the course, Cloudflare Workers Builds | stretch goal; ask first | stretch |
| [ ] | Count Cathode's Night School, 31 nights | Auntie Aerial's Night School, 24 nights; stretch goal; ask first | stretch |
| [ ] | `TUTORIAL.md` pointing at the course | only with the course | stretch |
| [x] | no equivalent | `docs/go-to-rust.md`, updated every phase | ongoing |

## New dependencies to vet

Each gets its default features and tree checked before it lands, and MSRV 1.88 must still pass.

| Phase | Crate | Why |
|-------|-------|-----|
| P1 | `ratatui` (default features off, `crossterm` backend only), `crossterm` | the explorer and the game screens |
| P3 | `directories` | the config dir |
| P5 | `clap_complete` feature `unstable-dynamic` | `CompleteEnv`; it's behind an unstable feature flag, so the version gets pinned |
| P5 | `clap_mangen` | man pages |

No `regex` (the quiz's name masking is small enough to write by hand) and no `rand` (SplitMix64 stays, so
seeded games never change between releases).

## Open questions

1. **`creature <name>` with a terminal.** Terror opens the explorer on that monster. Your brief says only
   the no-name form opens the TUI. I'd keep the static page for `creature <name>` (it's shipped, and good
   in scripts) and let `creature <name> --explore` (`-x`) open the explorer there. Yes?
2. **The Dickens pack as an engraving.** Terror draws silent-film monsters in black and white with grain.
   The nearest Christmas twin is drawing the Dickens spirits in sepia like John Leech's 1843 plates, with
   snow drifting. In or out?
3. **Next anniversary.** Few Tinsel creatures have a film, so "next film anniversary" would often be empty.
   I'd make it the "next special night": the nearer of the next film anniversary and the next dated feast
   (Krampusnacht, St Lucy's night and so on). Yes?
4. **Man pages in the archives.** cargo-dist includes files that exist at build time. I'd commit generated
   `man/` and `completions/` and have a test fail if they're stale, which is simple and reviewable. The
   other way is a hidden `terminal-of-tinsel man <dir>` command run by a dist build step. Preference?
5. **Slopstopper profile.** Without a website nearly every browser check is moot, so I'd install with
   `--profile library` (hygiene and security checks, the pre-push hook). The installer comes from
   `hungovercoders/slopstopper`, which I'll need read access to in P7.
6. **Game names.** Christmas Cracker Quiz is yours. Guess the Guest, the Snowball Showdown and the badge
   names above are my suggestions: change any you like.
7. **Order.** It's 12 November, 43 sleeps out. The phases are in your order. If you'd like more guests on
   screen before December, P6 (content) can move up without disturbing the rest.

## Phases

One branch and one PR each, conventional commits, `cargo fmt`, `cargo clippy --all-targets -- -D warnings`
and `cargo test` before every PR. Each phase updates the README, `CHANGELOG.md`, `AGENTS.md` and
`docs/go-to-rust.md`.

| Phase | Branch | Contents | go-to-rust.md teaches |
|-------|--------|----------|-----------------------|
| 1 | `feature/explorer` | ratatui explorer, portraits, `no-terminal`, search, `--all`, `--no-intro` | Bubble Tea's Elm loop against ratatui's immediate mode; who owns the event loop |
| 2 | `feature/games` | quiz, guess, mash, the seed everywhere | `math/rand` against a pinned generator; enums with data for question kinds |
| 3 | `feature/stocking` | store, captures, badges, reset | `os.UserConfigDir` against `directories`; maps of times in serde; atomic writes |
| 4 | `feature/rituals` | `tonight`, the moon, anniversaries, new film fields | `time.Time` against jiff's civil dates |
| 5 | `feature/packs` | `packs`, community packs, `packs new`, dynamic completion, man pages | `embed.FS` and `fs.FS` against `include_dir` plus `std::fs`; Cobra completion against `CompleteEnv` |
| 6 | `feature/roster` | new creatures and portraits, advent doors refreshed | nothing new in code: the data tests as the reviewer |
| 7 | `chore/polish` | tapes and GIFs, CONTRIBUTING, git-cliff, dependabot, .editorconfig, slopstopper | GoReleaser against cargo-dist, start to finish |
