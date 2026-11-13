//! The Midwinter Special: the interactive explorer, drawn with ratatui.
//!
//! `model` holds the state and the key handling, `view` draws the gallery
//! and the creature pages, and `intro` draws Channel 25 tuning in. Nothing
//! but `run` touches the real terminal, so the rest is tested by drawing
//! into ratatui's `TestBackend`.

mod intro;
mod model;
#[cfg(test)]
mod tests;
mod view;

use std::io;
use std::time::Instant;

use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event, KeyEventKind};

use crate::creatures::Vault;
use crate::error::{Code, Result, TinselError};

pub use model::{Model, Options};

/// What a visit to the explorer leaves behind.
#[derive(Debug)]
pub struct Visit {
    /// The ids of the creature pages that were opened.
    pub seen: Vec<String>,
    /// The host's parting line, if the player quit rather than crashed out.
    pub sign_off: Option<&'static str>,
}

/// Opens the explorer and runs it until the player quits.
pub fn run(vault: &Vault, opts: Options) -> Result<Visit> {
    if vault.creatures.is_empty() {
        return Err(TinselError::new(Code::Usage, "no creatures in those packs")
            .with_hint("try a different --pack"));
    }
    let mut model = Model::new(vault, opts);
    let mut terminal = ratatui::try_init().map_err(terminal_error)?;
    let result = event_loop(&mut terminal, &mut model);
    ratatui::restore();
    result.map_err(terminal_error)?;
    Ok(Visit {
        seen: model.seen(),
        sign_off: model.sign_off(),
    })
}

/// Draw, wait for a key or the next tick, update, and round again. Unlike
/// Bubble Tea, nothing runs this loop for us: it's ours.
fn event_loop(terminal: &mut DefaultTerminal, model: &mut Model) -> io::Result<()> {
    let mut last_tick = Instant::now();
    while !model.quitting() {
        let size = terminal.size()?;
        model.resize(size.width, size.height);
        terminal.draw(|frame| view::draw(model, frame))?;

        // A still screen waits for a key; an animated one for a key or a tick.
        let key_waiting = match model.tick_every() {
            Some(every) => event::poll(every.saturating_sub(last_tick.elapsed()))?,
            None => true,
        };
        if key_waiting {
            // Windows reports key releases too; only presses count.
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                model.on_key(key);
            }
        }
        match model.tick_every() {
            Some(every) if last_tick.elapsed() >= every => {
                model.on_tick();
                last_tick = Instant::now();
            }
            Some(_) => {}
            None => last_tick = Instant::now(),
        }
    }
    Ok(())
}

fn terminal_error(e: io::Error) -> TinselError {
    TinselError::new(
        Code::NoTerminal,
        format!("the Midwinter Special lost its terminal: {e}"),
    )
    .with_hint("use --json for the data, or tune in from a shell")
}
