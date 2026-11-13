//! Guess the Guest: a portrait behind a frosted window. Clear a little
//! more frost for clues, but every clearing costs points.

use serde::Serialize;

use crate::creatures::Creature;
use crate::rng::Rng;

/// How much of the portrait shows at each stage, in percent, and what a
/// right answer is worth there.
pub const CLEARED: [usize; 5] = [15, 30, 50, 75, 100];
pub const POINTS: [usize; 5] = [100, 80, 60, 40, 20];

/// The last stage: the window is clear.
pub const CLEAR: usize = CLEARED.len() - 1;

/// The fewest different creatures that make a round worth playing.
const MIN_OPTIONS: usize = 3;

/// One portrait to identify.
#[derive(Debug, Clone)]
pub struct Round<'a> {
    pub creature: &'a Creature,
    pub options: Vec<String>,
    /// The index of the right option.
    pub answer: usize,
    /// What the player chose, once they have.
    pub chosen: Option<usize>,
    /// How far the frost had cleared when they chose.
    pub stage: usize,
    /// The portrait as a grid, padded to a rectangle.
    grid: Vec<Vec<char>>,
    /// The order the frost clears in, as cell numbers.
    order: Vec<usize>,
}

impl Round<'_> {
    /// Whether the player named the guest.
    pub fn correct(&self) -> bool {
        self.chosen == Some(self.answer)
    }

    /// Points scored this round.
    pub fn points(&self) -> usize {
        if self.correct() {
            POINTS[self.stage]
        } else {
            0
        }
    }

    /// The portrait at a stage: a cell is its character once the frost has
    /// cleared from it, and `None` while it is still frosted over.
    pub fn frosted(&self, stage: usize) -> Vec<Vec<Option<char>>> {
        let width = self.grid.first().map_or(0, Vec::len);
        let total = self.grid.len() * width;
        let mut clear = vec![false; total];
        for &cell in &self.order[..total * CLEARED[stage.min(CLEAR)] / 100] {
            clear[cell] = true;
        }
        self.grid
            .iter()
            .enumerate()
            .map(|(y, row)| {
                row.iter()
                    .enumerate()
                    .map(|(x, ch)| clear[y * width + x].then_some(*ch))
                    .collect()
            })
            .collect()
    }

    /// The clues, which grow as the frost clears.
    pub fn clues(&self, stage: usize) -> Vec<String> {
        let mut out = Vec::new();
        if stage >= 2 {
            let debut = &self.creature.debut;
            out.push(match (debut.year, &debut.era) {
                (Some(year), _) => format!("First appeared in {year}"),
                // Folklore has an era, not a year.
                (None, Some(era)) => format!("The legend: {era}"),
                (None, None) => format!("First appeared in {}", debut.title),
            });
        }
        if stage >= 3 {
            out.push(format!("Origin: {}", self.creature.origin));
        }
        out
    }
}

/// How the frost stands: how much has cleared and what a guess is worth,
/// or, once answered, how much had cleared when the guess was made.
pub fn status(stage: usize, answered: bool) -> String {
    let stage = stage.min(CLEAR);
    if answered {
        format!("You guessed with {}% of the frost cleared", CLEARED[stage])
    } else {
        format!(
            "Frost cleared: {}% · worth {} pts",
            CLEARED[stage], POINTS[stage]
        )
    }
}

/// The frost over a cell that hasn't cleared: a fixed, uneven pattern.
pub fn frost(x: usize, y: usize) -> char {
    ['░', '▒', '░', '*', '░'][(x * 7 + y * 3) % 5]
}

/// Picks up to `n` different creatures that have portraits. The wrong
/// answers can be any other creature. Returns nothing when there are too few
/// different creatures for a fair guess, or no portraits at all.
pub fn rounds<'a>(rng: &mut Rng, all: &'a [Creature], n: usize) -> Vec<Round<'a>> {
    let mut names: Vec<&str> = all.iter().map(|c| c.name.as_str()).collect();
    names.sort_unstable();
    names.dedup();
    if names.len() < MIN_OPTIONS {
        return Vec::new();
    }
    let mut sitters: Vec<&Creature> = all.iter().filter(|c| !c.ascii.is_empty()).collect();
    rng.shuffle(&mut sitters);
    sitters.truncate(n);
    sitters
        .into_iter()
        .map(|creature| {
            let mut others: Vec<&str> = names
                .iter()
                .copied()
                .filter(|name| *name != creature.name)
                .collect();
            rng.shuffle(&mut others);
            others.truncate(3);
            let mut options = vec![creature.name.as_str()];
            options.extend(others);
            rng.shuffle(&mut options);
            let answer = options
                .iter()
                .position(|o| *o == creature.name)
                .unwrap_or(0);

            let width = creature.ascii.lines().map(|l| l.chars().count()).max();
            let width = width.unwrap_or(0);
            let grid: Vec<Vec<char>> = creature
                .ascii
                .lines()
                .map(|l| {
                    let mut row: Vec<char> = l.chars().collect();
                    row.resize(width, ' ');
                    row
                })
                .collect();
            let mut order: Vec<usize> = (0..grid.len() * width).collect();
            rng.shuffle(&mut order);
            Round {
                creature,
                options: options.into_iter().map(str::to_string).collect(),
                answer,
                chosen: None,
                stage: 0,
                grid,
                order,
            }
        })
        .collect()
}

/// A round as `--json` prints it: the answer included, the way the quiz
/// gives its questions with theirs.
#[derive(Debug, Serialize)]
pub struct RoundJson<'a> {
    pub creature: &'a str,
    pub name: &'a str,
    pub options: &'a [String],
    pub answer: usize,
}

impl<'a> From<&'a Round<'a>> for RoundJson<'a> {
    fn from(r: &'a Round<'a>) -> Self {
        Self {
            creature: &r.creature.id,
            name: &r.creature.name,
            options: &r.options,
            answer: r.answer,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creatures::Vault;

    #[test]
    fn seeded_rounds_are_repeatable_and_well_formed() {
        let v = Vault::load();
        let a = rounds(&mut Rng::new(25), &v.creatures, 5);
        let b = rounds(&mut Rng::new(25), &v.creatures, 5);
        assert_eq!(a.len(), 5);
        for (x, y) in a.iter().zip(&b) {
            assert_eq!(x.creature.id, y.creature.id);
            assert_eq!(x.options, y.options);
            assert_eq!(x.order, y.order);
        }
        let mut ids: Vec<&str> = a.iter().map(|r| r.creature.id.as_str()).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), 5, "no creature sits twice");
        for r in &a {
            assert_eq!(r.options.len(), 4);
            assert_eq!(r.options[r.answer], r.creature.name);
            let mut unique = r.options.clone();
            unique.sort();
            unique.dedup();
            assert_eq!(unique.len(), 4);
        }
    }

    #[test]
    fn there_are_only_as_many_rounds_as_portraits() {
        let v = Vault::load();
        assert_eq!(rounds(&mut Rng::new(1), &v.creatures, 99).len(), 14);
    }

    #[test]
    fn too_few_creatures_is_no_game() {
        let v = Vault::load().only_packs(&["dickens".into()]).unwrap();
        assert!(rounds(&mut Rng::new(1), &v.creatures, 5).is_empty());
    }

    #[test]
    fn the_frost_clears_in_stages_and_never_comes_back() {
        let v = Vault::load();
        let r = &rounds(&mut Rng::new(25), &v.creatures, 1)[0];
        let shown = |stage| -> Vec<bool> {
            r.frosted(stage)
                .concat()
                .iter()
                .map(Option::is_some)
                .collect()
        };
        let total = shown(0).len();
        for (stage, percent) in CLEARED.iter().enumerate() {
            let clear = shown(stage).iter().filter(|c| **c).count();
            assert_eq!(clear, total * percent / 100);
            if stage > 0 {
                let before = shown(stage - 1);
                let now = shown(stage);
                assert!(before.iter().zip(&now).all(|(b, n)| !b || *n));
            }
        }
        let clear: Vec<String> = r
            .frosted(CLEAR)
            .iter()
            .map(|row| row.iter().map(|c| c.unwrap()).collect::<String>())
            .map(|row| row.trim_end().to_string())
            .collect();
        assert_eq!(clear.join("\n"), r.creature.ascii);
    }

    #[test]
    fn points_fall_as_the_frost_clears() {
        let v = Vault::load();
        let mut r = rounds(&mut Rng::new(25), &v.creatures, 1).remove(0);
        assert_eq!(r.points(), 0, "no guess, no points");
        r.chosen = Some(r.answer);
        r.stage = 0;
        assert_eq!(r.points(), 100);
        r.stage = CLEAR;
        assert_eq!(r.points(), 20);
        r.chosen = Some((r.answer + 1) % 4);
        assert_eq!(r.points(), 0);
    }

    #[test]
    fn clues_arrive_at_the_third_and_fourth_stages() {
        let v = Vault::load();
        let all = rounds(&mut Rng::new(25), &v.creatures, 14);
        let marley = all.iter().find(|r| r.creature.id == "marley").unwrap();
        assert!(marley.clues(1).is_empty());
        assert_eq!(marley.clues(2), ["First appeared in 1843"]);
        assert_eq!(marley.clues(3).len(), 2);
        let gryla = all.iter().find(|r| r.creature.id == "gryla").unwrap();
        assert!(gryla.clues(2)[0].starts_with("The legend: "));
    }

    #[test]
    fn status_reads_naturally() {
        assert_eq!(status(0, false), "Frost cleared: 15% · worth 100 pts");
        assert_eq!(status(2, true), "You guessed with 50% of the frost cleared");
    }
}
