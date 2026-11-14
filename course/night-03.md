# Night 3 · Meet the Guests

> 📺 *"Good evening. The green room is filling up: a horned gentleman from the Alps, a very large cat from Iceland, and a horse's skull from South Wales who is already singing at the door. Tonight we write down who they are, and learn who is allowed to hold the guest list."*

**Tonight you'll learn**
- Structs with several fields, and `Vec`, the growable list
- The two kinds of text: `String` and `&str`
- Ownership and borrowing, the idea Rust is known for
- `impl` blocks: functions that belong to a type
- `for` loops, and padding text with `format!`

**Where we are:** a command line with a welcome and a `list` command that apologises.

## A creature

Every guest has the same shape: an id, a name, an emoji, a description, where the story comes from and some facts. That's a struct. Add this to `src/main.rs`, after the `Command` enum:

```rust
/// A single creature and what we know about it.
#[derive(Debug)]
struct Creature {
    id: String,
    name: String,
    emoji: String,
    description: String,
    origin: String,
    facts: Vec<String>,
}
```

Two new types:

- **`String`** is text that the struct *owns*: it's stored on the heap, it can grow, and it lives as long as the creature does.
- **`Vec<String>`** is a growable list of strings. `Vec` (say "vector") is the list type you'll use most. The angle brackets say what it's a list of.

Last night's constant was a `&str`, and this is the first thing that confuses everybody, so here it is plainly. A `String` owns its text. A `&str` (say "string slice") is a *view* of text that lives somewhere else: inside a `String`, or, for text written in quotes in the source, inside the program file itself. A `&str` is two numbers, where the text starts and how long it is. It's cheap to hand around, but it can't outlive the text it points at.

So a struct that should keep its text holds `String`s, and a function that only wants to read some text asks for a `&str`.

## Functions that belong to a type

Under the struct, add an `impl` block:

```rust
impl Creature {
    /// Builds a creature from borrowed text, making its own copies.
    fn new(id: &str, name: &str, emoji: &str, description: &str, origin: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            emoji: emoji.to_string(),
            description: description.to_string(),
            origin: origin.to_string(),
            facts: Vec::new(),
        }
    }

    /// Adds a fact, and hands the creature back so calls can be chained.
    fn fact(mut self, text: &str) -> Self {
        self.facts.push(text.to_string());
        self
    }

    /// The creature as one line of the guest list.
    fn line(&self) -> String {
        format!("  {} {:<14} {}", self.emoji, self.name, self.origin)
    }
}
```

`impl Creature` holds functions that belong to `Creature`. There are three, and each shows a different way of receiving a creature.

**`new`** has no `self` at all. It's called as `Creature::new(...)` and builds a creature, like a constructor. `Self` (capital S) is shorthand for the type being implemented. Each argument arrives as a borrowed `&str`, and `.to_string()` makes an owned copy for the struct to keep. `Vec::new()` is an empty list. The struct literal at the end has no semicolon and no `return`: the last expression in a function is its value.

**`fact`** takes `mut self`: the creature itself, by value, and changeable. It pushes a fact onto the list and hands the creature back. Because it takes a creature and returns a creature, calls can be chained, which you'll see in a moment.

**`line`** takes `&self`: a *borrowed* creature. It may look but not change, and when it returns, the creature is still where it was. `format!` works like `println!` but returns a `String` instead of printing. `{:<14}` pads the name with spaces on the right to 14 columns, so the next column lines up.

## Ownership, in one page

Those three kinds of `self` are Rust's central idea, so let's say it straight.

1. Every value has exactly one **owner**: a variable, a field, or a list it sits in.
2. When the owner goes away (the variable goes out of scope at a closing brace), the value is cleaned up. There's no garbage collector, and you never free memory by hand.
3. Assigning a value to another variable, or passing it to a function, **moves** it. The old name can't be used afterwards.
4. If you want to let a function use a value without giving it away, you **borrow** it with `&`. Any number of readers may borrow at once (`&`), or exactly one writer (`&mut`), never both.

The compiler checks all four. That's the whole trick: memory safety without a garbage collector, paid for with some arguments at compile time.

Small values such as numbers and `bool` are copied instead of moved, which is why you've never had to think about this with an integer.

## Three guests

Now the guests themselves. Add this function below the `impl` block:

```rust
/// Tonight's guests, typed in by hand for now.
fn creatures() -> Vec<Creature> {
    vec![
        Creature::new(
            "krampus",
            "Krampus",
            "😈",
            "The horned, chain-rattling demon who deals with the naughty while St Nicholas rewards the good",
            "Alpine folklore of Austria, Bavaria and their neighbours",
        )
        .fact("His night is 5 December, Krampusnacht, the eve of St Nicholas Day")
        .fact("He carries chains, bells and bundles of birch switches called Ruten")
        .fact("Austrians sent each other Krampus greeting cards in the 1800s"),
        Creature::new(
            "yule-cat",
            "The Yule Cat",
            "🐈",
            "The giant Icelandic cat who eats anyone without new clothes by Christmas Eve",
            "Icelandic folklore",
        )
        .fact("It eats people who haven't received new clothes before Christmas Eve")
        .fact("Farm workers who finished the autumn wool were rewarded with new clothes")
        .fact("In many tellings it's the pet of the ogress Grýla"),
        Creature::new(
            "mari-lwyd",
            "The Mari Lwyd",
            "🐴",
            "A horse's skull on a pole, draped in white, that comes singing to Welsh doors at midwinter",
            "Welsh folklore, especially Glamorgan and Gwent",
        )
        .fact("She's carried from house to house between Christmas and Twelfth Night")
        .fact("Her jaw is often fitted with a spring so it can snap at people")
        .fact("Her party challenges each household to a battle of verses called the pwnco"),
    ]
}
```

`vec![...]` is a macro that builds a `Vec` from the values listed. Each creature is made by `Creature::new` and then passed through `.fact(...)` three times: each call takes the creature, adds a fact and returns it to the next call. The function returns the whole list, moving it out to whoever called.

The facts are true to the folklore, with the usual caution: tellings vary from valley to valley. Krampus walks with St Nicholas on the evening of 5 December. The Yule Cat eats those who got no new clothes for Christmas. The Mari Lwyd's party trades verses at the door in a contest called the pwnco.

## The list command

Change `main`, and add a `list` function at the end of the file:

```rust
fn main() {
    let cli = Cli::parse();
    let guests = creatures();
    match cli.command {
        None => welcome(),
        Some(Command::List) => list(&guests),
    }
}
```

```rust
fn list(creatures: &[Creature]) {
    println!("🎄 TONIGHT'S GUESTS · {} creatures", creatures.len());
    println!();
    for c in creatures {
        println!("{}", c.line());
        println!("     {}", c.description);
        println!("     {} facts on file, id {}", c.facts.len(), c.id);
    }
}
```

`let guests = creatures();` makes `main` the owner of the list. `list(&guests)` lends it: the `&` borrows the list for the length of the call. Nothing is copied, and `main` still owns it afterwards.

`list` asks for `&[Creature]`, a **slice**: a borrowed view of some creatures in a row. A `&Vec<Creature>` turns into one automatically. Asking for a slice is the convention, because then the function doesn't care whether the creatures came from a `Vec`, an array or part of either. It's the same relationship as `&str` has to `String`.

`for c in creatures` walks the slice. Since the slice is borrowed, each `c` is a `&Creature`, borrowed in turn. `c.facts.len()` and `creatures.len()` count items.

```bash
cargo run -q -- list
```

```
🎄 TONIGHT'S GUESTS · 3 creatures

  😈 Krampus        Alpine folklore of Austria, Bavaria and their neighbours
     The horned, chain-rattling demon who deals with the naughty while St Nicholas rewards the good
     3 facts on file, id krampus
  🐈 The Yule Cat   Icelandic folklore
     The giant Icelandic cat who eats anyone without new clothes by Christmas Eve
     3 facts on file, id yule-cat
  🐴 The Mari Lwyd  Welsh folklore, especially Glamorgan and Gwent
     A horse's skull on a pole, draped in white, that comes singing to Welsh doors at midwinter
     3 facts on file, id mari-lwyd
```

```bash
git add .
git commit -m "feat: three creatures and a list command"
```

## Try it

- See a move happen. In `main`, under `let guests = creatures();`, add `let moved = guests;` and build. The compiler says ``borrow of moved value: `guests` `` and shows where the value moved and where you tried to use it afterwards. Take the line out again.
- Change `line(&self)` to `line(self)` and build. Now `line` would consume the creature, and the compiler objects, because `c` is only borrowed and can't be given away. Put the `&` back.
- Print a creature whole: `println!("{:?}", guests[0]);` uses the `Debug` derive. `{:#?}` spreads it over several lines.
- Change `{:<14}` to `{:>14}` and to `{:^14}`.

## 🎄 Festive fact

Rust's string types are honest about what text is. A `String` is UTF-8, so `"Grýla".len()` is 6, not 5: `len` counts bytes, and `ý` takes two. To count characters you write `"Grýla".chars().count()`. That's also why Rust won't let you write `name[0]` to get the first letter of a string: the first byte isn't always a whole character. We'll need this on Night 7, when Grýla has to be found by people who can't type the accent, and on Night 9, when emoji turn out to be two columns wide.

## 🕯️ Before morning

Add a fourth guest: Grýla, the Icelandic ogress, with the id `gryla` and the emoji `👹`. Her description in the finished program is "The Icelandic ogress who comes down from the mountains at Christmas to cook naughty children", and her origin is "Icelandic folklore". For facts, the finished [`packs/nordic/gryla.json`](../packs/nordic/gryla.json) has seven to choose from.

Then make the title line say "4 creatures" without your changing it. (It already does. Work out why.)

Hint: the list is built in one place and counted in another. Take Grýla out again before tomorrow if you'd like your output to match the lessons; from Night 5 she arrives with the rest of the data.

> 📺 *"Three guests, and the list goes back to its owner at the end of the night. Tomorrow we pull a cracker and see which fact falls out. Mind the bang."*

[← Night 2](night-02.md) · [Index](README.md) · [Night 4 →](night-04.md)
