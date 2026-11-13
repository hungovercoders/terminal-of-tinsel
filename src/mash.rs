//! The Snowball Showdown: a best-of-three bout between two creatures,
//! decided by their stat cards and a roll of the dice.

use serde::Serialize;

use crate::creatures::{Creature, Stats};
use crate::quiz::mid_sentence;
use crate::rng::Rng;

/// One exchange in the bout.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Round {
    pub number: u8,
    pub stat: &'static str,
    pub roll_a: u8,
    pub roll_b: u8,
    /// 0 for the first creature, 1 for the second.
    pub winner: u8,
    pub narration: String,
    #[serde(rename = "move")]
    pub signature: String,
}

/// A whole showdown.
#[derive(Debug, Clone)]
pub struct Bout<'a> {
    pub a: &'a Creature,
    pub b: &'a Creature,
    pub rounds: Vec<Round>,
    /// 0 for the first creature, 1 for the second.
    pub winner: u8,
    pub finale: String,
}

impl<'a> Bout<'a> {
    fn side(&self, side: u8) -> &'a Creature {
        if side == 0 { self.a } else { self.b }
    }

    /// The victor.
    pub fn winner(&self) -> &'a Creature {
        self.side(self.winner)
    }

    /// Who won a round.
    pub fn round_winner(&self, round: &Round) -> &'a Creature {
        self.side(round.winner)
    }

    /// The rounds won by the winner and by the loser.
    pub fn score(&self) -> (usize, usize) {
        let won = self
            .rounds
            .iter()
            .filter(|r| r.winner == self.winner)
            .count();
        (won, self.rounds.len() - won)
    }
}

/// A stat to contest and how the round is told: {A} and {B} are the
/// creatures, {W} the round's winner and {L} its loser. The lines are in the
/// past tense so they read the same for one Krampus or thirteen Yule Lads.
type Contest = (&'static str, fn(&Stats) -> u8, &'static str);

const CONTESTS: [Contest; 4] = [
    (
        "Strength",
        |s| s.strength,
        "{A} and {B} locked arms on the frozen pond. {W} hurled {L} clean into a snowdrift!",
    ),
    (
        "Speed",
        |s| s.speed,
        "A chase across the rooftops! {W} vanished between the chimneys, and {L} gave up, puffing, by the weathervane.",
    ),
    (
        "Cunning",
        |s| s.cunning,
        "A battle of wits by candlelight. {W} hid the last mince pie, and {L} walked straight under the mistletoe.",
    ),
    (
        "Dread",
        |s| s.dread,
        "A staring contest over the punch bowl. {L} blinked first, and {W} let out a laugh that rattled the baubles.",
    ),
];

/// Stages a bout of three rounds. Each round tests a different stat: the
/// stat plus a six-sided die, highest wins.
pub fn fight<'a>(rng: &mut Rng, a: &'a Creature, b: &'a Creature) -> Bout<'a> {
    let mut order = [0, 1, 2, 3];
    rng.shuffle(&mut order);
    let mut bout = Bout {
        a,
        b,
        rounds: Vec::new(),
        winner: 0,
        finale: String::new(),
    };
    for (n, &contest) in order.iter().take(3).enumerate() {
        let (stat, value, line) = CONTESTS[contest];
        let (stat_a, stat_b) = (value(&a.stats), value(&b.stats));
        let mut roll = || (stat_a + rng.die(), stat_b + rng.die());
        let (mut roll_a, mut roll_b) = roll();
        // Re-roll a tie a few times; if it won't break, the better stat wins.
        for _ in 0..5 {
            if roll_a != roll_b {
                break;
            }
            (roll_a, roll_b) = roll();
        }
        let winner = u8::from(roll_b > roll_a || (roll_b == roll_a && stat_b > stat_a));
        let (w, l) = if winner == 0 { (a, b) } else { (b, a) };
        let power = if w.powers.is_empty() {
            "sheer festive spirit"
        } else {
            rng.pick(&w.powers).as_str()
        };
        bout.rounds.push(Round {
            number: n as u8 + 1,
            stat,
            roll_a,
            roll_b,
            winner,
            narration: fill(line, a, b, w, l),
            signature: format!("{}'s signature move: {power}!", w.name),
        });
    }
    let wins_b = bout.rounds.iter().filter(|r| r.winner == 1).count();
    bout.winner = u8::from(wins_b * 2 > bout.rounds.len());
    let loser = bout.side(1 - bout.winner);
    let weakness = if loser.weaknesses.is_empty() {
        "Bad luck"
    } else {
        rng.pick(&loser.weaknesses).as_str()
    };
    // Quoted, because weaknesses are written every which way: "Sticks",
    // "Twelfth Night", "Can only warn, not save".
    bout.finale = format!(
        "{} trudged off into the snow, muttering: “{}...”",
        loser.name,
        weakness.trim_end_matches('.')
    );
    bout
}

/// Fills in a narration line, writing "the" rather than "The" mid-sentence.
fn fill(line: &str, a: &Creature, b: &Creature, w: &Creature, l: &Creature) -> String {
    let mut out = String::new();
    let mut rest = line;
    while let Some(at) = rest.find('{') {
        out.push_str(&rest[..at]);
        let name = match &rest[at..at + 3] {
            "{A}" => &a.name,
            "{B}" => &b.name,
            "{W}" => &w.name,
            _ => &l.name,
        };
        let starts_sentence =
            out.trim_end().is_empty() || out.trim_end().ends_with(['.', '!', '?']);
        if starts_sentence {
            out.push_str(name);
        } else {
            out.push_str(&mid_sentence(name));
        }
        rest = &rest[at + 3..];
    }
    out + rest
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creatures::Vault;

    #[test]
    fn a_seeded_showdown_is_repeatable() {
        let v = Vault::load();
        let (a, b) = (v.get("krampus").unwrap(), v.get("yule-cat").unwrap());
        let one = fight(&mut Rng::new(25), a, b);
        let two = fight(&mut Rng::new(25), a, b);
        assert_eq!(one.rounds, two.rounds);
        assert_eq!(one.finale, two.finale);
    }

    #[test]
    fn three_rounds_three_stats_and_the_majority_wins() {
        let v = Vault::load();
        let (a, b) = (v.get("gryla").unwrap(), v.get("tomte").unwrap());
        for seed in 0..200 {
            let bout = fight(&mut Rng::new(seed), a, b);
            assert_eq!(bout.rounds.len(), 3);
            let mut stats: Vec<&str> = bout.rounds.iter().map(|r| r.stat).collect();
            stats.sort_unstable();
            stats.dedup();
            assert_eq!(stats.len(), 3, "each round tests a different stat");
            let (won, lost) = bout.score();
            assert!(won > lost && won + lost == 3);
            for r in &bout.rounds {
                let higher = u8::from(r.roll_b > r.roll_a);
                assert!(r.roll_a == r.roll_b || r.winner == higher, "{r:?}");
                assert!(r.signature.starts_with(&bout.round_winner(r).name));
                assert!(!r.narration.contains('{'), "{}", r.narration);
            }
            let loser = if bout.winner == 0 { b } else { a };
            assert!(bout.finale.starts_with(&loser.name));
        }
    }

    #[test]
    fn names_read_naturally_mid_sentence() {
        let v = Vault::load();
        let (cat, lads) = (v.get("yule-cat").unwrap(), v.get("yule-lads").unwrap());
        let line = fill(CONTESTS[0].2, cat, lads, lads, cat);
        assert_eq!(
            line,
            "The Yule Cat and the Yule Lads locked arms on the frozen pond. The Yule Lads hurled the Yule Cat clean into a snowdrift!"
        );
    }
}
