# Night 17 · The Christmas Cracker Quiz

> 📺 *"Last night we filled the crackers. Tonight we pull them, on air, with a scoreboard in the corner where everyone can see it. I'll be marking, and I'm strict but fair. Mostly strict."*

**Tonight you'll learn**
- A second `App`: the same trait and event loop from Night 11, with different state
- Keeping what the game is for (the result) apart from what the screen needs
- `into_result(self)`: a method that uses up its value
- Key handling with `match`, `@` bindings and range patterns
- Wiring the `quiz` command: the order of the checks matters
- Plain mode on a full-screen page

**Where we are:** `quiz --json` prints questions. There is no way to play them yet.

## The state of a quiz

The explorer was the first `App`. The quiz is the second, in [`src/games.rs`](../src/games.rs), and it runs on the same `tui::run`. It needs far less state:

```rust
/// How a quiz went.
#[derive(Debug, Clone)]
pub struct QuizResult {
    pub questions: Vec<Question>,
    /// The option chosen for each question, once it has been answered.
    pub answers: Vec<Option<usize>>,
    /// Whether the quiz was played to the end.
    pub completed: bool,
}

pub struct QuizApp {
    result: QuizResult,
    /// The question being asked.
    i: usize,
    cursor: usize,
    answered: bool,
    finished: bool,
    quip: &'static str,
    rng: Rng,
    ink: Ink,
    width: u16,
    quitting: bool,
}
```

There are two structs because there are two lifetimes of interest. `QuizResult` is what matters after the screen has gone: the questions and what was chosen. Everything else in `QuizApp` (where the cursor is, how wide the terminal is) only matters while the screen is up. `answers` is a `Vec<Option<usize>>`: `None` for a question not yet answered, `Some(2)` for one answered with the third option. The score is never stored. It is worked out from the answers each time, so it cannot disagree with them:

```rust
    pub fn score(&self) -> (usize, usize) {
        let given = self.answers.iter().flatten().count();
        let right = self
            .questions
            .iter()
            .zip(&self.answers)
            .filter(|(q, a)| **a == Some(q.answer))
            .count();
        (right, given)
    }
```

`flatten` on an iterator of `Option`s drops the `None`s, so `given` counts the answers that exist. `zip` walks the questions and answers side by side.

When the game ends, the app gives up its result:

```rust
    pub fn into_result(self) -> QuizResult {
        self.result
    }
```

It takes `self`, not `&self`. Calling it moves the app into the method, which hands back one field and drops the rest. The screen cannot be used afterwards, and the compiler enforces that. No copy of the questions is made.

## Three states, four keys

A quiz is always in one of three situations: a question is waiting, an answer is showing, or the final score is up. Two booleans, `answered` and `finished`, tell them apart, and `on_key` is a ladder over them:

```rust
    fn on_key(&mut self, key: KeyEvent) {
        if is_quit(key) || self.finished {
            self.quitting = true;
        } else if is_chord(key) {
            // Not an answer, whatever letter came with it.
        } else if self.answered {
            if is_next(key.code) {
                self.answered = false;
                self.cursor = 0;
                if self.i + 1 == self.result.questions.len() {
                    self.finished = true;
                } else {
                    self.i += 1;
                }
            }
        } else {
            let code = match key.code {
                KeyCode::Char(' ') => KeyCode::Enter,
                code => code,
            };
            let options = self.result.questions[self.i].options.len();
            if let Some(n) = pick_option(code, &mut self.cursor, options) {
                self.result.answers[self.i] = Some(n);
                // The quiz is complete once the last question is answered,
                // whether or not the player stays for the final score.
                self.result.completed = self.i + 1 == self.result.questions.len();
                self.answered = true;
                self.cursor = n;
                self.quip = host::quip(&mut self.rng);
            }
        }
    }
```

Read it from the top. `q`, `esc` and `ctrl+c` always leave, and on the final screen any key does. A key pressed with ctrl or alt is ignored: without that rung, `ctrl+d` would arrive as the letter `d` and lock in option 4. While an answer is showing, only the "next" keys do anything. Otherwise the key may choose an option.

Notice the last line. The host's quip is chosen once, when the answer is given, and kept in the state. `draw` takes `&self` and runs on every pass of the event loop, so a quip picked inside `draw` would need a mutable `Rng` it cannot have, and would change every time the terminal was resized. Anything random belongs in `on_key` or `on_tick`, never in drawing.

Choosing an option is shared with Guess the Guest:

```rust
fn pick_option(code: KeyCode, cursor: &mut usize, n: usize) -> Option<usize> {
    match code {
        KeyCode::Up | KeyCode::Char('k') => *cursor = (*cursor + n - 1) % n,
        KeyCode::Down | KeyCode::Char('j') => *cursor = (*cursor + 1) % n,
        KeyCode::Enter => return Some(*cursor),
        KeyCode::Char(c @ '1'..='9') => return Some(c as usize - '1' as usize).filter(|i| *i < n),
        // Caps Lock shouldn't stop a letter counting.
        KeyCode::Char(c) => {
            let c = c.to_ascii_lowercase();
            return ('a'..='d')
                .contains(&c)
                .then(|| c as usize - 'a' as usize)
                .filter(|i| *i < n);
        }
        _ => {}
    }
    None
}
```

`c @ '1'..='9'` matches a character in that range and names it `c`. Characters are numbers underneath, so `c as usize - '1' as usize` turns `'3'` into 2. The `.filter(|i| *i < n)` turns a `3` on a true-or-false question into `None`. Moving up is `(*cursor + n - 1) % n` and not `*cursor - 1`, because `cursor` is a `usize` and subtracting 1 from 0 would panic in a debug build. Adding `n - 1` and wrapping gives the same answer without going below zero.

## Drawing from the bottom up

As in the explorer, the screen is a `Vec<Line>` built by a plain method, `lines()`, and only the last step touches the frame. `lines()` is long but flat: a title bar, then either the final score, or the prompt, the clue, the options and the verdict. The laying out is shared by both games:

```rust
fn place(mut lines: Lines, width: usize, ink: Ink, frame: &mut Frame) {
    let area = frame.area();
    let rows = usize::from(area.height);
    if lines.len() > rows && rows >= 2 {
        let cut = lines.len() - rows + 1;
        lines.drain(1..cut);
    }
    let width = u16::try_from(width).unwrap_or(u16::MAX).min(area.width);
    let x = area.x + (area.width - width) / 2;
    Paragraph::new(lines).render(Rect::new(x, area.y, width, area.height), frame.buffer_mut());
    if !ink.colour {
        crate::plain::plainify(frame.buffer_mut());
    }
}
```

If there are more lines than rows, the title stays and lines are removed from directly below it. The explorer scrolls, but a game should not make you scroll to find the answer keys. So when the terminal is short, it is the top of the clue that gives way, and the choices, the verdict and the keys stay on screen. `drain(1..cut)` removes that range from the `Vec` in place.

## Ranks

When the last question has been answered, the score becomes a rank, in [`src/quiz.rs`](../src/quiz.rs):

```rust
pub fn rank(percent: usize) -> &'static str {
    match percent {
        90.. => "Star of the Tree",
        70.. => "Carol Scholar",
        40.. => "Cracker Puller",
        _ => "Stocking Filler",
    }
}
```

`90..` is a range pattern with no upper end: 90 or more. Arms are tried in order, so 95 stops at the first and never reaches `70..`.

## Wiring the command

The `Quiz` variant in [`src/cli.rs`](../src/cli.rs) carries a `creature: Vec<String>` (so `quiz yule cat` works without quotes) and `questions: usize` with `-n` and a default of 10. The method that runs it is in [`src/main.rs`](../src/main.rs):

```rust
        let mut rng = Rng::from_env_or_clock();
        let quiz = quiz::generate(&mut rng, &self.vault.creatures, questions, focus);
        // ...
        if self.json {
            return print_json(&quiz);
        }
        require_terminal(
            "the Christmas Cracker Quiz",
            "use --json for the questions and answers, or play from a shell",
        )?;
        let started = calendar::now();
        let mut app = games::QuizApp::new(quiz, rng, self.colour);
        tui::run(&mut app, "the Christmas Cracker Quiz")?;
        let result = app.into_result();
        print(&render::quiz_summary(&self.painter, &result));
```

The order is deliberate. The questions are generated first, so an empty quiz is an error before any screen opens. `--json` is answered before `require_terminal`, so a script with no terminal still gets its data. The `Rng` that made the questions is then moved into the app, where it picks the quips: one seed fixes the whole session. And the summary is printed after `tui::run` returns, because by then the alternate screen from Night 11 has been closed and the game has vanished with it. Two printed lines are what remains in your scrollback.

## Plain mode, full screen

`--no-color` and `NO_COLOR` promise no colour, no emoji and ASCII borders. On printed output the `Painter` from Night 9 keeps that promise. A full-screen page has no `Painter`, so it is kept in three places.

Colour: every style comes from `Ink` in [`src/tui.rs`](../src/tui.rs), which returns an unstyled `Style` when `colour` is false. Emoji: text is built with `ink.icon`, which gives the emoji and a space, or an empty string:

```rust
    pub fn icon(self, emoji: &str) -> String {
        if self.colour {
            format!("{emoji} ")
        } else {
            String::new()
        }
    }
```

Borders and marks are the awkward part, since `╭`, `▶` and `✔` are spread through the drawing code. Instead of two versions of every line, [`src/plain.rs`](../src/plain.rs) fixes the screen after it is drawn. That is the last step of `place` above:

```rust
/// Swaps every drawing character on the screen for its ASCII stand-in.
pub fn plainify(buf: &mut Buffer) {
    for cell in &mut buf.content {
        if let Some(plain) = ascii(cell.symbol()) {
            cell.set_symbol(plain);
        }
    }
}
```

A ratatui `Buffer` is a grid of cells, and `ascii` looks each cell's symbol up in a table of 29 pairs: `╭` becomes `+`, `▶` becomes `>`, `✔` becomes `*`, and the `▒` of a masked name becomes `%`. A test in `games.rs` draws both games in plain mode and checks every cell: no symbol is two columns wide, none is in the table, and no cell has a foreground colour.

## Run it

You can see a screen without a terminal. The tests draw into ratatui's `TestBackend`, and one of them prints what it drew when asked:

```bash
DUMP=1 cargo test dump_games -- --nocapture
```

The test uses seed 25 and answers 2 to the first question:

```
 🎉 THE CHRISTMAS CRACKER QUIZ                                   Question 1 of 3 · Score 0

Which creature is this about?
  Viktor Rydberg's poem ▒▒▒▒ (1881) helped make him a Christmas figure

  1. Krampus
✘ 2. Tió de Nadal
  3. Marley's Ghost
✔ 4. The Tomte

✘ Not quite. The answer is The Tomte.
The Tomte: Viktor Rydberg's poem Tomten (1881) helped make him a Christmas figure
📺 Festive, isn't it? In a worrying sort of way.

enter next question · q quit
```

Then play it for real: `cargo run -- quiz -n 5`.

## Try it

- Play with `cargo run -- quiz -n 3 --no-color` and compare the borders and marks.
- Run `cargo run -- quiz | cat`. The pipe is not a terminal, so you get the `no-terminal` error and its hint, not a hung program.
- Shrink your terminal to twelve rows mid-question. The title and the options stay.
- Read `a_perfect_quiz_ends_at_the_top_of_the_tree` in `games.rs`: it plays five questions by pressing keys and reading the screen back.

## 🎄 Festive fact

The `matches!` macro, which `is_quit` and `is_next` use to ask "is this value one of these patterns?", joined the standard library in Rust 1.42, in March 2020. Before that, the same test was a `match` with a `true` arm and a `false` arm.

## 🕯️ Before morning

Give the quiz a fourth rank, "Fireside Regular", for 55% and over. Hint: the order of the arms in `rank` matters, `final_word` beside it has the same bands and should say something too, and the test `ranks_and_mid_sentence_names` wants a new line.

> 📺 *"Pencils down. Tomorrow night there's somebody at the window, and the glass has frosted over. You may wipe a little of it clear, but it will cost you."*

[← Night 16](night-16.md) · [Index](README.md) · [Night 18 →](night-18.md)
