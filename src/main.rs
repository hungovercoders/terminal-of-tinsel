//! Terminal of Tinsel: Channel 25's Midwinter Special, live in your terminal.

mod calendar;
mod cli;
mod complete;
mod creatures;
mod error;
mod explorer;
mod games;
mod guess;
mod host;
mod mash;
mod packs;
mod plain;
mod quiz;
mod render;
mod rng;
mod search;
#[cfg(test)]
mod shelldocs;
mod stocking;
mod store;
mod tui;

use std::io::{self, IsTerminal, Write};
use std::process::ExitCode;

use clap::error::{ContextKind, ContextValue, ErrorKind};
use clap::{CommandFactory, Parser};
use serde::Serialize;

use cli::{Cli, Command, PacksAction, StockingAction};
use creatures::Vault;
use error::{Code, Result, TinselError};
use render::Painter;
use rng::Rng;
use stocking::{Kind, Outcome};

fn main() -> ExitCode {
    // When the shell asks for completions (COMPLETE=bash and so on), answer
    // and leave: nothing else runs.
    clap_complete::CompleteEnv::with_factory(Cli::command).complete();

    let args: Vec<String> = std::env::args().collect();
    let wants_json = args.iter().any(|a| a == "--json");
    let cli = match Cli::try_parse_from(&args) {
        Ok(cli) => cli,
        Err(e) if wants_json && !is_help(&e) => {
            eprintln!("{}", clap_error(&e).to_json());
            return ExitCode::FAILURE;
        }
        Err(e) => e.exit(),
    };
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            if wants_json {
                eprintln!("{}", e.to_json());
            } else {
                eprintln!("error: {}", e.message);
                if let Some(hint) = &e.hint {
                    eprintln!("hint: {hint}");
                }
            }
            ExitCode::FAILURE
        }
    }
}

/// What every command needs: how to draw, which creatures are in play, and
/// whether to print data instead.
struct Show {
    painter: Painter,
    /// Every creature, whatever `--pack` says: the stocking's badges count
    /// them all.
    everything: Vault,
    /// The creatures in play.
    vault: Vault,
    json: bool,
    /// Whether the full-screen pages are in colour.
    colour: bool,
}

fn run(cli: Cli) -> Result<()> {
    let no_colour = cli.no_color || std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty());
    let plain = no_colour || !io::stdout().is_terminal();
    let mut everything = Vault::load();
    // Community packs: anything broken is reported and skipped, never fatal.
    if let Ok(dir) = store::dir() {
        for warning in everything.add_community(&store::packs_dir(&dir)) {
            eprintln!("warning: community pack: {warning}");
        }
    }
    let show = Show {
        painter: Painter::new(plain, terminal_width()),
        vault: everything.clone().only_packs(&cli.pack)?,
        everything,
        json: cli.json,
        colour: !no_colour,
    };
    show.dispatch(cli.command)
}

impl Show {
    /// The commands that show something.
    fn dispatch(&self, command: Option<Command>) -> Result<()> {
        match command {
            None => self.welcome(),
            Some(Command::List) => self.list(),
            Some(Command::Creature {
                name,
                reveal,
                explore,
                all,
                no_intro,
            }) => self.creature(&name.join(" "), reveal, explore || all, all, no_intro),
            Some(Command::Random { daily, date }) => self.random(daily, date.as_deref()),
            Some(Command::Tonight { date, shuffle }) => self.tonight(date.as_deref(), shuffle),
            Some(Command::Countdown { date }) => self.countdown(date.as_deref()),
            Some(Command::Advent { date }) => self.advent(date.as_deref()),
            Some(other) => self.play_or_keep(other),
        }
    }

    /// The games, and the commands that look after what's kept on disk.
    fn play_or_keep(&self, command: Command) -> Result<()> {
        match command {
            Command::Quiz {
                creature,
                questions,
            } => self.quiz(&creature.join(" "), questions),
            Command::Guess { rounds } => self.guess(rounds),
            Command::Mash { creatures, fast } => self.mash(&creatures, fast),
            Command::Stocking { action } => self.stocking(action),
            Command::Packs { action } => self.packs(action),
            Command::Completions { shell } => complete_script(shell).map(|s| print(&s)),
            // Every other command is handled in `dispatch`, which sends only
            // these here. Looping back would never end if one were missed.
            _ => Err(TinselError::new(
                Code::Usage,
                "that command isn't wired up yet",
            )),
        }
    }

    fn welcome(&self) -> Result<()> {
        let mut rng = Rng::from_env_or_clock();
        let greeting = host::greeting(&mut rng);
        if self.json {
            return print_json(&serde_json::json!({
                "host": host::NAME,
                "channel": host::CHANNEL,
                "greeting": greeting,
                "sleepsUntilChristmas": calendar::sleeps_until_christmas(calendar::today()),
            }));
        }
        print(&render::welcome(
            &self.painter,
            &self.vault,
            calendar::today(),
            greeting,
        ));
        Ok(())
    }

    fn list(&self) -> Result<()> {
        if self.json {
            return print_json(&self.vault.creatures);
        }
        print(&render::list(&self.painter, &self.vault));
        Ok(())
    }

    /// A creature's page, or with no name (or when asked to) the explorer.
    fn creature(
        &self,
        name: &str,
        reveal: bool,
        explore: bool,
        gallery: bool,
        no_intro: bool,
    ) -> Result<()> {
        let named = match name {
            "" => None,
            name => Some(self.vault.find(name)?),
        };
        if self.json {
            return match named {
                Some(c) => print_json(c),
                None => print_json(&self.vault.creatures),
            };
        }
        if let Some(c) = named.filter(|_| !explore) {
            let pack = self.vault.pack_of(c).map_or(c.pack.as_str(), |p| &p.name);
            print(&render::creature(&self.painter, c, pack, reveal));
            return Ok(());
        }
        require_terminal(
            "the Midwinter Special",
            "use --json for the data, name a creature to print its page, or tune in from a shell",
        )?;
        let visit = explorer::run(
            &self.vault,
            explorer::Options {
                show_all: gallery,
                start_id: named.map(|c| c.id.clone()),
                no_intro,
                reveal,
                colour: self.colour,
                today: calendar::today(),
                rng: Rng::from_env_or_clock(),
            },
        )?;
        if let Some(line) = visit.sign_off {
            let text = format!("{}{line}", self.painter.glyph("📺"));
            print(&(self.painter.italic(&text, render::DIM) + "\n"));
        }
        if !visit.seen.is_empty() {
            let mut outcome = Outcome::new(Kind::Explore, calendar::now());
            outcome.seen = visit.seen;
            self.record(&outcome);
        }
        Ok(())
    }

    fn random(&self, daily: bool, date: Option<&str>) -> Result<()> {
        let daily = daily || date.is_some();
        let today = calendar::parse_date(date)?;
        let mut rng = if daily {
            Rng::new(calendar::daily_seed(today))
        } else {
            Rng::from_env_or_clock()
        };
        if self.vault.creatures.is_empty() {
            return Err(TinselError::new(Code::Usage, "no creatures in those packs"));
        }
        let (c, fact) = match calendar::advent_guest(today, &self.vault).filter(|_| daily) {
            Some((_, c, fact)) => (c, fact),
            None => {
                let c = rng.pick(&self.vault.creatures);
                (c, rng.pick(&c.facts).clone())
            }
        };
        let notes = if daily {
            calendar::notes(today, &self.vault.creatures)
        } else {
            Vec::new()
        };
        if self.json {
            let note_texts: Vec<String> = notes.iter().map(|n| n.text.clone()).collect();
            return print_json(&FactJson {
                id: &c.id,
                creature: &c.name,
                fact: &fact,
                date: daily.then(|| today.to_string()),
                notes: &note_texts,
            });
        }
        let title = if daily {
            format!("FACT OF THE NIGHT · {}", today.strftime("%A %-d %B"))
        } else {
            "RANDOM FESTIVE FACT".to_string()
        };
        let quip = host::quip(&mut rng);
        print(&render::fact_card(
            &self.painter,
            &title,
            c,
            &fact,
            &notes,
            quip,
        ));
        Ok(())
    }

    fn quiz(&self, creature: &str, questions: usize) -> Result<()> {
        if questions == 0 {
            return Err(TinselError::new(
                Code::BadFlag,
                "--questions must be at least 1",
            ));
        }
        let focus = match creature {
            "" => None,
            name => Some(self.vault.find(name)?.id.as_str()),
        };
        let mut rng = Rng::from_env_or_clock();
        let quiz = quiz::generate(&mut rng, &self.vault.creatures, questions, focus);
        if quiz.is_empty() {
            return Err(
                TinselError::new(Code::Usage, "couldn't find any questions to ask")
                    .with_hint("try a different --pack"),
            );
        }
        if self.json {
            return print_json(&quiz);
        }
        require_terminal(
            "the Christmas Cracker Quiz",
            "use --json for the questions and answers, or play from a shell",
        )?;
        let mut app = games::QuizApp::new(quiz, rng, self.colour);
        tui::run(&mut app, "the Christmas Cracker Quiz")?;
        let result = app.into_result();
        print(&render::quiz_summary(&self.painter, &result));
        if result.score().1 > 0 {
            let mut outcome = Outcome::new(Kind::Quiz, calendar::now());
            outcome.correct = result.correct_by();
            outcome.score = result.percent() as u32;
            outcome.total = result.questions.len() as u32;
            outcome.completed = result.completed;
            self.record(&outcome);
        }
        Ok(())
    }

    fn guess(&self, rounds: usize) -> Result<()> {
        if rounds == 0 {
            return Err(TinselError::new(
                Code::BadFlag,
                "--rounds must be at least 1",
            ));
        }
        let mut rng = Rng::from_env_or_clock();
        let game = guess::rounds(&mut rng, &self.vault.creatures, rounds);
        if game.is_empty() {
            return Err(TinselError::new(
                Code::Usage,
                "Guess the Guest needs at least 3 different creatures, including one with a portrait",
            )
            .with_hint("try a different --pack"));
        }
        if self.json {
            let out: Vec<guess::RoundJson> = game.iter().map(Into::into).collect();
            return print_json(&out);
        }
        require_terminal(
            "Guess the Guest",
            "use --json for the rounds and answers, or play from a shell",
        )?;
        let mut app = games::GuessApp::new(game, rng, self.colour);
        tui::run(&mut app, "Guess the Guest")?;
        let result = app.into_result();
        print(&render::guess_summary(&self.painter, &result));
        if result.played() > 0 {
            let mut outcome = Outcome::new(Kind::Guess, calendar::now());
            outcome.correct = result.correct_by();
            outcome.score = result.score() as u32;
            outcome.total = result.rounds.len() as u32;
            outcome.completed = result.completed;
            outcome.frost_reader = result.frost_reader();
            self.record(&outcome);
        }
        Ok(())
    }

    /// The two creatures for a showdown: the ones named, and the rest drawn
    /// from creatures not already in a corner. Picking from that pool, not
    /// retrying at random, can't loop.
    fn corners(&self, names: &[String], rng: &mut Rng) -> Result<[&creatures::Creature; 2]> {
        let mut corners = Vec::new();
        for name in names {
            corners.push(self.vault.find(name)?);
        }
        if let [a, b] = corners.as_slice()
            && a.id == b.id
        {
            return Err(TinselError::new(
                Code::Usage,
                format!("{} can't throw snowballs at itself", a.name),
            )
            .with_hint("pick two different creatures"));
        }
        while corners.len() < 2 {
            let pool: Vec<&creatures::Creature> = self
                .vault
                .creatures
                .iter()
                .filter(|c| corners.iter().all(|taken| taken.id != c.id))
                .collect();
            if pool.is_empty() {
                return Err(TinselError::new(
                    Code::Usage,
                    "the Snowball Showdown needs at least two different creatures",
                )
                .with_hint("try a different --pack"));
            }
            corners.push(pool[rng.below(pool.len())]);
        }
        Ok([corners[0], corners[1]])
    }

    fn mash(&self, names: &[String], fast: bool) -> Result<()> {
        let mut rng = Rng::from_env_or_clock();
        let [a, b] = self.corners(names, &mut rng)?;
        let bout = mash::fight(&mut rng, a, b);
        if self.json {
            return print_json(&MashJson {
                a: bout.a.into(),
                b: bout.b.into(),
                rounds: &bout.rounds,
                winner: &bout.winner().id,
                finale: &bout.finale,
            });
        }
        let pause = |ms: u64| {
            if !fast && io::stdout().is_terminal() {
                std::thread::sleep(std::time::Duration::from_millis(ms));
            }
        };
        print(&render::mash_header(&self.painter, &bout));
        pause(1200);
        for round in &bout.rounds {
            print(&render::mash_round(&self.painter, &bout, round));
            pause(1500);
        }
        print(&render::mash_result(&self.painter, &bout));
        self.record(&Outcome::new(Kind::Mash, calendar::now()));
        Ok(())
    }

    fn stocking(&self, action: Option<StockingAction>) -> Result<()> {
        let dir = store::dir()?;
        if let Some(StockingAction::Reset { yes }) = action {
            return self.stocking_reset(&dir, yes);
        }
        let progress = store::load(&dir)?;
        if self.json {
            return print_json(&progress);
        }
        print(&render::stocking(
            &self.painter,
            &progress,
            &self.everything,
        ));
        let path = store::tilde(&store::path_in(&dir));
        print(&format!(
            "\n{}\n",
            self.painter
                .paint(&format!("Progress is saved in {path}"), render::DIM)
        ));
        Ok(())
    }

    fn stocking_reset(&self, dir: &std::path::Path, yes: bool) -> Result<()> {
        if !yes && !confirm_reset()? {
            if self.json {
                return print_json(&serde_json::json!({ "reset": false }));
            }
            print("Phew. Your stocking is untouched.\n");
            return Ok(());
        }
        store::reset(dir)?;
        if self.json {
            return print_json(&serde_json::json!({ "reset": true }));
        }
        print(
            "The stocking is tipped out on the hearthrug, and every creature has scampered off. Your progress has been reset.\n",
        );
        Ok(())
    }

    fn tonight(&self, date: Option<&str>, shuffle: bool) -> Result<()> {
        let today = calendar::parse_date(date)?;
        let too_few = || {
            TinselError::new(Code::Usage, "a double bill needs at least two creatures")
                .with_hint("try a different --pack")
        };
        if self.vault.creatures.len() < 2 {
            return Err(too_few());
        }
        let (first, second) = if shuffle {
            let mut bill: Vec<&creatures::Creature> = self.vault.creatures.iter().collect();
            Rng::from_env_or_clock().shuffle(&mut bill);
            (bill[0], bill[1])
        } else {
            calendar::double_bill(today, &self.vault).ok_or_else(too_few)?
        };
        let notes = calendar::notes(today, &self.vault.creatures);
        if self.json {
            return print_json(&TonightJson {
                date: today.to_string(),
                features: [
                    FeatureJson::new(render::FIRST_FEATURE, first),
                    FeatureJson::new(render::SECOND_FEATURE, second),
                ],
                notes: notes.into_iter().map(|n| n.text).collect(),
            });
        }
        print(&render::ticket(&self.painter, today, first, second));
        print(&render::fact_card(
            &self.painter,
            "FROM THE HOST",
            first,
            &host::intro(first),
            &notes,
            "",
        ));
        Ok(())
    }

    fn countdown(&self, date: Option<&str>) -> Result<()> {
        let today = calendar::parse_date(date)?;
        if !self.json {
            print(&render::countdown(&self.painter, &self.vault, today));
            return Ok(());
        }
        let guest = calendar::advent_guest(today, &self.vault);
        print_json(&CountdownJson {
            date: today.to_string(),
            christmas: calendar::is_christmas(today),
            sleeps_until_christmas: calendar::sleeps_until_christmas(today),
            advent: guest.map(|(night, c, fact)| AdventJson {
                night: night.night,
                id: &c.id,
                name: &c.name,
                note: &night.note,
                fact,
            }),
            twelve_days: calendar::twelve_days(today),
            yule_lads_in_town: calendar::yule_lads_in_town(today),
            moon: calendar::moon(today).into(),
            next_special_night: calendar::next_special_night(today, &self.vault.creatures)
                .map(Into::into),
            notes: calendar::notes(today, &self.vault.creatures)
                .into_iter()
                .map(|n| n.text)
                .collect(),
        })
    }

    fn advent(&self, date: Option<&str>) -> Result<()> {
        let today = calendar::parse_date(date)?;
        if !self.json {
            print(&render::advent(&self.painter, &self.vault, today));
            return Ok(());
        }
        let doors: Vec<_> = self
            .vault
            .advent
            .iter()
            .map(|n| DoorJson {
                night: n.night,
                id: &n.id,
                name: self.vault.get(&n.id).map(|c| c.name.as_str()),
                note: &n.note,
                door: render::door(n.night, today),
            })
            .collect();
        print_json(&serde_json::json!({ "date": today.to_string(), "doors": doors }))
    }

    fn packs(&self, action: Option<PacksAction>) -> Result<()> {
        if let Some(PacksAction::New { id }) = action {
            return self.packs_new(&id);
        }
        if self.json {
            let out: Vec<PackJson> = self
                .everything
                .by_pack()
                .into_iter()
                .map(|(pack, members)| PackJson {
                    id: &pack.id,
                    name: &pack.name,
                    description: &pack.description,
                    source: &pack.source,
                    creatures: members.iter().map(|c| c.id.as_str()).collect(),
                })
                .collect();
            return print_json(&out);
        }
        let dir = store::dir().ok().map(|d| store::packs_dir(&d));
        print(&render::packs(
            &self.painter,
            &self.everything,
            dir.as_deref(),
        ));
        Ok(())
    }

    fn packs_new(&self, id: &str) -> Result<()> {
        let dir = packs::scaffold(&self.everything, &store::packs_dir(&store::dir()?), id)?;
        let creature = packs::example_id(id);
        if self.json {
            return print_json(&serde_json::json!({
                "pack": id,
                "creature": creature,
                "path": dir.display().to_string(),
            }));
        }
        print(&format!(
            "Created {}\n\n\
             Edit {creature}.json (and {creature}.txt for its portrait), then try:\n  \
             terminal-of-tinsel creature {creature}\n  \
             terminal-of-tinsel quiz --pack {id}\n\n\
             Only name, description and facts are required.\n",
            store::tilde(&dir)
        ));
        Ok(())
    }

    /// Saves an outcome to the player's stocking and announces anything new.
    /// Trouble with the save file is reported but never stops the fun.
    fn record(&self, outcome: &Outcome) {
        let saved = store::dir().and_then(|dir| {
            let mut progress = store::load(&dir)?;
            let unlocks = stocking::apply(&mut progress, &self.everything.creatures, outcome);
            store::save(&dir, &progress)?;
            Ok(unlocks)
        });
        match saved {
            Ok(unlocks) => print(&render::unlocks(&self.painter, &unlocks)),
            Err(e) => eprintln!("warning: this session wasn't saved: {}", e.message),
        }
    }
}

/// Asks before emptying the stocking. Without a terminal to ask on, it
/// refuses rather than guess.
fn confirm_reset() -> Result<bool> {
    if !io::stdin().is_terminal() {
        return Err(TinselError::new(
            Code::NoTerminal,
            "refusing to empty the stocking without a terminal to confirm on",
        )
        .with_hint("pass -y to reset without asking"));
    }
    eprint!("This tips out every creature and forgets all your badges. Type 'yes' to confirm: ");
    let mut answer = String::new();
    let _ = io::stdin().read_line(&mut answer);
    Ok(answer.trim().to_lowercase() == "yes")
}

/// The script that registers dynamic completion with a shell: the same one
/// `COMPLETE=<shell> terminal-of-tinsel` prints.
fn complete_script(shell: clap_complete::Shell) -> Result<String> {
    let name = shell.to_string();
    let shells = clap_complete::env::Shells::builtins();
    let completer = shells.completer(&name).ok_or_else(|| {
        TinselError::new(Code::Usage, format!("no completions for {name}"))
            .with_hint("choose from: bash, elvish, fish, powershell, zsh")
    })?;
    let mut script = Vec::new();
    let bin = "terminal-of-tinsel";
    completer
        .write_registration("COMPLETE", bin, bin, bin, &mut script)
        .map_err(|e| TinselError::new(Code::Usage, e.to_string()))?;
    Ok(String::from_utf8_lossy(&script).into_owned())
}

/// A pack as `packs --json` prints it.
#[derive(Serialize)]
struct PackJson<'a> {
    id: &'a str,
    name: &'a str,
    description: &'a str,
    /// "built-in", or the directory a community pack was loaded from.
    source: &'a str,
    creatures: Vec<&'a str>,
}

/// A showdown as `--json` prints it: who fought, round by round, and who won.
#[derive(Serialize)]
struct MashJson<'a> {
    a: FighterJson<'a>,
    b: FighterJson<'a>,
    rounds: &'a [mash::Round],
    /// The winner's id.
    winner: &'a str,
    finale: &'a str,
}

#[derive(Serialize)]
struct FighterJson<'a> {
    id: &'a str,
    name: &'a str,
}

impl<'a> From<&'a creatures::Creature> for FighterJson<'a> {
    fn from(c: &'a creatures::Creature) -> Self {
        Self {
            id: &c.id,
            name: &c.name,
        }
    }
}

#[derive(Serialize)]
struct FactJson<'a> {
    id: &'a str,
    creature: &'a str,
    fact: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    date: Option<String>,
    #[serde(skip_serializing_if = "<[String]>::is_empty")]
    notes: &'a [String],
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CountdownJson<'a> {
    date: String,
    christmas: bool,
    sleeps_until_christmas: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    advent: Option<AdventJson<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    twelve_days: Option<u8>,
    yule_lads_in_town: u8,
    moon: MoonJson,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_special_night: Option<SpecialNightJson>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    notes: Vec<String>,
}

#[derive(Serialize)]
struct MoonJson {
    name: &'static str,
    emoji: &'static str,
    /// From 0 (new) to 1 (full), to two decimal places.
    illumination: f64,
}

impl From<calendar::Moon> for MoonJson {
    fn from(m: calendar::Moon) -> Self {
        Self {
            name: m.name,
            emoji: m.emoji,
            illumination: (m.illumination * 100.0).round() / 100.0,
        }
    }
}

#[derive(Serialize)]
struct SpecialNightJson {
    name: String,
    date: String,
    sleeps: i64,
    /// The creature whose film has its anniversary, if it's one of those.
    #[serde(skip_serializing_if = "Option::is_none")]
    creature: Option<String>,
}

impl From<calendar::SpecialNight> for SpecialNightJson {
    fn from(n: calendar::SpecialNight) -> Self {
        Self {
            name: n.name,
            date: n.date.to_string(),
            sleeps: n.sleeps,
            creature: n.creature,
        }
    }
}

/// Tonight's double bill as `--json` prints it.
#[derive(Serialize)]
struct TonightJson<'a> {
    date: String,
    features: [FeatureJson<'a>; 2],
    #[serde(skip_serializing_if = "Vec::is_empty")]
    notes: Vec<String>,
}

#[derive(Serialize)]
struct FeatureJson<'a> {
    time: &'static str,
    creature: &'a str,
    name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    film: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    year: Option<i32>,
}

impl<'a> FeatureJson<'a> {
    fn new(time: &'static str, c: &'a creatures::Creature) -> Self {
        Self {
            time,
            creature: &c.id,
            name: &c.name,
            film: c.film.as_ref().map(|f| f.title.as_str()),
            year: c.film.as_ref().map(|f| f.year),
        }
    }
}

#[derive(Serialize)]
struct AdventJson<'a> {
    night: u8,
    id: &'a str,
    name: &'a str,
    note: &'a str,
    fact: String,
}

#[derive(Serialize)]
struct DoorJson<'a> {
    night: u8,
    id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<&'a str>,
    note: &'a str,
    door: render::Door,
}

/// Writes to stdout, and exits quietly if the reader has gone away
/// (`terminal-of-tinsel list | head`).
fn print(text: &str) {
    let mut out = io::stdout().lock();
    if let Err(e) = out.write_all(text.as_bytes()).and_then(|()| out.flush()) {
        if e.kind() == io::ErrorKind::BrokenPipe {
            std::process::exit(0);
        }
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn print_json<T: Serialize + ?Sized>(value: &T) -> Result<()> {
    let text = serde_json::to_string_pretty(value)
        .map_err(|e| TinselError::new(Code::Usage, e.to_string()))?;
    print(&(text + "\n"));
    Ok(())
}

/// The guard in front of every screen that takes over the terminal. Run from
/// a script, a pipe or CI, such a screen would hang waiting for a key or spray
/// escape codes into the pipe; this says plainly what it needs instead.
/// `what` names the screen in the show's own words; `instead` says what a
/// script should use.
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

fn terminal_width() -> usize {
    terminal_size::terminal_size()
        .map(|(w, _)| usize::from(w.0))
        .or_else(|| std::env::var("COLUMNS").ok()?.parse().ok())
        .unwrap_or(80)
}

fn is_help(e: &clap::Error) -> bool {
    matches!(
        e.kind(),
        ErrorKind::DisplayHelp
            | ErrorKind::DisplayVersion
            | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
    )
}

/// A clap parse error as a coded error, for `--json`.
fn clap_error(e: &clap::Error) -> TinselError {
    let arg = match e.get(ContextKind::InvalidArg) {
        Some(ContextValue::String(s)) => s.clone(),
        _ => String::new(),
    };
    let code = if arg.starts_with("--pack") {
        Code::BadPack
    } else {
        Code::BadFlag
    };
    let message = e
        .render()
        .to_string()
        .lines()
        .next()
        .unwrap_or("bad arguments")
        .trim_start_matches("error: ")
        .to_string();
    TinselError::new(code, message).with_hint("try 'terminal-of-tinsel --help'")
}
