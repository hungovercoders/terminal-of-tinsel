//! The command line, defined with clap's derive API.

use clap::builder::PossibleValuesParser;
use clap::{Parser, Subcommand};
use clap_complete::Shell;

use crate::creatures::PACK_ORDER;

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
Channel 25's Midwinter Special.\n\nStart here:  terminal-of-tinsel list",
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
        value_parser = PossibleValuesParser::new(PACK_ORDER)
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

    /// Read a creature's page: facts, legend, myth or fact, stats and more
    #[command(
        visible_alias = "show",
        after_help = "Examples:\n  terminal-of-tinsel creature krampus\n  terminal-of-tinsel creature yule cat --reveal\n  terminal-of-tinsel creature marley --json"
    )]
    Creature {
        /// The creature's name, nickname or part of one, e.g. krampus, "yule cat", nisse
        #[arg(required = true, num_args = 1.., value_name = "NAME")]
        name: Vec<String>,

        /// Show the verdicts in Myth or Fact?
        #[arg(short, long)]
        reveal: bool,
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

    /// Print a shell completion script
    #[command(
        after_help = "For example, in ~/.zshrc:\n  source <(terminal-of-tinsel completions zsh)"
    )]
    Completions {
        /// The shell to complete for
        shell: Shell,
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
    fn packs_are_comma_separated_and_global() {
        let cli = Cli::try_parse_from(["t", "list", "--pack", "nordic,dickens"]).unwrap();
        assert_eq!(cli.pack, ["nordic", "dickens"]);
        assert!(Cli::try_parse_from(["t", "list", "--pack", "elves"]).is_err());
    }
}
