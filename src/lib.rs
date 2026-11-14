//! Terminal of Tinsel: Channel 25's Midwinter Special, live in your terminal.
//!
//! The program is the binary in `main.rs`. Its modules live in this library
//! so that the website generator (`site/`) reads the creatures, the calendar
//! and the host's lines from the same code as the command line does.

pub mod calendar;
pub mod cli;
pub mod complete;
pub mod creatures;
pub mod error;
pub mod explorer;
pub mod games;
pub mod guess;
pub mod host;
pub mod mash;
pub mod packs;
pub mod plain;
pub mod quiz;
pub mod render;
pub mod rng;
pub mod search;
pub mod stocking;
pub mod store;
pub mod tui;
