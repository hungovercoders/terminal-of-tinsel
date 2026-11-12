//! The explorer's state and what each key does to it.

use std::collections::BTreeSet;
use std::time::Duration;

use jiff::civil::Date;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{intro, view};
use crate::calendar;
use crate::creatures::{Creature, Vault};
use crate::host;
use crate::rng::Rng;
use crate::search::{self, Hit};

/// Animation speeds: the intro runs fast, and the snow over an engraving
/// only needs to drift.
const INTRO_TICK: Duration = Duration::from_millis(70);
const SNOW_TICK: Duration = Duration::from_millis(300);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Intro,
    Gallery,
    Detail,
}

/// A numbered section of a creature's page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Facts,
    Legend,
    Film,
    Myths,
    Quotes,
    Stats,
    Legacy,
}

impl Tab {
    /// The section's name; a shorter one keeps the tab bar on one line in
    /// narrow terminals.
    pub fn name(self, wide: bool) -> &'static str {
        match (self, wide) {
            (Tab::Facts, _) => "Facts",
            (Tab::Legend, _) => "Legend",
            (Tab::Film, true) => "On Screen",
            (Tab::Film, false) => "Screen",
            (Tab::Myths, true) => "Myth or Fact?",
            (Tab::Myths, false) => "Myths",
            (Tab::Quotes, true) => "From the Page",
            (Tab::Quotes, false) => "Quotes",
            (Tab::Stats, true) => "Stat Card",
            (Tab::Stats, false) => "Stats",
            (Tab::Legacy, _) => "Legacy",
        }
    }
}

/// The sections a creature has something to say in.
pub fn tabs_for(c: &Creature) -> Vec<Tab> {
    let mut tabs = vec![Tab::Facts];
    if !c.legend.is_empty() {
        tabs.push(Tab::Legend);
    }
    if c.film.is_some() {
        tabs.push(Tab::Film);
    }
    if !c.myths.is_empty() {
        tabs.push(Tab::Myths);
    }
    if !c.quotes.is_empty() {
        tabs.push(Tab::Quotes);
    }
    tabs.push(Tab::Stats);
    if !c.legacy.is_empty() {
        tabs.push(Tab::Legacy);
    }
    tabs
}

/// How to open the explorer.
#[derive(Debug, Clone)]
pub struct Options {
    /// Open on the gallery of every creature.
    pub show_all: bool,
    /// Open on this creature's page.
    pub start_id: Option<String>,
    /// Skip Channel 25 tuning in.
    pub no_intro: bool,
    /// Start with the Myth or Fact? verdicts showing.
    pub reveal: bool,
    /// Draw in colour (off for `--no-color` and `NO_COLOR`).
    pub colour: bool,
    pub today: Date,
    pub rng: Rng,
}

pub struct Model<'a> {
    pub(super) vault: &'a Vault,
    pub(super) colour: bool,
    rng: Rng,

    pub(super) screen: Screen,
    /// Where the intro leads.
    next: Screen,

    /// The creature shown on the detail page.
    pub(super) index: usize,
    /// The gallery selection.
    pub(super) cursor: usize,
    /// An index into `tabs_for` the current creature.
    pub(super) tab: usize,
    pub(super) scroll: usize,
    pub(super) revealed: bool,
    pub(super) show_help: bool,

    pub(super) searching: bool,
    pub(super) query: String,
    pub(super) hits: Vec<Hit>,
    pub(super) hit_cursor: usize,

    pub(super) width: u16,
    pub(super) height: u16,

    seen: BTreeSet<String>,

    pub(super) frame: usize,
    /// Scatters the snow; fixed by `TERMINAL_OF_TINSEL_SEED`.
    pub(super) snow_seed: u64,
    pub(super) greeting: String,
    /// What's special about tonight.
    pub(super) notes: Vec<String>,
    sign_off: Option<&'static str>,
    quitting: bool,
}

impl<'a> Model<'a> {
    /// The vault must hold at least one creature.
    pub fn new(vault: &'a Vault, opts: Options) -> Self {
        let mut rng = opts.rng;
        let greeting = host::seasonal_greeting(opts.today.month(), opts.today.day())
            .unwrap_or_else(|| host::greeting(&mut rng));
        let mut m = Self {
            vault,
            colour: opts.colour,
            snow_seed: rng.next_u64(),
            rng,
            screen: Screen::Intro,
            next: Screen::Detail,
            index: 0,
            cursor: 0,
            tab: 0,
            scroll: 0,
            revealed: false,
            show_help: false,
            searching: false,
            query: String::new(),
            hits: Vec::new(),
            hit_cursor: 0,
            width: 0,
            height: 0,
            seen: BTreeSet::new(),
            frame: 0,
            greeting: greeting.to_string(),
            notes: calendar::notes(opts.today)
                .into_iter()
                .map(|n| format!("{} {}", n.emoji, n.text))
                .collect(),
            sign_off: None,
            quitting: false,
        };
        if opts.show_all {
            m.next = Screen::Gallery;
        }
        let start = opts
            .start_id
            .as_deref()
            .and_then(|id| vault.creatures.iter().position(|c| c.id == id));
        if let Some(i) = start {
            m.index = i;
            m.cursor = i;
            m.next = Screen::Detail;
        }
        // Asking for a creature by name means you know what you want: no intro.
        if opts.no_intro || start.is_some() {
            m.enter(m.next);
        }
        m.revealed = opts.reveal;
        m
    }

    pub(super) fn current(&self) -> &'a Creature {
        &self.vault.creatures[self.index]
    }

    /// Whether the current creature's pack is drawn as an engraving.
    pub(super) fn engraving(&self) -> bool {
        self.vault
            .pack_of(self.current())
            .is_some_and(|p| p.is_engraving())
    }

    pub(super) fn tabs(&self) -> Vec<Tab> {
        tabs_for(self.current())
    }

    /// The section showing on the detail page.
    pub(super) fn current_tab(&self) -> Tab {
        let tabs = self.tabs();
        tabs[self.tab.min(tabs.len() - 1)]
    }

    /// The usable size, assuming 80 by 24 until told otherwise.
    pub(super) fn size(&self) -> (usize, usize) {
        if self.width == 0 || self.height == 0 {
            (80, 24)
        } else {
            (usize::from(self.width), usize::from(self.height))
        }
    }

    pub(super) fn intro_done(&self) -> bool {
        self.frame >= intro::done_frame(&self.greeting)
    }

    /// How often the current screen animates, if it does.
    pub fn tick_every(&self) -> Option<Duration> {
        match self.screen {
            Screen::Intro => Some(INTRO_TICK),
            Screen::Detail if self.engraving() => Some(SNOW_TICK),
            _ => None,
        }
    }

    /// Advances the animation by a frame.
    pub fn on_tick(&mut self) {
        self.frame = self.frame.wrapping_add(1);
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        if (width, height) != (self.width, self.height) {
            self.width = width;
            self.height = height;
            self.clamp_scroll();
        }
    }

    pub fn quitting(&self) -> bool {
        self.quitting
    }

    /// The host's parting line, once the player has quit.
    pub fn sign_off(&self) -> Option<&'static str> {
        self.sign_off
    }

    /// The ids of the creature pages visited, in id order.
    pub fn seen(&self) -> Vec<String> {
        self.seen.iter().cloned().collect()
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            if key.code == KeyCode::Char('c') {
                self.quit();
            }
            return;
        }
        if self.screen == Screen::Intro {
            return self.intro_key(key.code);
        }
        if self.searching {
            return self.search_key(key.code);
        }
        match key.code {
            KeyCode::Char('q') => self.quit(),
            KeyCode::Char('?') => self.show_help = !self.show_help,
            KeyCode::Char('/') => {
                self.searching = true;
                self.query.clear();
                self.hits.clear();
                self.hit_cursor = 0;
            }
            code if self.screen == Screen::Gallery => self.gallery_key(code),
            code => self.detail_key(code),
        }
        self.clamp_scroll();
    }

    /// The first key finishes the intro; the next one enters.
    fn intro_key(&mut self, code: KeyCode) {
        if code == KeyCode::Char('q') {
            self.quit();
        } else if !self.intro_done() {
            self.frame = intro::done_frame(&self.greeting);
        } else {
            self.enter(self.next);
        }
    }

    fn quit(&mut self) {
        self.quitting = true;
        self.sign_off = Some(host::sign_off(&mut self.rng));
    }

    fn gallery_key(&mut self, code: KeyCode) {
        let n = self.vault.creatures.len();
        match code {
            KeyCode::Up | KeyCode::Char('k') => self.cursor = (self.cursor + n - 1) % n,
            KeyCode::Down | KeyCode::Char('j') => self.cursor = (self.cursor + 1) % n,
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = n - 1,
            KeyCode::Enter | KeyCode::Char(' ' | 'l') | KeyCode::Right => self.open(self.cursor),
            _ => {}
        }
    }

    fn detail_key(&mut self, code: KeyCode) {
        let n = self.vault.creatures.len();
        let tabs = self.tabs().len();
        let page = view::body_height(self).saturating_sub(2).max(1);
        match code {
            KeyCode::Down | KeyCode::Char('j') => self.scroll += 1,
            KeyCode::Up | KeyCode::Char('k') => self.scroll = self.scroll.saturating_sub(1),
            KeyCode::PageDown | KeyCode::Char(' ') => self.scroll += page,
            KeyCode::PageUp => self.scroll = self.scroll.saturating_sub(page),
            KeyCode::Home => self.scroll = 0,
            KeyCode::End => self.scroll = usize::MAX / 2,
            KeyCode::Right | KeyCode::Char('l' | 'n') => self.open((self.index + 1) % n),
            KeyCode::Left | KeyCode::Char('h' | 'p') => self.open((self.index + n - 1) % n),
            KeyCode::Tab => self.set_tab((self.tab + 1) % tabs),
            KeyCode::BackTab => self.set_tab((self.tab + tabs - 1) % tabs),
            KeyCode::Char('r') => self.revealed = !self.revealed,
            KeyCode::Char('g') | KeyCode::Esc | KeyCode::Backspace => {
                self.cursor = self.index;
                self.screen = Screen::Gallery;
            }
            KeyCode::Char(c @ '1'..='9') => {
                let i = c as usize - '1' as usize;
                if i < tabs {
                    self.set_tab(i);
                }
            }
            _ => {}
        }
    }

    fn search_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => {
                self.searching = false;
                return;
            }
            KeyCode::Enter => {
                if let Some(hit) = self.hits.get(self.hit_cursor) {
                    let index = hit.index;
                    self.searching = false;
                    self.open(index);
                }
                return;
            }
            KeyCode::Up => {
                self.hit_cursor = self.hit_cursor.saturating_sub(1);
                return;
            }
            KeyCode::Down => {
                if self.hit_cursor + 1 < self.hits.len() {
                    self.hit_cursor += 1;
                }
                return;
            }
            KeyCode::Backspace => {
                self.query.pop();
            }
            KeyCode::Char(c) => self.query.push(c),
            _ => return,
        }
        self.hits = search::search(&self.vault.creatures, &self.query);
        self.hit_cursor = 0;
    }

    /// Shows creature `i` on the detail page, keeping the same kind of
    /// section where the new creature has one.
    fn open(&mut self, i: usize) {
        let previous = (self.screen == Screen::Detail).then(|| self.current_tab());
        self.index = i;
        self.cursor = i;
        self.enter(Screen::Detail);
        self.scroll = 0;
        self.revealed = false;
        self.tab = previous
            .and_then(|t| self.tabs().iter().position(|&x| x == t))
            .unwrap_or(0);
    }

    /// Switches screen, noting creature pages as visited.
    fn enter(&mut self, screen: Screen) {
        self.screen = screen;
        if screen == Screen::Detail {
            self.seen.insert(self.current().id.clone());
        }
    }

    fn set_tab(&mut self, i: usize) {
        self.tab = i;
        self.scroll = 0;
    }

    fn clamp_scroll(&mut self) {
        if self.screen != Screen::Detail {
            return;
        }
        let max = view::body_lines(self)
            .len()
            .saturating_sub(view::body_height(self));
        self.scroll = self.scroll.min(max);
    }
}
