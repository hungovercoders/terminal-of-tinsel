//! The Midwinter Special: the interactive explorer, drawn with ratatui.
//!
//! `model` holds the state and the key handling, `view` draws the gallery
//! and the creature pages, and `intro` draws Channel 25 tuning in. The
//! event loop is `tui::run`; nothing here touches the real terminal, so it
//! is all tested by drawing into ratatui's `TestBackend`.

mod intro;
mod model;
#[cfg(test)]
mod tests;
mod view;

use ratatui::Frame;
use ratatui::crossterm::event::KeyEvent;

use crate::creatures::Vault;
use crate::error::{Code, Result, TinselError};
use crate::tui::{self, App};

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
    tui::run(&mut model, "the Midwinter Special")?;
    Ok(Visit {
        seen: model.seen(),
        sign_off: model.sign_off(),
    })
}

impl App for Model<'_> {
    fn tick_every(&self) -> Option<std::time::Duration> {
        Model::tick_every(self)
    }

    fn on_tick(&mut self) {
        Model::on_tick(self);
    }

    fn on_key(&mut self, key: KeyEvent) {
        Model::on_key(self, key);
    }

    fn resize(&mut self, width: u16, height: u16) {
        Model::resize(self, width, height);
    }

    fn draw(&self, frame: &mut Frame) {
        view::draw(self, frame);
    }

    fn quitting(&self) -> bool {
        Model::quitting(self)
    }
}
