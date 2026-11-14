# Changing the UI

1. The explorer lives in `src/explorer/`: `model.rs` holds the state and the key handling, `view.rs` builds the
   gallery and the creature pages, `intro.rs` draws Channel 25 tuning in. The quiz and Guess the Guest are in
   `src/games.rs`. All three implement `App` and run on the one event loop in `src/tui.rs`.
2. A screen is built as lines of styled text (`Vec<Line>`) by plain functions of the model, and only the last
   step lays them on the frame. Add state to the model, handle keys in `on_key`, and draw from the model.
3. Run `cargo test` (it draws every page at several terminal sizes into ratatui's `TestBackend`), and
   `DUMP=1 cargo test dump_ -- --nocapture` to see sample screens without a terminal.
4. Try it in a real terminal at small and large sizes. A standard 80 by 24 window must keep the choices and
   the keys on screen.
5. If the change shows in the README demos, re-record them (see "Recording the demos" in CONTRIBUTING.md).

Colours come from `Ink` in `src/tui.rs` and each creature's `theme`. Plain mode (`--no-color`, `NO_COLOR`) means
no colour, no emoji and ASCII borders: build an emoji with `icon`, so it disappears in plain mode, and add any
new drawing character to `src/plain.rs`, which swaps them for ASCII once the screen is drawn. Avoid emoji
that need a variation selector, because terminals disagree on their width and they break box alignment.

The printed output (`src/render.rs`) has the same plain mode through `Painter`, decided once in `main`.
