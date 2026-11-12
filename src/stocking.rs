//! The Stocking turns play into progress: a creature goes in the stocking
//! once you've answered enough questions about it, and badges reward
//! milestones.

use std::collections::BTreeMap;

use jiff::civil::DateTime;

use crate::calendar;
use crate::creatures::Creature;
use crate::store::Progress;

/// How many correct answers about a creature put it in the stocking.
pub const CAPTURE_AT: u32 = 3;

/// An achievement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Badge {
    pub id: &'static str,
    pub emoji: &'static str,
    pub name: &'static str,
    pub description: &'static str,
}

const fn badge(
    id: &'static str,
    emoji: &'static str,
    name: &'static str,
    description: &'static str,
) -> Badge {
    Badge {
        id,
        emoji,
        name,
        description,
    }
}

/// Every badge, in the order they are shown.
pub const BADGES: [Badge; 17] = [
    badge(
        "first-cracker",
        "🎉",
        "First Cracker",
        "Finish your first quiz",
    ),
    badge(
        "top-of-the-tree",
        "🏆",
        "Top of the Tree",
        "Score 100% on a quiz of 5 or more questions",
    ),
    badge(
        "stocking-filler",
        "🧦",
        "Stocking Filler",
        "Put your first creature in the stocking",
    ),
    badge(
        "frost-reader",
        "🔍",
        "Frost Reader",
        "Name a guest in Guess the Guest through the thickest frost",
    ),
    badge(
        "snowball-referee",
        "🥊",
        "Snowball Referee",
        "Stage your first Snowball Showdown",
    ),
    badge(
        "fireside-scholar",
        "📚",
        "Fireside Scholar",
        "Visit every creature's page in the explorer",
    ),
    badge(
        "up-past-bedtime",
        "🦉",
        "Up Past Bedtime",
        "Play between midnight and 4am",
    ),
    badge(
        "cold-moon",
        "🌕",
        "Cold Moon",
        "Play a game on a full-moon night",
    ),
    badge(
        "krampusnacht",
        "😈",
        "Krampusnacht Survivor",
        "Play a game on 5 December",
    ),
    badge(
        "christmas-spirit",
        "🎄",
        "Christmas Spirit",
        "Play a game on Christmas Day",
    ),
    badge(
        "twelfth-nighter",
        "🌟",
        "Twelfth Nighter",
        "Play a game on Twelfth Night, 5 January",
    ),
    badge(
        "matinee-regular",
        "🎬",
        "Matinee Regular",
        "Capture every creature with a classic film",
    ),
    badge(
        "saints-shadow",
        "🎁",
        "Saint's Shadow",
        "Capture every companion of St Nicholas",
    ),
    badge(
        "midwinter-wanderer",
        "🌍",
        "Midwinter Wanderer",
        "Capture 5 from Midwinter Europe, or all it has",
    ),
    badge(
        "friend-of-the-yule-cat",
        "🐈",
        "Friend of the Yule Cat",
        "Capture every creature of the Nordic Yule",
    ),
    badge(
        "dickensian",
        "📖",
        "Dickensian",
        "Capture every spirit of A Christmas Carol",
    ),
    badge(
        "keeper",
        "👑",
        "Keeper of the Stocking",
        "Capture every creature",
    ),
];

/// The badges for capturing every creature in a pack.
const PACK_BADGES: [(&str, &str); 3] = [
    ("companions", "saints-shadow"),
    ("nordic", "friend-of-the-yule-cat"),
    ("dickens", "dickensian"),
];

/// What kind of session an outcome describes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Quiz,
    Guess,
    Mash,
    Explore,
}

/// One session of play.
#[derive(Debug, Clone)]
pub struct Outcome {
    pub kind: Kind,
    /// Correct answers per creature id.
    pub correct: BTreeMap<String, u32>,
    /// Percent for the quiz, points for guessing.
    pub score: u32,
    /// Questions or rounds in the game.
    pub total: u32,
    /// Whether it was played to the end.
    pub completed: bool,
    /// Creature pages visited.
    pub seen: Vec<String>,
    /// Whether a guest was named through the thickest frost.
    pub frost_reader: bool,
    /// When it was played, in local time.
    pub now: DateTime,
}

impl Outcome {
    pub fn new(kind: Kind, now: DateTime) -> Self {
        Self {
            kind,
            correct: BTreeMap::new(),
            score: 0,
            total: 0,
            completed: false,
            seen: Vec::new(),
            frost_reader: false,
            now,
        }
    }
}

/// What a session newly earned.
#[derive(Debug, Default)]
pub struct Unlocks<'a> {
    pub captured: Vec<&'a Creature>,
    pub badges: Vec<Badge>,
}

impl Unlocks<'_> {
    /// Whether nothing new was earned.
    pub fn is_empty(&self) -> bool {
        self.captured.is_empty() && self.badges.is_empty()
    }
}

/// Records an outcome in the progress and reports anything newly unlocked.
/// `all` is every creature, whatever `--pack` says: the badges count them all.
pub fn apply<'a>(p: &mut Progress, all: &'a [Creature], o: &Outcome) -> Unlocks<'a> {
    let when = o.now.strftime("%Y-%m-%dT%H:%M:%S").to_string();
    let mut unlocks = Unlocks::default();
    let mut award = |p: &mut Progress, earned: bool, id: &str| {
        if earned
            && !p.badges.contains_key(id)
            && let Some(b) = BADGES.iter().find(|b| b.id == id)
        {
            p.badges.insert(id.to_string(), when.clone());
            unlocks.badges.push(*b);
        }
    };

    // The game itself.
    let finished = o.completed && o.total > 0;
    match o.kind {
        Kind::Quiz if finished => {
            p.quizzes_played += 1;
            p.best_quiz_score = p.best_quiz_score.max(o.score);
            award(p, true, "first-cracker");
            award(p, o.score == 100 && o.total >= 5, "top-of-the-tree");
        }
        Kind::Guess => {
            if finished {
                p.guesses_played += 1;
                p.best_guess_score = p.best_guess_score.max(o.score);
            }
            award(p, o.frost_reader, "frost-reader");
        }
        Kind::Mash => {
            p.mashes_played += 1;
            award(p, true, "snowball-referee");
        }
        _ => {}
    }

    // Playing at a memorable time. Reading in the explorer isn't playing.
    if o.kind != Kind::Explore {
        let today = o.now.date();
        award(p, o.now.hour() < 4, "up-past-bedtime");
        award(p, calendar::is_full_moon(today), "cold-moon");
        award(p, calendar::is_krampusnacht(today), "krampusnacht");
        award(p, calendar::is_christmas(today), "christmas-spirit");
        award(p, calendar::is_twelfth_night(today), "twelfth-nighter");
    }

    // What was read and what was known.
    for id in &o.seen {
        p.seen.insert(id.clone(), true);
    }
    for (id, n) in &o.correct {
        *p.knowledge.entry(id.clone()).or_default() += n;
    }
    for c in all {
        let known = p.knowledge.get(&c.id).copied().unwrap_or(0);
        if known >= CAPTURE_AT && !p.captured.contains_key(&c.id) {
            p.captured.insert(c.id.clone(), when.clone());
            unlocks.captured.push(c);
        }
    }

    // What the stocking holds now.
    let held = |c: &&Creature| p.captured.contains_key(&c.id);
    let every = |group: &dyn Fn(&Creature) -> bool| {
        let members: Vec<&Creature> = all.iter().filter(|c| group(c)).collect();
        !members.is_empty() && members.iter().all(held)
    };
    let stocking_filler = !p.captured.is_empty();
    let scholar = !all.is_empty() && all.iter().all(|c| p.seen.contains_key(&c.id));
    let matinee = every(&|c| c.film.is_some());
    let midwinter: Vec<&Creature> = all.iter().filter(|c| c.pack == "midwinter").collect();
    let wanderer = !midwinter.is_empty()
        && midwinter.iter().filter(|c| held(c)).count() >= midwinter.len().min(5);
    let packs: Vec<bool> = PACK_BADGES
        .iter()
        .map(|(pack, _)| every(&|c| c.pack == *pack))
        .collect();
    let keeper = every(&|_| true);

    award(p, stocking_filler, "stocking-filler");
    award(p, scholar, "fireside-scholar");
    award(p, matinee, "matinee-regular");
    award(p, wanderer, "midwinter-wanderer");
    for ((_, id), earned) in PACK_BADGES.iter().zip(packs) {
        award(p, earned, id);
    }
    award(p, keeper, "keeper");
    unlocks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creatures::Vault;
    use jiff::civil::datetime;
    use unicode_width::UnicodeWidthStr;

    /// An ordinary Thursday evening: no feast, no full moon.
    fn evening() -> DateTime {
        datetime(2026, 11, 12, 20, 0, 0, 0)
    }

    fn correct(pairs: &[(&str, u32)]) -> BTreeMap<String, u32> {
        pairs.iter().map(|(id, n)| (id.to_string(), *n)).collect()
    }

    fn names(u: &Unlocks) -> Vec<&'static str> {
        u.badges.iter().map(|b| b.id).collect()
    }

    #[test]
    fn badges_are_unique_and_two_columns_wide() {
        let mut ids: Vec<&str> = BADGES.iter().map(|b| b.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), BADGES.len());
        for b in BADGES {
            assert!(
                !b.emoji.contains('\u{FE0F}'),
                "{}: variation selector",
                b.id
            );
            assert_eq!(b.emoji.width(), 2, "{}: emoji width", b.id);
        }
        for (_, id) in PACK_BADGES {
            assert!(BADGES.iter().any(|b| b.id == id), "{id}");
        }
    }

    #[test]
    fn three_right_answers_put_a_creature_in_the_stocking() {
        let v = Vault::load();
        let mut p = Progress::default();
        let mut o = Outcome::new(Kind::Quiz, evening());
        o.correct = correct(&[("krampus", 2)]);
        let u = apply(&mut p, &v.creatures, &o);
        assert!(u.captured.is_empty());
        o.correct = correct(&[("krampus", 1), ("tomte", 1)]);
        let u = apply(&mut p, &v.creatures, &o);
        assert_eq!(u.captured.len(), 1);
        assert_eq!(u.captured[0].id, "krampus");
        assert_eq!(names(&u), ["stocking-filler"]);
        assert_eq!(p.captured["krampus"], "2026-11-12T20:00:00");
        // Capturing it again changes nothing.
        let u = apply(&mut p, &v.creatures, &o);
        assert!(u.is_empty());
    }

    #[test]
    fn finishing_a_quiz_counts_and_a_perfect_one_tops_the_tree() {
        let v = Vault::load();
        let mut p = Progress::default();
        let mut o = Outcome::new(Kind::Quiz, evening());
        (o.score, o.total, o.completed) = (60, 10, true);
        assert_eq!(names(&apply(&mut p, &v.creatures, &o)), ["first-cracker"]);
        (o.score, o.total) = (100, 4);
        assert!(
            apply(&mut p, &v.creatures, &o).is_empty(),
            "too short to count"
        );
        (o.score, o.total) = (100, 5);
        assert_eq!(names(&apply(&mut p, &v.creatures, &o)), ["top-of-the-tree"]);
        assert_eq!((p.quizzes_played, p.best_quiz_score), (3, 100));
        // A quiz abandoned halfway isn't a quiz played.
        o.completed = false;
        apply(&mut p, &v.creatures, &o);
        assert_eq!(p.quizzes_played, 3);
    }

    #[test]
    fn the_games_have_their_own_badges_and_records() {
        let v = Vault::load();
        let mut p = Progress::default();
        let u = apply(&mut p, &v.creatures, &Outcome::new(Kind::Mash, evening()));
        assert_eq!(names(&u), ["snowball-referee"]);
        assert_eq!(p.mashes_played, 1);
        let mut o = Outcome::new(Kind::Guess, evening());
        (o.score, o.total, o.completed, o.frost_reader) = (260, 5, true, true);
        assert_eq!(names(&apply(&mut p, &v.creatures, &o)), ["frost-reader"]);
        assert_eq!((p.guesses_played, p.best_guess_score), (1, 260));
    }

    #[test]
    fn memorable_nights_are_rewarded_for_playing_not_reading() {
        let v = Vault::load();
        let night = |y, m, d, h| datetime(y, m, d, h, 0, 0, 0);
        let play = |now| {
            let mut p = Progress::default();
            names(&apply(&mut p, &v.creatures, &Outcome::new(Kind::Mash, now)))
        };
        assert!(play(night(2026, 12, 5, 20)).contains(&"krampusnacht"));
        assert!(play(night(2026, 12, 25, 12)).contains(&"christmas-spirit"));
        assert!(play(night(2027, 1, 5, 20)).contains(&"twelfth-nighter"));
        assert!(play(night(2027, 12, 13, 20)).contains(&"cold-moon"));
        assert!(play(night(2026, 11, 12, 2)).contains(&"up-past-bedtime"));
        assert_eq!(play(evening()), ["snowball-referee"]);
        let mut p = Progress::default();
        let read = Outcome::new(Kind::Explore, night(2026, 12, 25, 2));
        assert!(apply(&mut p, &v.creatures, &read).is_empty());
    }

    #[test]
    fn reading_every_page_makes_a_fireside_scholar() {
        let v = Vault::load();
        let mut p = Progress::default();
        let mut o = Outcome::new(Kind::Explore, evening());
        o.seen = v.creatures.iter().skip(1).map(|c| c.id.clone()).collect();
        assert!(apply(&mut p, &v.creatures, &o).is_empty());
        o.seen = vec![v.creatures[0].id.clone()];
        assert_eq!(
            names(&apply(&mut p, &v.creatures, &o)),
            ["fireside-scholar"]
        );
    }

    #[test]
    fn filling_packs_and_then_the_whole_stocking() {
        let v = Vault::load();
        let mut p = Progress::default();
        let capture = |p: &mut Progress, pack: &str| {
            let mut o = Outcome::new(Kind::Quiz, evening());
            o.correct = v
                .creatures
                .iter()
                .filter(|c| c.pack == pack)
                .map(|c| (c.id.clone(), CAPTURE_AT))
                .collect();
            names(&apply(p, &v.creatures, &o))
        };
        assert_eq!(
            capture(&mut p, "nordic"),
            ["stocking-filler", "friend-of-the-yule-cat"]
        );
        assert_eq!(capture(&mut p, "companions"), ["saints-shadow"]);
        // Midwinter Europe has four creatures today, so all four earn it.
        assert_eq!(capture(&mut p, "midwinter"), ["midwinter-wanderer"]);
        // Marley is the only creature with a film, and Dickens completes the set.
        assert_eq!(
            capture(&mut p, "dickens"),
            ["matinee-regular", "dickensian", "keeper"]
        );
        assert_eq!(p.captured.len(), v.creatures.len());
    }

    #[test]
    fn an_empty_roster_earns_nothing() {
        let mut p = Progress::default();
        let u = apply(&mut p, &[], &Outcome::new(Kind::Explore, evening()));
        assert!(u.is_empty());
    }
}
