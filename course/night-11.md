# Night 11 · Channel 25 Tunes In

> 📺 *"Ten nights you've been printing things and going home. Tonight the programme stops being a leaflet and becomes a broadcast: we take the whole screen, we keep it, and we hand it back tidy when we're done. Mind the cables. I'm told one of them is holding up the tree."*

**Tonight you'll learn**
- What raw mode and the alternate screen are, and who switches them on and off
- Defining a trait of your own: `App`, with default methods
- Writing an event loop: draw, wait, update, round again
- How ratatui draws a frame, and why drawing takes `&self`
- Why only key presses count, and how a still screen costs nothing
- Refusing politely when there is no terminal: `require_terminal` and the `no-terminal` code

**Where we are:** your build-along program has commands, packs in a `Vault`, an `Rng`, `TinselError`, tests, a `Painter` with plain mode, the printed creature page and `--json`. Everything it does is print and exit.

## The repository is the textbook now

From tonight the finished program is too big to type in one sitting, so the lessons change shape. Each night takes one idea, quotes the code that carries it from the repository, and explains why it is written that way. Clone it and keep it open beside the lesson:

```bash
git clone https://github.com/hungovercoders/terminal-of-tinsel
cd terminal-of-tinsel
cargo test
cargo run -- creature
```

The last command opens the explorer, which is what the next five nights take apart. Press `q` to leave it.

Your own copy is not finished with. The names you have used since Night 3 are the repository's names, so you can carry on by adding each file as it is discussed. Tonight's file is [`src/tui.rs`](../src/tui.rs), and tonight's exercise gets its first half running in your copy.

## One new dependency

```toml
ratatui = { version = "0.30.2", default-features = false, features = ["crossterm"] }
```

ratatui is a library for drawing full-screen terminal programs. It does not talk to the terminal itself: a *backend* does, and crossterm is the backend that works on Linux, macOS and Windows. Asking for the `crossterm` feature brings it in and re-exports it as `ratatui::crossterm`, which is why you will not see a separate `crossterm` line in [`Cargo.toml`](../Cargo.toml). There is one copy, and it is always the version ratatui was built against.

`default-features = false` switches off what ratatui would otherwise add: a calendar widget, macros, a layout cache and coloured underlines. The program uses none of them, and the house rule is to keep dependencies few.

## What "taking over the terminal" means

A terminal normally works a line at a time. It waits for Enter before your program sees anything, echoes what is typed, and scrolls. A full-screen program needs two things changed:

- **Raw mode**: every key arrives at once, unechoed, including the arrows and Ctrl+C.
- **The alternate screen**: a second, blank screen that the terminal swaps in. When the program leaves it, the shell's history is back exactly as it was.

Both must be undone on the way out, however the program leaves. A terminal left in raw mode is a miserable thing to hand back. Here is how the repository does it:

```rust
/// Takes over the terminal and runs the app until it quits. `what` names
/// the screen in the show's own words, for the error if the terminal fails.
pub fn run(app: &mut impl App, what: &str) -> Result<()> {
    let lost = |e: io::Error| {
        TinselError::new(Code::NoTerminal, format!("{what} lost its terminal: {e}"))
            .with_hint("use --json for the data, or tune in from a shell")
    };
    let mut terminal = ratatui::try_init().map_err(lost)?;
    let result = event_loop(&mut terminal, app);
    ratatui::restore();
    result.map_err(lost)
}
```

`ratatui::try_init()` enables raw mode, enters the alternate screen and returns a `Terminal` to draw on. `ratatui::restore()` reverses both. Look at the order of the last three lines. There is no `?` on the `event_loop` line: the result is kept in a variable, the terminal is restored, and only then is the error passed up. With a `?` there, a failure would return early and skip the restore.

`try_init` also installs a panic hook that restores the terminal before the panic message is printed, so even a bug leaves the shell usable and the message readable.

`lost` is a closure, a function written in place that can use the variables around it (here, `what`). It turns an `io::Error` into the `TinselError` you met on Night 7.

## A trait of your own

You have used traits other people wrote: `Deserialize`, `Debug`. A trait is a list of methods a type promises to have. Here is the first one the program defines:

```rust
/// A screen that takes over the terminal: the explorer or a game.
pub trait App {
    /// How often the screen animates, if it does.
    fn tick_every(&self) -> Option<Duration> {
        None
    }
    /// Advances the animation by a frame.
    fn on_tick(&mut self) {}
    fn on_key(&mut self, key: KeyEvent);
    fn resize(&mut self, width: u16, height: u16);
    fn draw(&self, frame: &mut Frame);
    fn quitting(&self) -> bool;
}
```

Four methods have no body, so every `App` must write them. Two have a body, which is a **default**: a screen that never animates leaves them alone and gets `None` and "do nothing".

Read the `self` in each signature, because the borrowing rules from Night 3 are doing real work here. `on_key` and `on_tick` take `&mut self`: they change the state. `draw` takes `&self`: it may look at the state and may not change it. The compiler enforces that, so "drawing never changes anything" is a fact about this program, not a convention somebody might forget. Tomorrow night's tests lean on it.

`run` takes `app: &mut impl App`, which reads as "a mutable borrow of any type that implements `App`". The explorer, the quiz and Guess the Guest are three such types, and they share this one loop.

## The loop

```rust
fn event_loop(terminal: &mut DefaultTerminal, app: &mut impl App) -> io::Result<()> {
    let mut last_tick = Instant::now();
    while !app.quitting() {
        let size = terminal.size()?;
        app.resize(size.width, size.height);
        terminal.draw(|frame| app.draw(frame))?;

        // A still screen waits for a key; an animated one for a key or a tick.
        let key_waiting = match app.tick_every() {
            Some(every) => event::poll(every.saturating_sub(last_tick.elapsed()))?,
            None => true,
        };
        if key_waiting {
            // Windows reports key releases too; only presses count.
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                app.on_key(key);
            }
        }
        match app.tick_every() {
            Some(every) if last_tick.elapsed() >= every => {
                app.on_tick();
                last_tick = Instant::now();
            }
            Some(_) => {}
            None => last_tick = Instant::now(),
        }
    }
    Ok(())
}
```

That is the whole engine. Each time round:

1. **Measure and draw.** The app is told the terminal's size, then asked to draw. `terminal.draw` hands the closure a `Frame`, and the app paints the *entire* screen onto it, every time. This is called immediate mode: there are no widgets that live between frames and need updating, only a function from the state to a picture. It is not as wasteful as it sounds, because ratatui keeps the previous frame and sends the terminal only the cells that changed.

2. **Wait.** `event::read()` blocks until something happens. A still screen (`tick_every()` is `None`) goes straight to it and sleeps there, using no processor at all. An animated screen first calls `event::poll` with the time left until its next tick, which returns `true` if an event arrived in that time and `false` if the time ran out.

3. **Update.** A key goes to `on_key`. If a tick is due, `on_tick` runs. Then round again, where the new state is drawn.

Two details are worth a second look.

The `if let ... && ...` tests a pattern and a condition in one go: "if this event is a key, and the key is a press". Windows reports a key going down and coming back up as two events, and without the second half every key there would count twice.

`every.saturating_sub(last_tick.elapsed())` is how the ticks stay even. If a key arrives 20 milliseconds into a 70 millisecond wait, the next `poll` waits only the remaining 50. `saturating_sub` stops at zero where plain subtraction of a longer `Duration` from a shorter one would panic.

## Plugging the explorer in

The explorer's state is a struct called `Model` (Night 13 opens it up). Making it an `App` is a few lines in [`src/explorer/mod.rs`](../src/explorer/mod.rs):

```rust
impl App for Model<'_> {
    // ...
    fn on_key(&mut self, key: KeyEvent) {
        Model::on_key(self, key);
    }
    // ...
    fn draw(&self, frame: &mut Frame) {
        view::draw(self, frame);
    }

    fn quitting(&self) -> bool {
        Model::quitting(self)
    }
}
```

Each method passes straight through to the model, except `draw`, which goes to a separate `view` module. The model knows nothing about the trait or the terminal. The only code that needs a real terminal is `run` and `event_loop` above, and that is what lets everything else be tested.

## When there is no terminal

Run from a script or a pipe, a full-screen program would hang waiting for a key that never comes, or pour escape codes into a file. So every screen that takes over the terminal is guarded, in [`src/main.rs`](../src/main.rs):

```rust
fn require_terminal(what: &str, instead: &str) -> Result<()> {
    if io::stdout().is_terminal() && io::stdin().is_terminal() {
        return Ok(());
    }
    Err(TinselError::new(
        Code::NoTerminal,
        format!("{what} needs a terminal to play on (stdout or stdin is not one)"),
    )
    .with_hint(instead))
}
```

`is_terminal` comes from the standard library's `IsTerminal` trait. Both ends are checked: the output must be a screen and the input must be a keyboard. You can see it refuse by taking the keyboard away:

```bash
cargo run -q -- creature < /dev/null
```

```
error: the Midwinter Special needs a terminal to play on (stdout or stdin is not one)
hint: use --json for the data, name a creature to print its page, or tune in from a shell
```

The exit code is 1. `Code::NoTerminal` is one of the stable codes from Night 7, spelled `no-terminal` when an error is printed as JSON. The hint says what a script should do instead, and it is true: in `main.rs` the `--json` branch returns the data before `require_terminal` is ever reached, so `creature --json` works in a pipe. 

## Try it

- Run `cargo run -- creature --all --no-intro`, then resize the window while it is open. Nothing in the loop handles a resize event, yet the screen redraws. A resize wakes `event::read()`. Find the two lines of `event_loop` that do the rest.
- Run `cargo run -q -- creature | cat`. Which half of `require_terminal` caught it this time?
- In [`src/tui.rs`](../src/tui.rs), read `snapshot` at the bottom. It calls `resize` and `draw` exactly as the loop does, on a terminal that exists only in memory. That is tomorrow night.

## 🎄 Festive fact

The `if let ... && ...` in the loop is a *let chain*. They became stable in Rust 1.88, in June 2025, and only for code on the 2024 edition. That is one reason `Cargo.toml` says `rust-version = "1.88"`: an older compiler stops with a clear message about the version instead of a confusing syntax error.

## 🕯️ Before morning

Get a full screen of your own on the air. In your build-along copy, add the ratatui line to `Cargo.toml`, then create `src/tui.rs` with the `App` trait, `run` and `event_loop` from the repository, and `mod tui;` in `main.rs`. Write a struct `Hello { quitting: bool }` that implements `App`, draws one line of text and quits on `q`, and run it from a new `hello` command.

Hint: you need the `use` lines from the top of the real file except the ones for colours, and a `NoTerminal` variant in your error codes. `resize` can have an empty body. For `draw`, `frame.render_widget(Paragraph::new("Channel 25 is on the air. Press q."), frame.area())` is enough, with `use ratatui::widgets::Paragraph;`. In `on_key`, compare `key.code` with `KeyCode::Char('q')`.

> 📺 *"There. We took the screen, and we gave it back without so much as a stray bracket. Tomorrow night we find out what happens when somebody watches on a very small set."*

[← Night 10](night-10.md) · [Index](README.md) · [Night 12 →](night-12.md)
