# 📺 Auntie Aerial's Night School

### Build Terminal of Tinsel from nothing, in 24 nights

> 📺 *"Good evening, and welcome to Night School. You've watched the Midwinter Special. Now you'll see how the picture gets made. Twenty-four nights, one door each, and by Christmas Eve you'll have built the whole thing yourself. Bring a pencil and a mince pie. Mind the tinsel."*

This is a course in writing a real Rust command-line application, taught by building **Terminal of Tinsel** from an empty folder to a released program. Every night opens one door and adds one idea. Along the way you'll meet the standard library, [clap](https://docs.rs/clap) for commands, [serde](https://serde.rs) for data, [ratatui](https://ratatui.rs) for interactive screens and [jiff](https://docs.rs/jiff) for dates, and you'll learn how to test, ship and release the lot.

**Who it's for:** anyone who can open a terminal and has written a little code in any language. No Rust experience is needed; Rust is taught as we go, ownership and borrowing included. If you already know Rust, skip to Night 11, where the interactive terminal starts.

**How it works:** each night takes 30 to 60 minutes. Nights 1 to 10 are *build-along*: type the code, run it, see it work. From Night 11 the program is too big to type, so those nights teach the *ideas* with the key code, and point at the finished files in this repository for the rest. Every night ends with an exercise ("Before morning") and a Rust fact ("Festive fact").

Call your project `terminal-of-tinsel`, as this repository does, so the commands and paths in the lessons match yours. You can rename it on the last night.

The course has a sister. [Count Cathode's Night School](https://github.com/hungovercoders/terminal-of-terror/tree/main/course) builds Terminal of Terror in Go over the 31 nights of October. The two programs do the same things in two languages, and [docs/go-to-rust.md](../docs/go-to-rust.md) compares them.

## 📅 The programme

### Week 1 · The First Doors: foundations

| Night | Lesson | You'll learn |
|-------|--------|--------------|
| [1](night-01.md) | Light the Tree | Installing Rust, `cargo new`, `fn main`, `cargo run` and `cargo build` |
| [2](night-02.md) | The Running Order | clap: a `Cli` struct, `--help`, `--version`, subcommands and `match` |
| [3](night-03.md) | Meet the Guests | Structs, `Vec`, `String` and `&str`, ownership and borrowing, a `list` command |
| [4](night-04.md) | A Fact from the Cracker | `Option`, a random generator of your own, seeds |
| [5](night-05.md) | Data under the Tree | JSON with serde, derive macros, `include_str!` |
| [6](night-06.md) | Packs of Creatures | Modules, one file per creature, `include_dir!`, a `Vault` |

### Week 2 · By the Fire: errors, tests, colour and the first screen

| Night | Lesson | You'll learn |
|-------|--------|--------------|
| [7](night-07.md) | Speak Its Name | `Result` and `?`, an error type with codes, name lookup, "did you mean" |
| [8](night-08.md) | Check It Twice | `cargo test`, data tests, end-to-end tests, clippy and rustfmt |
| [9](night-09.md) | Deck the Terminal | Colour, boxes, wrapping, terminal width, plain mode |
| [10](night-10.md) | Every Creature in Its Colours | Themes, the creature page, ASCII portraits, `--json` |
| [11](night-11.md) | Channel 25 Tunes In | ratatui: raw mode, an event loop, drawing a frame |
| [12](night-12.md) | Fitting the Screen | Layout, wrapping, scrolling, testing screens without a terminal |

### Week 3 · The Midwinter Special: the explorer and the games

| Night | Lesson | You'll learn |
|-------|--------|--------------|
| [13](night-13.md) | St Lucy's Night: Turn the Page | Sections and tabs, enums for state, a gallery with a cursor |
| [14](night-14.md) | Whispers in the Snow | Search mode: text input, folding accents, snippets |
| [15](night-15.md) | Your Host, Auntie Aerial | Ticks, TV static, snow with no state, typewriter text |
| [16](night-16.md) | Writing the Crackers | Generating quiz questions from data, hiding the answer, distractors |
| [17](night-17.md) | The Christmas Cracker Quiz | A second screen, feedback, ranks, wiring a command |
| [18](night-18.md) | Behind the Frosted Window | Guess the Guest: the frost, its reveal order, stages and points |

### Week 4 · The Long Night: progress, rituals and release

| Night | Lesson | You'll learn |
|-------|--------|--------------|
| [19](night-19.md) | The Snowball Showdown | Stats and a die, three rounds, narration for one creature or several |
| [20](night-20.md) | What the Stocking Remembers | Saving progress safely, captures and badges |
| [21](night-21.md) | The Longest Night | Dates with jiff, the moon, advent doors, daily seeds |
| [22](night-22.md) | Open the Doors | `--json` everywhere, community packs from disk, validation |
| [23](night-23.md) | The Night Before the Night Before | Tab completion, man pages, generated files, CI on three systems |
| [24](night-24.md) | Christmas Eve: Sign Off | The changelog, cargo-dist, tagging a release, what next |

## Before Night 1

You'll need:

- A terminal. On macOS or Linux, the one you have. On Windows, [Windows Terminal](https://aka.ms/terminal) with PowerShell works well.
- A text editor. [VS Code](https://code.visualstudio.com) with the rust-analyzer extension is a fine choice.
- [Rust 1.88 or newer](https://www.rust-lang.org/tools/install). Night 1 walks you through installing it.
- [Git](https://git-scm.com), so you can save your progress and look at the finished code.

The finished program is in this repository. Whenever a lesson says "the full file in the repo", it means the file at the same path here, for example [`src/creatures.rs`](../src/creatures.rs). Reading ahead is allowed. Copying is encouraged. Understanding is the point.

You don't have to wait for December, and you don't have to keep to one door a night.

📺 *"Channel 25 will now begin its broadcast. Night 1 is [this way](night-01.md)."*
