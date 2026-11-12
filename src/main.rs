//! Terminal of Tinsel: Channel 25's Midwinter Special, live in your terminal.

mod calendar;
mod cli;
mod creatures;
mod error;
mod explorer;
mod games;
mod guess;
mod host;
mod mash;
mod quiz;
mod render;
mod rng;
mod search;
mod tui;

use std::io::{self, IsTerminal, Write};
use std::process::ExitCode;

use clap::error::{ContextKind, ContextValue, ErrorKind};
use clap::{CommandFactory, Parser};
use serde::Serialize;

use cli::{Cli, Command};
use creatures::Vault;
use error::{Code, Result, TinselError};
use render::Painter;
use rng::Rng;

fn main() -> ExitCode {
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

fn run(cli: Cli) -> Result<()> {
    let no_colour = cli.no_color || std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty());
    let plain = no_colour || !io::stdout().is_terminal();
    let painter = Painter::new(plain, terminal_width());
    let vault = Vault::load().only_packs(&cli.pack)?;
    let json = cli.json;

    match cli.command {
        None => {
            let mut rng = Rng::from_env_or_clock();
            if json {
                return print_json(&serde_json::json!({
                    "host": host::NAME,
                    "channel": host::CHANNEL,
                    "greeting": host::greeting(&mut rng),
                    "sleepsUntilChristmas": calendar::sleeps_until_christmas(calendar::today()),
                }));
            }
            let greeting = host::greeting(&mut rng);
            print(&render::welcome(
                &painter,
                &vault,
                calendar::today(),
                greeting,
            ));
        }

        Some(Command::List) => {
            if json {
                return print_json(&vault.creatures);
            }
            print(&render::list(&painter, &vault));
        }

        Some(Command::Creature {
            name,
            reveal,
            explore,
            all,
            no_intro,
        }) => {
            let named = if name.is_empty() {
                None
            } else {
                Some(vault.find(&name.join(" "))?)
            };
            if json {
                return match named {
                    Some(c) => print_json(c),
                    None => print_json(&vault.creatures),
                };
            }
            match named {
                Some(c) if !explore && !all => {
                    let pack = vault.pack_of(c).map_or(c.pack.as_str(), |p| &p.name);
                    print(&render::creature(&painter, c, pack, reveal));
                }
                _ => {
                    require_terminal(
                        "the Midwinter Special",
                        "use --json for the data, name a creature to print its page, or tune in from a shell",
                    )?;
                    let visit = explorer::run(
                        &vault,
                        explorer::Options {
                            show_all: all,
                            start_id: named.map(|c| c.id.clone()),
                            no_intro,
                            reveal,
                            colour: !no_colour,
                            today: calendar::today(),
                            rng: Rng::from_env_or_clock(),
                        },
                    )?;
                    if let Some(line) = visit.sign_off {
                        let text = format!("{}{line}", painter.glyph("📺"));
                        print(&(painter.italic(&text, render::DIM) + "\n"));
                    }
                    let met = visit.seen.len();
                    if met > 0 {
                        let guests = if met == 1 { "guest" } else { "guests" };
                        let text = format!(
                            "You met {met} of tonight's {} {guests}.",
                            vault.creatures.len()
                        );
                        print(&(painter.paint(&text, render::DIM) + "\n"));
                    }
                }
            }
        }

        Some(Command::Random { daily, date }) => {
            let daily = daily || date.is_some();
            let today = calendar::parse_date(date.as_deref())?;
            let mut rng = if daily {
                Rng::new(calendar::daily_seed(today))
            } else {
                Rng::from_env_or_clock()
            };
            if vault.creatures.is_empty() {
                return Err(TinselError::new(Code::Usage, "no creatures in those packs"));
            }
            let (c, fact) = match calendar::advent_guest(today, &vault).filter(|_| daily) {
                Some((_, c, fact)) => (c, fact),
                None => {
                    let c = rng.pick(&vault.creatures);
                    (c, rng.pick(&c.facts).clone())
                }
            };
            let notes = if daily {
                calendar::notes(today)
            } else {
                Vec::new()
            };
            let note_texts: Vec<String> = notes.iter().map(|n| n.text.clone()).collect();
            if json {
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
            print(&render::fact_card(&painter, &title, c, &fact, &notes, quip));
        }

        Some(Command::Quiz {
            creature,
            questions,
        }) => {
            if questions == 0 {
                return Err(TinselError::new(
                    Code::BadFlag,
                    "--questions must be at least 1",
                ));
            }
            let focus = if creature.is_empty() {
                None
            } else {
                Some(vault.find(&creature.join(" "))?.id.as_str())
            };
            let mut rng = Rng::from_env_or_clock();
            let quiz = quiz::generate(&mut rng, &vault.creatures, questions, focus);
            if quiz.is_empty() {
                return Err(
                    TinselError::new(Code::Usage, "couldn't find any questions to ask")
                        .with_hint("try a different --pack"),
                );
            }
            if json {
                return print_json(&quiz);
            }
            require_terminal(
                "the Christmas Cracker Quiz",
                "use --json for the questions and answers, or play from a shell",
            )?;
            let mut app = games::QuizApp::new(quiz, rng, !no_colour);
            tui::run(&mut app, "the Christmas Cracker Quiz")?;
            let result = app.into_result();
            // The stocking records what was won here, once it exists.
            print(&render::quiz_summary(&painter, &result));
        }

        Some(Command::Guess { rounds }) => {
            if rounds == 0 {
                return Err(TinselError::new(
                    Code::BadFlag,
                    "--rounds must be at least 1",
                ));
            }
            let mut rng = Rng::from_env_or_clock();
            let game = guess::rounds(&mut rng, &vault.creatures, rounds);
            if game.is_empty() {
                return Err(TinselError::new(
                    Code::Usage,
                    "Guess the Guest needs at least 3 different creatures, including one with a portrait",
                )
                .with_hint("try a different --pack"));
            }
            if json {
                let out: Vec<guess::RoundJson> = game.iter().map(Into::into).collect();
                return print_json(&out);
            }
            require_terminal(
                "Guess the Guest",
                "use --json for the rounds and answers, or play from a shell",
            )?;
            let mut app = games::GuessApp::new(game, rng, !no_colour);
            tui::run(&mut app, "Guess the Guest")?;
            print(&render::guess_summary(&painter, &app.into_result()));
        }

        Some(Command::Mash { creatures, fast }) => {
            let mut rng = Rng::from_env_or_clock();
            let mut corners = Vec::new();
            for name in &creatures {
                corners.push(vault.find(name)?);
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
            // Fill the empty corners from creatures not already in one.
            // Picking from this pool, not retrying at random, can't loop.
            while corners.len() < 2 {
                let pool: Vec<&creatures::Creature> = vault
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
            let bout = mash::fight(&mut rng, corners[0], corners[1]);
            if json {
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
            print(&render::mash_header(&painter, &bout));
            pause(1200);
            for round in &bout.rounds {
                print(&render::mash_round(&painter, &bout, round));
                pause(1500);
            }
            print(&render::mash_result(&painter, &bout));
        }

        Some(Command::Countdown { date }) => {
            let today = calendar::parse_date(date.as_deref())?;
            if json {
                let guest = calendar::advent_guest(today, &vault);
                return print_json(&CountdownJson {
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
                    notes: calendar::notes(today).into_iter().map(|n| n.text).collect(),
                });
            }
            print(&render::countdown(&painter, &vault, today));
        }

        Some(Command::Advent { date }) => {
            let today = calendar::parse_date(date.as_deref())?;
            if json {
                let doors: Vec<_> = vault
                    .advent
                    .iter()
                    .map(|n| DoorJson {
                        night: n.night,
                        id: &n.id,
                        name: vault.get(&n.id).map(|c| c.name.as_str()),
                        note: &n.note,
                        door: render::door(n.night, today),
                    })
                    .collect();
                return print_json(
                    &serde_json::json!({ "date": today.to_string(), "doors": doors }),
                );
            }
            print(&render::advent(&painter, &vault, today));
        }

        Some(Command::Completions { shell }) => {
            let mut cmd = Cli::command();
            let mut buf = Vec::new();
            clap_complete::generate(shell, &mut cmd, "terminal-of-tinsel", &mut buf);
            print(&String::from_utf8_lossy(&buf));
        }
    }
    Ok(())
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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    notes: Vec<String>,
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
