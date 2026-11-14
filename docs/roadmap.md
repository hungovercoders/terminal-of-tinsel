# Roadmap: decisions, dependencies and phases

The companion to [parity.md](parity.md), which is the checklist itself.

## New dependencies to vet

Each gets its default features and tree checked before it lands, and MSRV 1.88 must still pass.

| Phase | Crate | Why |
|-------|-------|-----|
| P1, done | `ratatui` 0.30 (default features off, `crossterm` backend only; crossterm comes re-exported, so it isn't a direct dependency) | the explorer and the game screens. MSRV 1.88, the same as Tinsel's. About 45 more crates |
| P3, done | `directories` 6.0 (no features; brings `dirs-sys` and `option-ext`, which is MPL-2.0) | the config dir |
| P5, done | `clap_complete` feature `unstable-dynamic` (adds `shlex` and `is_executable`) | `CompleteEnv`; it's behind an unstable feature flag, so the committed scripts in `completions/` show any change |
| P5, done | `clap_mangen` 0.3 as a dev-dependency (adds `roff`) | man pages; never in the shipped binary |

No `regex` (the quiz's name masking is small enough to write by hand) and no `rand` (SplitMix64 stays, so
seeded games never change between releases).

## Decisions

Settled with the maintainer on 12 November 2026.

1. **`creature <name>` with a terminal** prints the static page; `--explore` (`-x`) opens the explorer on
   that creature. Done in P1.
2. **The Dickens pack is an engraving.** In. Done in P1.
3. **Next anniversary** becomes the "next special night": the nearer of the next film anniversary and the
   next dated feast. P4.
4. **Man pages in the archives.** cargo-dist copies extra files into each archive but has no "run this
   first" step, so the files must already exist. Plan: commit generated `man/` and `completions/`, with a
   test that fails when they no longer match the command tree. The alternative, a hidden command run by a
   hand-edited `release.yml`, was explained and not chosen; say so before P5 if you'd rather have it.
5. **Slopstopper and the website.** The website is planned (P8), not a stretch goal. Slopstopper goes in
   with profile `library` in P7 and moves to `ui` in P8. P7 needs read access to
   `hungovercoders/slopstopper`.
6. **Names:** the Christmas Cracker Quiz, Guess the Guest, the Snowball Showdown, the Stocking, and the
   badge names above.
7. **Order:** as numbered.

The Night School course went ahead and is in `course/`. Still to ask, for the website: a domain name
and the go-ahead to connect Cloudflare.

## Phases

One branch and one PR each, conventional commits, `cargo fmt`, `cargo clippy --all-targets -- -D warnings`
and `cargo test` before every PR. Each phase updates the README, `CHANGELOG.md`, `AGENTS.md` and
`docs/go-to-rust.md`.

| Phase | Branch | Contents | go-to-rust.md teaches |
|-------|--------|----------|-----------------------|
| 1 (done) | `feature/explorer` | ratatui explorer, portraits, `no-terminal`, search, `--all`, `--no-intro` | Bubble Tea's Elm loop against ratatui's immediate mode; who owns the event loop |
| 2 (done) | `feature/games` | quiz, guess, mash, the seed everywhere | `math/rand` against a pinned generator; enums with data for question kinds |
| 3 (done) | `feature/stocking` | store, captures, badges, reset | `os.UserConfigDir` against `directories`; maps of times in serde; atomic writes |
| 4 (done) | `feature/rituals` | `tonight`, the moon, anniversaries, new film fields | `time.Time` against jiff's civil dates |
| 5 (done) | `feature/packs` | `packs`, community packs, `packs new`, dynamic completion, man pages | `embed.FS` and `fs.FS` against `include_dir` plus `std::fs`; Cobra completion against `CompleteEnv` |
| 6 (done) | `feature/roster` | new creatures and portraits, advent doors refreshed | nothing new in code: the data tests as the reviewer |
| 7 (done, bar the recordings) | `chore/polish` | tapes and GIFs, CONTRIBUTING, git-cliff, dependabot, .editorconfig, slopstopper | GoReleaser against cargo-dist, start to finish |
| 8 | `feature/website` | the site generator, its pages and tests, Cloudflare hosting, slopstopper profile `ui` | `html/template` and `go run` against a workspace member; sharing code between a binary and a generator |
