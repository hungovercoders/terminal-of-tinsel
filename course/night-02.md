# Night 2 · The Running Order

> 📺 *"Welcome back. A television special needs a running order: who's on, in what order, and what happens if somebody shouts a request from the audience. Tonight the programme learns to listen. It won't have much to say yet, but it will be very polite about it."*

**Tonight you'll learn**
- How to add a library (a *crate*) with `cargo add`
- clap's derive API: a `Cli` struct that describes the command line
- `--help` and `--version`, which clap writes for you
- An `enum` of subcommands, and `match` to choose between them
- What the program does with no command at all: the welcome

**Where we are:** a program that prints one line, whatever you type after its name.

## What a command line is made of

Terminal of Tinsel is going to be run like this:

```bash
terminal-of-tinsel list
terminal-of-tinsel creature krampus
terminal-of-tinsel random --json
```

The first word after the program's name is a **subcommand**. Words after it are **arguments**, and words starting with `--` are **flags**. Reading those by hand from the raw list of words is tedious and error-prone, so nearly every Rust command-line program uses a crate called **clap** to do it.

## Add clap

```bash
cargo add clap@4.6.7 --features derive,wrap_help
```

`cargo add` edits `Cargo.toml` for you. Look at the end of it now:

```toml
[dependencies]
clap = { version = "4.6.7", features = ["derive", "wrap_help"] }
```

A **crate** is a Rust library, published on [crates.io](https://crates.io). The version `4.6.7` means "4.6.7 or any newer 4.x": Cargo trusts that releases with the same first number are compatible, and `Cargo.lock` records the one it picked.

**Features** are optional parts of a crate. `derive` is the part of clap that lets us describe the command line as a struct, and `wrap_help` makes the help text wrap to the width of the terminal. We use the same versions and features as the finished program all the way through, so its code and yours stay interchangeable.

## Describe the command line

Replace `src/main.rs` with this:

```rust
use clap::{Parser, Subcommand};

const GREETING: &str = "Ah, you've tuned in. Lovely. Mind the tinsel, it's holding the aerial up.";

/// Channel 25's Midwinter Special: the creatures of Christmas folklore, live
/// in your terminal
#[derive(Debug, Parser)]
#[command(name = "terminal-of-tinsel", version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// List every creature
    List,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        None => welcome(),
        Some(Command::List) => println!("The guest list arrives tomorrow night."),
    }
}

fn welcome() {
    println!("🎄 TERMINAL OF TINSEL");
    println!("Channel 25's Midwinter Special, live in your terminal");
    println!();
    println!("📺 Auntie Aerial: “{GREETING}”");
    println!();
    println!("Start here:  terminal-of-tinsel list");
}
```

Run it with no arguments:

```bash
cargo run -q
```

```
🎄 TERMINAL OF TINSEL
Channel 25's Midwinter Special, live in your terminal

📺 Auntie Aerial: “Ah, you've tuned in. Lovely. Mind the tinsel, it's holding the aerial up.”

Start here:  terminal-of-tinsel list
```

The first build takes a little while, because Cargo compiles clap and the crates clap uses. After that, only your own code is rebuilt.

There's a lot of new Rust in those thirty-odd lines, so let's take it from the top.

**`use clap::{Parser, Subcommand};`** brings two names from the clap crate into this file, so we can write `Parser` instead of `clap::Parser`.

**`const GREETING: &str = "...";`** is a constant: a value fixed at compile time, with its name in capitals. Its type is `&str`, a piece of text. Constants must have their type written out. There's more to say about `&str`, and tomorrow we'll say it.

**`struct Cli { ... }`** declares a struct: a named bundle of fields. This one has a single field, `command`. The struct is the *description* of our command line, and the lines above it turn that description into a parser:

- `#[derive(Debug, Parser)]` is an attribute. It asks the compiler to write some code for this struct: `Debug`, so the struct can be printed for debugging, and `Parser`, which is clap's and gives the struct a `parse()` function. Night 5 explains derives properly. For now: it's code you didn't have to type.
- `#[command(name = "terminal-of-tinsel", version)]` passes settings to clap: the program's name, and that it should have a `--version` flag.
- The `///` lines are **doc comments**. Ordinary comments start with `//` and are ignored. Doc comments are attached to the thing below them, and clap reads them: this one becomes the description at the top of `--help`.

**`enum Command { List }`** declares an enum: a type whose value is exactly one of a list of **variants**. Tonight there's one variant, `List`. By Night 10 there will be three, and the finished program has twelve. Its doc comment becomes the subcommand's line in the help.

**`command: Option<Command>`** is the important line. `Option` is Rust's way of saying "maybe a value, maybe nothing": it's either `Some(a command)` or `None`. Rust has no null, so anything that might be absent has to say so in its type, and the compiler then makes you deal with both cases. clap sees the `Option` and understands that the subcommand may be left out.

## `match`

```rust
fn main() {
    let cli = Cli::parse();
    match cli.command {
        None => welcome(),
        Some(Command::List) => println!("The guest list arrives tomorrow night."),
    }
}
```

`Cli::parse()` reads the words the program was started with and returns a filled-in `Cli`. If they don't make sense, it prints an error and stops the program there, so the lines after it can rely on a valid `cli`.

`match` looks at a value and runs the first arm whose **pattern** fits. `None` fits when no subcommand was given. `Some(Command::List)` fits when one was, and it was `List`. The `::` in `Command::List` means "the `List` inside `Command`", the same `::` as in `Cli::parse`.

A `match` must cover every possibility. Tonight's "Try it" has you add a variant and watch the compiler point at the `match` you forgot to update. This is the best thing about enums in Rust: add a command and the compiler lists every place that needs to know.

`welcome` is an ordinary function. Inside `println!`, `{GREETING}` puts the constant's text into the line, and an empty `println!()` prints a blank line. The curly quotation marks are ordinary characters in the text.

## What clap gives you

Arguments for your program go after a `--`, which tells Cargo "the rest is not for you":

```bash
cargo run -q -- --help
```

```
Channel 25's Midwinter Special: the creatures of Christmas folklore, live in your terminal

Usage: terminal-of-tinsel [COMMAND]

Commands:
  list  List every creature
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

You didn't write any of that. The description came from the doc comment, the `list` line from the variant, and `help` is a subcommand clap adds itself.

```bash
cargo run -q -- --version
```

```
terminal-of-tinsel 0.1.0
```

The number comes from `version` in `Cargo.toml`. Cargo hands it to the compiler, and clap picks it up from there.

```bash
cargo run -q -- list
```

```
The guest list arrives tomorrow night.
```

And a mistake:

```bash
cargo run -q -- lsit
```

```
error: unrecognized subcommand 'lsit'

  tip: a similar subcommand exists: 'list'

Usage: terminal-of-tinsel [COMMAND]

For more information, try '--help'.
```

clap noticed that `lsit` is close to `list`. On Night 7 we'll write the same kind of "did you mean" ourselves, for creature names.

A program also tells whoever ran it whether it worked, with a number called the **exit code**: 0 for success, anything else for failure. After the mistake above, run `echo $?` (or `echo $LASTEXITCODE` in PowerShell) and you'll see `2`, which is what clap uses for a command line it couldn't understand. Scripts rely on these numbers, and from Night 7 we'll choose our own.

## Save your work

```bash
git add .
git commit -m "feat: a command line with a welcome"
```

## Try it

- Run `cargo run -q -- help list`. Every subcommand gets its own help page.
- Change the doc comment on `List` and look at `--help` again.
- Add a second variant, `Random`, with a doc comment, and build. The compiler refuses with "non-exhaustive patterns: `Some(Command::Random)` not covered" and points at the `match`. Add an arm that prints a line, or take the variant out again: we add it for real on Night 4.
- Replace `cli.command` with `dbg!(cli.command)` and run it with and without `list`. `dbg!` prints a value in its `Debug` form and passes it on, which is what the `Debug` derive is for.

## 🎄 Festive fact

clap stands for Command Line Argument Parser, and it has been around since 2015, the year Rust 1.0 was released. The derive API you used tonight began as a separate crate called structopt, which was folded into clap itself in version 3. The struct-and-attributes style is now how most Rust command-line programs are written.

## 🕯️ Before morning

Give the welcome a second suggestion, so that it ends:

```
Start here:  terminal-of-tinsel list
More:        terminal-of-tinsel --help
```

Then make the name appear only once in the source: add `const NAME: &str = "terminal-of-tinsel";` and use `{NAME}` in both lines.

Hint: a constant can go anywhere outside a function, and `println!` accepts any name that's in scope inside its braces.

> 📺 *"A running order with one item on it. I've worked with less. Tomorrow night the guests arrive, three of them to begin with, and one has brought his own chains."*

[← Night 1](night-01.md) · [Index](README.md) · [Night 3 →](night-03.md)
