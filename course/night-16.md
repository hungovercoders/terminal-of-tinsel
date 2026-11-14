# Night 16 · Writing the Crackers

> 📺 *"Every cracker has a paper hat, a plastic frog and a question nobody can answer. The questions I refuse to write by hand, because then I'd owe you a dozen more every time a new guest came in from the snow. Tonight the program writes its own, out of the facts it already has."*

**Tonight you'll learn**
- Generating content from data instead of writing it by hand
- Hiding the answer: masking a creature's name without a regular expression
- Distractors: wrong answers that are plausible and never accidentally right
- Closures that capture their surroundings, and `&dyn Fn` to pass one to another
- A quota, with an array for a counter
- Tests that check a property of every question, not one question

**Where we are:** the explorer is finished and the host has her lines. Tonight is logic only, with no screen: by the end there are questions, as data.

## Questions for free

Every creature has facts, quotes, a debut, myths and sometimes a film. Each of those is a question waiting to be asked:

| From | Question |
|------|----------|
| a fact | "Which creature is this about?" with the name hidden |
| a quote | "Whose story does this line come from?" |
| the debut | "Where did X first appear?" a book, a film or folklore |
| a myth | "X: true or false?" |
| the film | who played it, who directed it, what year |

That table is the design of [`src/quiz.rs`](../src/quiz.rs). Add a creature and the quiz has about a dozen new questions, and nobody wrote one.

```rust
/// One multiple-choice question.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Question {
    pub kind: Kind,
    pub prompt: String,
    /// Quoted text shown under the prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clue: Option<String>,
    pub options: Vec<String>,
    /// The index of the right option.
    pub answer: usize,
    pub explanation: String,
    pub creature_id: String,
}
```

`answer` is an index into `options`. `creature_id` is what Night 20 uses to give credit for a right answer. `Kind` is an enum of six variants (`Fact`, `Quote`, `Debut`, `Myth`, `Film`, `Year`), not a string, so a misspelt kind is a compile error; serde writes it as `"fact"` or `"myth"` for `--json`.

## One builder, a fixed order

`questions_about` builds every question for one creature: facts, then quotes, then the debut, then myths, then the film. Its doc comment says why the order is fixed:

```rust
/// Every question we can ask about a creature. The order is fixed, so a
/// seeded generator always asks the same questions.
fn questions_about(rng: &mut Rng, c: &Creature, all: &[Creature]) -> Vec<Question> {
```

Every step draws from the same `Rng`. Change the order of the steps and the same seed deals different questions. Each fact becomes a question whose clue is `mask(fact, c)` and whose options come from `choice(rng, q, &c.name, &others)`. Those three names, `mask`, `others` and `choice`, are the ideas of the night.

## Hiding the name

"Which creature is this about? *Krampus* carries birch switches" is not a question. `mask` replaces the creature's names with `▒▒▒▒`. It is done by hand, without a regular expression crate, because the house rule is to keep dependencies few and the job is small enough.

First it gathers the terms to hide:

```rust
    let folded_term = |t: &str| t.chars().map(fold).collect::<Vec<char>>();
    // The whole name and each alias, which may hide a compound...
    let mut terms: Vec<(Vec<char>, bool)> = std::iter::once(c.name.as_str())
        .chain(c.name.strip_prefix("The "))
        .chain(c.aliases.iter().map(String::as_str))
        .map(|t| (folded_term(t), false))
        .collect();
```

`strip_prefix` returns an `Option<&str>`, and an `Option` can be chained onto an iterator: it yields one item or none. So "The Yule Cat" contributes itself and "Yule Cat". `fold` is the accent folding from Night 7: it lowercases and turns `ý` into `y`, always one character for one character, so a position in the folded text is the same position in the original. That is why the terms are `Vec<char>`: a match found in the folded text is the same range in the real one.

Then each telling word of the name is added on its own (the `bool` marks these), because the facts say "the Mari" and "the Lads". Small words such as "the" and "of" are skipped, terms shorter than three characters are dropped, and the list is sorted longest first, so "Yule Cat" is hidden as one blank before "Cat" can leave "Yule ▒▒▒▒" behind.

The scan walks the text and tries the terms at the start of each word. The rules are in the test:

```rust
        assert_eq!(
            mask(
                "His night is Krampusnacht, and Krampus's chains rattle.",
                krampus
            ),
            "His night is ▒▒▒▒, and ▒▒▒▒ chains rattle."
        );
        // ...
        assert_eq!(mask("fed the yule cat's dinner", cat), "fed ▒▒▒▒ dinner");
```

A single-word name takes the rest of its compound with it, so "Krampusnacht" doesn't leak. A possessive goes too. A word from inside a name must match as a whole word, with an optional plural, so "catch" keeps its cat.

## Wrong answers that stay wrong

The wrong options for "which creature is this about?" are other creatures' names. Which ones? Not all of them:

```rust
    // The wrong answers to "which creature is this about?". A creature from
    // the same tradition is left out: a fact about A Christmas Carol fits
    // both of its ghosts, and a decree about the Yule Lads fits their mother
    // too, so offering both would give two right answers.
    let names = |same_tradition: &dyn Fn(&Creature) -> bool| -> Vec<&str> {
        all.iter()
            .filter(|o| o.id != c.id && !same_tradition(o))
            .map(|o| o.name.as_str())
            .collect()
    };
    // Creatures of the same pack share too much as well (Krampus and Čert
    // both rattle chains), so they are left out while there are enough
    // creatures from other packs to choose from.
    let mut others = names(&|o| o.origin == c.origin || o.pack == c.pack);
    if others.len() < 3 {
        others = names(&|o| o.origin == c.origin);
    }
```

Four creatures have the origin "A Christmas Carol by Charles Dickens", and a fact about the book can be true of all of them. If the answer is one ghost and a distractor is another, the player who picks the distractor is right and is told they are wrong. The program cannot read a fact and decide whether it also fits someone else, so it uses what it can compare: creatures with the same `origin` are never offered against each other, and creatures of the same pack are kept out too while there are at least three others to choose from.

`names` is a closure: it uses `all` and `c` without being passed them, because a closure captures the variables around it. It also takes a closure. `&dyn Fn(&Creature) -> bool` means "a reference to anything callable with a creature that returns a bool"; the `dyn` is needed because each closure has its own unnamed type, and `names` is called with two different ones.

`choice` then turns an answer and a list of candidates into shuffled options:

```rust
fn choice(rng: &mut Rng, mut q: Question, answer: &str, distractors: &[&str]) -> Option<Question> {
    let mut wrong: Vec<&str> = Vec::new();
    for d in distractors {
        if !d.eq_ignore_ascii_case(answer) && !wrong.contains(d) {
            wrong.push(d);
        }
    }
    if wrong.len() < 2 {
        return None;
    }
    rng.shuffle(&mut wrong);
    wrong.truncate(3);
    let mut options = vec![answer];
    options.extend(wrong);
    rng.shuffle(&mut options);
    q.answer = options.iter().position(|o| *o == answer)?;
    q.options = options.into_iter().map(str::to_string).collect();
    Some(q)
}
```

Remove the answer under any capitalisation, remove repeats, shuffle, keep three, add the answer, shuffle again, find where the answer landed. With fewer than two wrong options it returns `None`, and the caller's `out.extend(choice(...))` adds nothing: `extend` accepts an `Option` for the same reason `chain` did. A test with only the Dickens pack loaded checks that no who-is-it question is asked.

One kind of question is missing on purpose. A comment in `questions_about` explains that there are no "which is its weakness?" questions, because weaknesses written for different creatures overlap in meaning. Two strings cannot be compared for meaning, so the question is not asked. Leaving a feature out is sometimes the fix.

## A mix, not a pile

There are far more facts than films. A ten-question quiz drawn at random from the pool would be mostly "which creature is this about?". `generate` shuffles the pool, then fills the quiz with a quota for each kind:

```rust
    // Keep a mix: no kind may take more than a third of the quiz, unless we
    // would otherwise run short.
    let limit = n.div_ceil(3).max(1);
    let mut per_kind = [0usize; 6];
    let (mut out, mut spare) = (Vec::new(), Vec::new());
    for q in pool {
        let count = &mut per_kind[q.kind as usize];
        if out.len() < n && *count < limit {
            *count += 1;
            out.push(q);
        } else {
            spare.push(q);
        }
    }
    let short = n.saturating_sub(out.len());
    out.extend(spare.into_iter().take(short));
    out
```

`q.kind as usize` turns an enum variant into its position, 0 for `Fact` to 5 for `Year`, so an array of six counters does the work of a map. `div_ceil` divides and rounds up: a third of 10 is 4. Questions over the quota go to `spare`, and if the first pass comes up short (a quiz about one creature cannot fill every kind's share) the spares top it up.

The shuffle is `Rng::shuffle` in [`src/rng.rs`](../src/rng.rs): three lines of Fisher-Yates on top of the generator from Night 4. Same seed, same shuffle, same quiz.

## Testing every question

The tests ask for `usize::MAX` questions, which is every question the data can make, and check a property of each:

```rust
        let all = generate(&mut Rng::new(1), &v.creatures, usize::MAX, None);
        for q in &all {
            assert!((2..=4).contains(&q.options.len()), "{q:?}");
            assert!(q.answer < q.options.len(), "{q:?}");
            let mut unique = q.options.clone();
            unique.sort();
            unique.dedup();
            assert_eq!(unique.len(), q.options.len(), "repeated option: {q:?}");
            // ...
        }
```

Covering the whole pool does more than many seeds of ten questions would, since no question can hide. `clues_never_name_the_answer` masks every clue a second time and expects nothing to change, and another test checks that no who-is-it question offers a creature of the answer's origin or pack. A loop over many seeds arrives on Night 19, where the dice make it necessary.

## Run it

```bash
cargo test quiz::
TERMINAL_OF_TINSEL_SEED=25 cargo run -- quiz --json -n 4
```

With seed 25, the first of the four questions is (trimmed):

```
    "clue": "Viktor Rydberg's poem ▒▒▒▒ (1881) helped make him a Christmas figure",
    "options": [
      "Krampus",
      "Tió de Nadal",
      "Marley's Ghost",
      "The Tomte"
    ],
    "answer": 3,
    "explanation": "The Tomte: Viktor Rydberg's poem Tomten (1881) helped make him a Christmas figure",
```

The poem's title, *Tomten*, was hidden as a compound of the name.

## Try it

- Run `cargo run -- quiz yule cat --json -n 3`. Every `creatureId` is `yule-cat`, and the quota gives way so that you still get three.
- Run `cargo run -- quiz --pack dickens --json -n 50` and look for a `"fact"`. There isn't one.
- In `generate`, change `limit` to `n` and run `cargo test quiz::`. One test fails, and its name says what you broke. Put it back.

## 🎄 Festive fact

Two of tonight's helpers are recent arrivals in the standard library. `div_ceil` on unsigned integers was stabilised in Rust 1.73, and `Option::is_none_or`, which `generate` uses to apply the `focus`, in Rust 1.82. The `rust-version = "1.88"` line in `Cargo.toml` makes cargo refuse to build the program with anything older.

## 🕯️ Before morning

Write a test for `nearby_years`: for seeds 0 to 99, it returns three different years, none of them the real one, all within eight of it. Hint: the `tests` module is inside `quiz.rs`, so it can call the private function directly; loop with `for seed in 0..100` and make a fresh `Rng::new(seed)` each time.

> 📺 *"More than a hundred and fifty questions, and I wrote none of them. Tomorrow night we put them on screen, with a score in the corner and a rank at the end."*

[← Night 15](night-15.md) · [Index](README.md) · [Night 17 →](night-17.md)
