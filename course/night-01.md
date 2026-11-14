# Night 1 · Light the Tree

> 📺 *"Good evening, and welcome to the first night of Night School. I'm Auntie Aerial, and behind door number one there is nothing at all: an empty folder and a terminal. By bedtime it will hold a program that says one line. Every tree starts with a single light, and somebody has to find the plug."*

**Tonight you'll learn**
- What Rust is and why it suits a command-line program
- How to install Rust with `rustup` and check it works
- What `cargo new` gives you, and what `Cargo.toml` and the edition are
- Your first program: `fn main` and `println!`
- `cargo run` and `cargo build`, and where the program ends up
- Making a first commit

**Where we are:** an empty folder.

## Why Rust for a terminal program?

Rust compiles to a single program file with no runtime to install beside it. You build `terminal-of-tinsel`, give the file to a friend with the same kind of computer, and it runs. It starts in milliseconds, which matters for something you run forty times an evening.

Rust is also strict. The compiler checks things most languages leave until the program is running: that every value has one clear owner, that nothing is used after it's gone, that every error has somewhere to go. The first few nights that strictness will feel like being marked by a teacher with a very fine pen. By the second week you'll find that when the program compiles, it usually works.

Over 24 nights we build Terminal of Tinsel: a program about the creatures of Christmas folklore, presented by a television host. For the first ten nights you type every line. From Night 11 the program is too big for that, so the lessons quote the finished code in this repository.

## Install Rust

Rust is installed with **rustup**, a small tool that fetches the compiler and keeps it up to date. On macOS or Linux:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

On Windows, download and run `rustup-init.exe` from [rustup.rs](https://rustup.rs). It will ask to install Microsoft's C++ build tools if you don't have them; say yes, because Rust uses their linker.

Then open a *new* terminal and check:

```bash
rustc --version
cargo --version
```

You need Rust 1.88 or newer. If you already had Rust and it's older, `rustup update` brings it up to date.

You've installed three things worth knowing by name:

- **rustc**, the compiler. You'll rarely call it yourself.
- **cargo**, the build tool and package manager. It compiles, runs, tests and fetches libraries. Nearly every command in this course starts with `cargo`.
- **rustup**, which manages the other two.

## A place for the creatures

In the folder where you keep your projects, run:

```bash
cargo new terminal-of-tinsel
cd terminal-of-tinsel
```

Use this exact name, so that the commands and paths in the lessons match yours. `cargo new` makes a folder with four things in it:

- `Cargo.toml`, the description of the project
- `src/main.rs`, the program
- `.gitignore`, which tells Git to ignore the `target` folder
- a new Git repository (unless the folder is already inside one)

Open `Cargo.toml`:

```toml
[package]
name = "terminal-of-tinsel"
version = "0.1.0"
edition = "2024"

[dependencies]
```

Rust calls a project like this a **package**, and the thing it compiles a **crate**. The `name` is what the program file will be called. The `version` starts at `0.1.0`; on Night 2 the program will learn to print it.

The **edition** is which dialect of Rust the code is written in. Every few years the language makes changes that would break old code, such as new keywords, and bundles them into an edition: 2015, 2018, 2021 and 2024. Your code opts in by naming one, and old code keeps compiling under its old edition. We use 2024, the newest.

`[dependencies]` is where libraries will be listed. It's empty tonight.

## The first program

`cargo new` wrote a program for you that prints "Hello, world!". Open `src/main.rs` and change it to this:

```rust
fn main() {
    println!("🎄 The tree is lit.");
}
```

Three things to know:

- **`fn main()`** is where the program starts. `fn` declares a function, and Rust looks for one called `main`.
- **`println!`** prints a line. The `!` means it's a *macro*, not a function: code that writes code at compile time. `println!` needs to be one because it checks its text against its arguments before the program ever runs. You'll use macros long before you need to write one.
- Statements end with a **semicolon**, and blocks are wrapped in braces. Indentation is four spaces. There's a tool that settles all of that, `cargo fmt`, which we'll meet properly on Night 8; run it whenever you like.

A string in Rust is always UTF-8, so the tree emoji needs no special handling. If your terminal shows a box instead of a tree, its font has no emoji; the program is still correct.

Run it:

```bash
cargo run
```

```
   Compiling terminal-of-tinsel v0.1.0 (/home/you/terminal-of-tinsel)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.42s
     Running `/tmp/claude-0/-home-user-terminal-of-tinsel/3b4da507-e117-5adc-8b28-dde3c1d52d17/scratchpad/targets/01/debug/terminal-of-tinsel`
🎄 The tree is lit.
```

`cargo run` compiles the program if anything has changed, then runs it. The first three lines are Cargo talking; the last is your program. (The path and the time will be your own.) Add `-q`, for quiet, and Cargo keeps its remarks to itself:

```bash
cargo run -q
```

```
🎄 The tree is lit.
```

## Where the program went

`cargo run` left the compiled program on disk, in the `target` folder:

```bash
./target/debug/terminal-of-tinsel
```

On Windows it's `.\target\debug\terminal-of-tinsel.exe`.

This is a **debug** build: quick to compile, slow to run, and full of information for debuggers. `cargo build` makes the same file without running it. When you want the one you'd give to somebody, ask for a **release** build:

```bash
cargo build --release
./target/release/terminal-of-tinsel
```

The release build takes longer to compile because the compiler spends the time optimising. On Linux tonight's debug program is about four megabytes and the release one about 450 kilobytes; yours will differ. Either file is the whole program. Nothing else needs installing to run it.

The `target` folder is disposable. Delete it (or run `cargo clean`) and the next build makes it again. That's why `.gitignore` lists it.

You'll also find a new file beside `Cargo.toml` called `Cargo.lock`. It records the exact version of every library the build used, so that the next build, on any machine, uses the same ones. It's short tonight, because there are no libraries yet. For a program, commit it.

## Save your work

`cargo new` already made the repository, so:

```bash
git add .
git commit -m "feat: light the tree"
```

The commit message follows a convention called [Conventional Commits](https://www.conventionalcommits.org): a type (`feat`, `fix`, `docs`, `test`, `chore`), a colon and a short description. On Night 24 a tool will turn these messages into a changelog, so it pays to start now.

## Try it

- Change the message and run `cargo run -q` again.
- Break something: delete the closing `"` and run it. Read the whole error. Rust's compiler errors name the file, the line and the column, draw an arrow at the spot and often suggest the fix. They're long because they're trying to help.
- Print two things: `println!("{} sleeps until {}", 24, "Christmas");`. Each `{}` is filled by the next argument. If you give it too few, the program won't compile, which is the macro doing its checking.
- Run `cargo run` twice in a row. The second time there's no "Compiling" line: nothing changed, so nothing was rebuilt.

## 🎄 Festive fact

Editions are a promise as well as a setting. A crate written in the 2015 edition and a crate written in the 2024 edition can be used in the same program, because the edition only changes how each crate's own source is read, and every edition compiles to the same thing underneath. The 2024 edition arrived with Rust 1.85 in February 2025, and code from 2015 still compiles with today's compiler.

## 🕯️ Before morning

Make the program print a three-line title card:

```
==========================
    TERMINAL OF TINSEL
==========================
```

Hint: `"=".repeat(26)` makes the rule, and you can print it with `println!("{}", "=".repeat(26));`. Do it twice without typing it twice: put the rule in a variable with `let rule = "=".repeat(26);` and print `{rule}`, with the name inside the braces.

> 📺 *"One light, and it's on. Tomorrow night we give the programme a running order, because a show that only does one thing is an advert. Sleep well."*

[Index](README.md) · [Night 2 →](night-02.md)
