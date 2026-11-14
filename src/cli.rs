//! The command line, defined with clap's derive API.

use clap::{Parser, Subcommand};
use clap_complete::{ArgValueCandidates, Shell};

use crate::complete;

/// Channel 25's Midwinter Special: the creatures of Christmas folklore, live
/// in your terminal.
#[derive(Debug, Parser)]
#[command(
    name = "terminal-of-tinsel",
    version,
    about = "Channel 25's Midwinter Special: the creatures of Christmas folklore, live in your terminal",
    long_about = "Terminal of Tinsel brings the creatures of Christmas folklore to your terminal: \
St Nicholas's grim companions, the Yule Cat and the Yule Lads, the Mari Lwyd at the door \
and the spirits of A Christmas Carol, presented by your host, Auntie Aerial, live on \
Channel 25's Midwinter Special.\n\nStart here:  terminal-of-tinsel creature",
    after_help = "Set NO_COLOR or pass --no-color for plain output. Every command takes --json."
)]
pub struct Cli {
    /// Print the command's data as JSON instead of drawing it (errors become a JSON object on stderr)
    #[arg(long, global = true)]
    pub json: bool,

    /// Plain output: no colour, no emoji, ASCII borders (also set by NO_COLOR, or by piping)
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Only use creatures from these packs, e.g. --pack nordic,dickens
    #[arg(
        long,
        global = true,
        value_delimiter = ',',
        value_name = "PACK",
        add = ArgValueCandidates::new(complete::packs)
    )]
    pub pack: Vec<String>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// List every creature, grouped by pack
    #[command(visible_alias = "ls")]
    List,

    /// Tune in to the Midwinter Special, or read one creature's page
    #[command(
        visible_alias = "show",
        long_about = "With no name, opens the Midwinter Special: an interactive explorer of every \
creature, with a gallery, numbered sections, portraits and search. Press ? inside it for the keys.\n\n\
With a name, prints that creature's page. Nicknames and parts of names work, and accents are \
optional. Add --explore to open the explorer on that creature instead.",
        after_help = "Examples:\n  terminal-of-tinsel creature\n  terminal-of-tinsel creature krampus\n  terminal-of-tinsel creature yule cat --reveal\n  terminal-of-tinsel creature marley --explore\n  terminal-of-tinsel creature marley --json"
    )]
    Creature {
        /// The creature's name, nickname or part of one, e.g. krampus, "yule cat", nisse
        #[arg(num_args = 0.., value_name = "NAME", add = ArgValueCandidates::new(complete::creatures))]
        name: Vec<String>,

        /// Show the verdicts in Myth or Fact?
        #[arg(short, long)]
        reveal: bool,

        /// Open the explorer on the named creature instead of printing its page
        #[arg(short = 'x', long)]
        explore: bool,

        /// Open the explorer on the gallery of every creature
        #[arg(short, long)]
        all: bool,

        /// Skip Channel 25 tuning in
        #[arg(long)]
        no_intro: bool,
    },

    /// A random creature fact, or with --daily the Fact of the Night
    #[command(
        after_help = "With --daily you get the Fact of the Night: the same for everyone all day. \
From 1 to 24 December it's about the guest behind that night's advent door.\n\n\
Add it to your shell's startup file:\n  echo 'terminal-of-tinsel random --daily' >> ~/.zshrc"
    )]
    Random {
        /// Show the Fact of the Night (the same all day)
        #[arg(short, long)]
        daily: bool,

        /// Show the Fact of the Night for another date (YYYY-MM-DD)
        #[arg(long, value_name = "YYYY-MM-DD")]
        date: Option<String>,
    },

    /// Pull a cracker: the Christmas Cracker Quiz
    #[command(
        long_about = "Multiple-choice questions built from the creature data: facts, quotes, first \
appearances, film credits and myth checks. Every answer explains itself. Name a creature to be \
quizzed only about that one.\n\nWith --json the questions are printed with their answers instead.",
        after_help = "Examples:\n  terminal-of-tinsel quiz\n  terminal-of-tinsel quiz -n 5\n  terminal-of-tinsel quiz yule cat\n  terminal-of-tinsel quiz --pack nordic"
    )]
    Quiz {
        /// Only ask about this creature
        #[arg(num_args = 0.., value_name = "NAME", add = ArgValueCandidates::new(complete::creatures))]
        creature: Vec<String>,

        /// Number of questions
        #[arg(short = 'n', long, default_value_t = 10, value_name = "N")]
        questions: usize,
    },

    /// Guess the Guest: name the creature behind the frosted window
    #[command(
        long_about = "A creature's portrait is hidden behind a frosted window. Guess who it is, or \
press space to clear a little more frost and get clues, at the cost of points.\n\n\
With --json the rounds are printed with their answers instead."
    )]
    Guess {
        /// Number of rounds
        #[arg(short = 'n', long, default_value_t = 5, value_name = "N")]
        rounds: usize,
    },

    /// The Snowball Showdown: pit two creatures against each other
    #[command(
        long_about = "Stage a three-round showdown between two creatures. Each round tests \
strength, speed, cunning or dread, using each creature's stat card plus the roll of a die. Leave \
out one or both names for random contenders. Put a name of several words in quotes.",
        after_help = "Examples:\n  terminal-of-tinsel mash\n  terminal-of-tinsel mash krampus\n  terminal-of-tinsel mash \"yule cat\" gryla --fast"
    )]
    Mash {
        /// Up to two creatures, e.g. krampus "yule cat"
        #[arg(num_args = 0..=2, value_name = "NAME", add = ArgValueCandidates::new(complete::creatures))]
        creatures: Vec<String>,

        /// Skip the dramatic pauses
        #[arg(long)]
        fast: bool,
    },

    /// Look in your stocking: the creatures you've caught, your badges and records
    #[command(
        long_about = "Your stocking holds every creature you've caught by answering three questions \
about it, in the quiz or Guess the Guest, plus your badges and records. It's saved as JSON in \
your config directory; set TERMINAL_OF_TINSEL_HOME to keep it somewhere else."
    )]
    Stocking {
        #[command(subcommand)]
        action: Option<StockingAction>,
    },

    /// See tonight's Christmas double bill
    #[command(
        long_about = "Print a ticket for tonight's Channel 25 double bill. The line-up changes every \
night, so check back tomorrow, or use --shuffle for a different bill. From 1 to 24 December the \
guest behind that night's advent door tops the bill."
    )]
    Tonight {
        /// See the bill for another night (YYYY-MM-DD)
        #[arg(long, value_name = "YYYY-MM-DD")]
        date: Option<String>,

        /// Pick a random double bill instead of tonight's
        #[arg(long)]
        shuffle: bool,
    },

    /// Count the sleeps until Christmas
    Countdown {
        /// Pretend it's this date (YYYY-MM-DD)
        #[arg(long, value_name = "YYYY-MM-DD")]
        date: Option<String>,
    },

    /// Open the advent calendar: a different guest behind each door
    Advent {
        /// Pretend it's this date (YYYY-MM-DD)
        #[arg(long, value_name = "YYYY-MM-DD")]
        date: Option<String>,
    },

    /// List the packs of creatures, or make your own
    #[command(
        long_about = "Creatures come in packs. Use --pack with any command to keep to some of them, \
e.g. 'terminal-of-tinsel quiz --pack nordic'.\n\n\
Community packs live in the 'packs' folder of your config directory and load automatically. \
'packs new <id>' makes one with an example creature to edit. Broken files are skipped with a warning."
    )]
    Packs {
        #[command(subcommand)]
        action: Option<PacksAction>,
    },

    /// Print a shell completion script
    #[command(
        alias = "completion",
        long_about = "Print the script that teaches your shell to complete terminal-of-tinsel: \
commands, flags, creature names and pack ids, community packs included. The script asks the \
program itself what to offer, so it never goes out of date.",
        after_help = "For example, in ~/.zshrc:\n  source <(terminal-of-tinsel completions zsh)\n\nThen:\n  terminal-of-tinsel creature kra<TAB>"
    )]
    Completions {
        /// The shell to complete for
        shell: Shell,
    },
}

#[derive(Debug, Subcommand)]
pub enum PacksAction {
    /// Make a community pack with an example creature to edit
    #[command(
        after_help = "Example:\n  terminal-of-tinsel packs new office-party\n  terminal-of-tinsel list --pack office-party"
    )]
    New {
        /// The new pack's id: lowercase letters, numbers and dashes
        id: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum StockingAction {
    /// Empty your stocking and start again
    Reset {
        /// Don't ask for confirmation
        #[arg(short, long)]
        yes: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn the_command_line_is_well_formed() {
        Cli::command().debug_assert();
    }

    #[test]
    fn creature_names_can_be_several_words() {
        let cli = Cli::try_parse_from(["t", "creature", "yule", "cat"]).unwrap();
        match cli.command {
            Some(Command::Creature { name, .. }) => assert_eq!(name.join(" "), "yule cat"),
            other => panic!("parsed {other:?}"),
        }
    }

    #[test]
    fn creature_needs_no_name_and_takes_the_explorer_flags() {
        let cli = Cli::try_parse_from(["t", "creature", "--all", "--no-intro"]).unwrap();
        match cli.command {
            Some(Command::Creature {
                name,
                all,
                no_intro,
                explore,
                ..
            }) => {
                assert!(name.is_empty() && all && no_intro && !explore);
            }
            other => panic!("parsed {other:?}"),
        }
        let cli = Cli::try_parse_from(["t", "creature", "marley", "-x"]).unwrap();
        assert!(matches!(
            cli.command,
            Some(Command::Creature { explore: true, .. })
        ));
    }

    #[test]
    fn the_games_parse() {
        let cli = Cli::try_parse_from(["t", "quiz", "yule", "cat", "-n", "5"]).unwrap();
        assert!(matches!(
            cli.command,
            Some(Command::Quiz { questions: 5, ref creature }) if creature.join(" ") == "yule cat"
        ));
        let cli = Cli::try_parse_from(["t", "guess"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Guess { rounds: 5 })));
        let cli = Cli::try_parse_from(["t", "mash", "krampus", "yule cat", "--fast"]).unwrap();
        assert!(matches!(
            cli.command,
            Some(Command::Mash { fast: true, ref creatures }) if creatures.len() == 2
        ));
        assert!(Cli::try_parse_from(["t", "mash", "a", "b", "c"]).is_err());
    }

    #[test]
    fn packs_can_be_listed_and_made() {
        let cli = Cli::try_parse_from(["t", "packs"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Packs { action: None })));
        let cli = Cli::try_parse_from(["t", "packs", "new", "office-party"]).unwrap();
        assert!(matches!(
            cli.command,
            Some(Command::Packs { action: Some(PacksAction::New { ref id }) }) if id == "office-party"
        ));
        // Cobra calls it "completion"; both spellings work.
        assert!(Cli::try_parse_from(["t", "completion", "bash"]).is_ok());
    }

    #[test]
    fn the_stocking_has_a_reset() {
        let cli = Cli::try_parse_from(["t", "stocking"]).unwrap();
        assert!(matches!(
            cli.command,
            Some(Command::Stocking { action: None })
        ));
        let cli = Cli::try_parse_from(["t", "stocking", "reset", "-y"]).unwrap();
        assert!(matches!(
            cli.command,
            Some(Command::Stocking {
                action: Some(StockingAction::Reset { yes: true })
            })
        ));
    }

    #[test]
    fn packs_are_comma_separated_and_global() {
        let cli = Cli::try_parse_from(["t", "list", "--pack", "nordic,dickens"]).unwrap();
        assert_eq!(cli.pack, ["nordic", "dickens"]);
        // Any id parses: community packs aren't known until the vault loads.
        assert!(Cli::try_parse_from(["t", "list", "--pack", "elves"]).is_ok());
    }
}
