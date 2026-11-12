//! Sleeps until Christmas, the advent calendar, the Twelve Days and the
//! special nights of the season. Everything works offline from a date.

use jiff::civil::{Date, date};

use crate::creatures::{AdventNight, Creature, Vault};
use crate::error::{Code, Result, TinselError};
use crate::rng::Rng;

/// Today's date in the local time zone.
pub fn today() -> Date {
    jiff::Zoned::now().date()
}

/// The date and time now, in the local time zone.
pub fn now() -> jiff::civil::DateTime {
    jiff::Zoned::now().datetime()
}

/// Parses `--date YYYY-MM-DD`, or returns today when it's absent.
pub fn parse_date(arg: Option<&str>) -> Result<Date> {
    match arg {
        None => Ok(today()),
        Some(s) => s.trim().parse::<Date>().map_err(|_| {
            TinselError::new(Code::BadDate, format!("{s:?} isn't a date I recognise"))
                .with_hint("use YYYY-MM-DD, e.g. --date 2026-12-24")
        }),
    }
}

/// The seed for the Fact of the Night: the same all day, for everyone.
pub fn daily_seed(d: Date) -> u64 {
    (i64::from(d.year()) * 10_000 + i64::from(d.month()) * 100 + i64::from(d.day())) as u64
}

/// Whether it's Christmas Day.
pub fn is_christmas(d: Date) -> bool {
    d.month() == 12 && d.day() == 25
}

/// Sleeps until Christmas: 1 on Christmas Eve, 0 on Christmas Day, and from
/// Boxing Day the count starts again for next year.
pub fn sleeps_until_christmas(d: Date) -> i64 {
    let this_year = date(d.year(), 12, 25);
    let target = if d <= this_year {
        this_year
    } else {
        date(d.year() + 1, 12, 25)
    };
    d.until(target)
        .map(|span| i64::from(span.get_days()))
        .unwrap_or_default()
}

/// The advent calendar door for this date (1 to 24 December), if any.
pub fn advent_night(d: Date, vault: &Vault) -> Option<&AdventNight> {
    if d.month() != 12 || d.day() > 24 {
        return None;
    }
    vault
        .advent
        .iter()
        .find(|n| i16::from(n.night) == i16::from(d.day()))
}

/// Tonight's advent guest, with a fact picked for the night, if the date is in
/// advent and the creature is in the chosen packs.
pub fn advent_guest(d: Date, vault: &Vault) -> Option<(&AdventNight, &Creature, String)> {
    let night = advent_night(d, vault)?;
    let creature = vault.get(&night.id)?;
    let mut rng = Rng::new(daily_seed(d));
    let fact = rng.pick(&creature.facts).clone();
    Some((night, creature, fact))
}

/// Which of the Twelve Days of Christmas this is (25 December is day 1, and
/// 5 January, Twelfth Night, is day 12), if any.
pub fn twelve_days(d: Date) -> Option<u8> {
    match (d.month(), d.day()) {
        (12, day @ 25..=31) => Some((day - 24) as u8),
        (1, day @ 1..=5) => Some((day + 7) as u8),
        _ => None,
    }
}

/// How many Yule Lads are in town: one more each night from 12 December, then
/// one leaves each day from Christmas Day until the last goes on 6 January.
pub fn yule_lads_in_town(d: Date) -> u8 {
    match (d.month(), d.day()) {
        (12, day @ 12..=24) => (day - 11) as u8,
        (12, day @ 25..=31) => (13 - (day - 24)) as u8,
        (1, day @ 1..=6) => (13 - 7 - day).max(0) as u8,
        _ => 0,
    }
}

/// The average time between new moons, in days.
const SYNODIC_MONTH: f64 = 29.530_588_853;

/// The moon's age on a date: days since the last new moon, counted from the
/// new moon of 6 January 2000 (18:14 UTC) with the mean lunar month. It is
/// accurate to within about a day, which is plenty for a Christmas special.
pub fn moon_age(d: Date) -> f64 {
    let since = date(2000, 1, 6)
        .until(d)
        .map(|span| f64::from(span.get_days()))
        .unwrap_or_default();
    // The date stands for its evening (9pm), and the reference moon was at 18:14.
    (since + (21.0 - 18.0 - 14.0 / 60.0) / 24.0).rem_euclid(SYNODIC_MONTH)
}

/// The moon on a date.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Moon {
    /// Days since the last new moon.
    pub age: f64,
    /// From 0 (new) to 1 (full).
    pub illumination: f64,
    pub name: &'static str,
    pub emoji: &'static str,
}

/// The eight phases, each with the upper bound of its share of the month in
/// eighths: the new moon straddles the start and the end.
const PHASES: [(f64, &str, &str); 9] = [
    (0.5, "New Moon", "🌑"),
    (1.5, "Waxing Crescent", "🌒"),
    (2.5, "First Quarter", "🌓"),
    (3.5, "Waxing Gibbous", "🌔"),
    (4.5, "Full Moon", "🌕"),
    (5.5, "Waning Gibbous", "🌖"),
    (6.5, "Last Quarter", "🌗"),
    (7.5, "Waning Crescent", "🌘"),
    (8.0, "New Moon", "🌑"),
];

/// The moon's phase on a date.
pub fn moon(d: Date) -> Moon {
    let age = moon_age(d);
    let eighths = age / SYNODIC_MONTH * 8.0;
    let (_, name, emoji) = PHASES
        .iter()
        .find(|(until, _, _)| eighths < *until)
        .copied()
        .unwrap_or(PHASES[0]);
    Moon {
        age,
        illumination: (1.0 - (std::f64::consts::TAU * age / SYNODIC_MONTH).cos()) / 2.0,
        name,
        emoji,
    }
}

/// A classic film that opened on this day of the year.
#[derive(Debug, Clone, Copy)]
pub struct Anniversary<'a> {
    pub creature: &'a Creature,
    pub years: i16,
}

/// The films that opened on this day and month in an earlier year.
pub fn anniversaries(d: Date, creatures: &[Creature]) -> Vec<Anniversary<'_>> {
    creatures
        .iter()
        .filter_map(|c| {
            let released = c.film.as_ref()?.released()?;
            let same_day = (released.month(), released.day()) == (d.month(), d.day());
            (same_day && d.year() > released.year()).then(|| Anniversary {
                creature: c,
                years: d.year() - released.year(),
            })
        })
        .collect()
}

/// The dated feasts and nights of the season, as (month, day, name).
const FEASTS: [(i8, i8, &str); 11] = [
    (12, 5, "Krampusnacht"),
    (12, 6, "St Nicholas Day"),
    (12, 8, "the Tió de Nadal's homecoming"),
    (12, 13, "St Lucy's Night"),
    (12, 19, "the anniversary of A Christmas Carol"),
    (12, 21, "the winter solstice"),
    (12, 24, "Christmas Eve"),
    (12, 25, "Christmas Day"),
    (12, 26, "Boxing Day"),
    (1, 5, "Twelfth Night"),
    (1, 6, "Epiphany"),
];

/// The next night worth staying up for: a feast of the season, or the
/// anniversary of a creature's classic film.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecialNight {
    /// "Krampusnacht", or for a film, "Scrooge turns 75".
    pub name: String,
    pub date: Date,
    /// Sleeps from the date asked about; never 0.
    pub sleeps: i64,
    /// The creature whose film it is, for an anniversary.
    pub creature: Option<String>,
}

/// The first time a month and day come round after `d`.
fn next_occurrence(d: Date, month: i8, day: i8) -> Option<Date> {
    [d.year(), d.year() + 1]
        .into_iter()
        .filter_map(|year| Date::new(year, month, day).ok())
        .find(|when| *when > d)
}

/// The next special night after `d`: the nearer of the next feast and the
/// next film anniversary. A feast wins a tie.
pub fn next_special_night(d: Date, creatures: &[Creature]) -> Option<SpecialNight> {
    let sleeps = |when: Date| d.until(when).map_or(0, |span| i64::from(span.get_days()));
    let feasts = FEASTS.iter().filter_map(|(month, day, name)| {
        let date = next_occurrence(d, *month, *day)?;
        Some(SpecialNight {
            name: name.to_string(),
            date,
            sleeps: sleeps(date),
            creature: None,
        })
    });
    let films = creatures.iter().filter_map(|c| {
        let film = c.film.as_ref()?;
        let released = film.released()?;
        let date = next_occurrence(d, released.month(), released.day())?;
        Some(SpecialNight {
            name: format!("{} turns {}", film.title, date.year() - released.year()),
            date,
            sleeps: sleeps(date),
            creature: Some(c.id.clone()),
        })
    });
    feasts.chain(films).min_by_key(|night| night.sleeps)
}

/// Whether a date falls within a day of the full moon.
pub fn is_full_moon(d: Date) -> bool {
    (moon_age(d) - SYNODIC_MONTH / 2.0).abs() < 1.0
}

/// Whether it's Krampusnacht, the eve of St Nicholas Day.
pub fn is_krampusnacht(d: Date) -> bool {
    d.month() == 12 && d.day() == 5
}

/// Whether it's Twelfth Night, the last of the Twelve Days.
pub fn is_twelfth_night(d: Date) -> bool {
    twelve_days(d) == Some(12)
}

/// A note about a special night of the season, with an emoji for the terminal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    pub emoji: &'static str,
    pub text: String,
}

fn note(emoji: &'static str, text: impl Into<String>) -> Note {
    Note {
        emoji,
        text: text.into(),
    }
}

/// Notes for the special nights of the season, for the countdown and the Fact
/// of the Night.
pub fn notes(d: Date, creatures: &[Creature]) -> Vec<Note> {
    let mut out = Vec::new();
    match (d.month(), d.day()) {
        (12, 5) => out.push(note("😈", "It's Krampusnacht. Hide the birch switches.")),
        (12, 6) => out.push(note(
            "🎁",
            "It's St Nicholas Day. Père Fouettard is close behind.",
        )),
        (12, 8) => out.push(note(
            "🪵",
            "The Tió de Nadal comes home today. Don't forget to feed it.",
        )),
        (12, 19) => out.push(note(
            "📖",
            format!(
                "A Christmas Carol was published on this day in 1843, {} years ago.",
                d.year() - 1843
            ),
        )),
        (12, 24) => out.push(note(
            "🎄",
            "It's Christmas Eve. Porridge out for the tomte, and new clothes on.",
        )),
        (12, 26) => out.push(note(
            "🐴",
            "It's Boxing Day. Listen out for the Mari Lwyd at the door.",
        )),
        (1, 5) => out.push(note(
            "🧶",
            "It's Twelfth Night, Perchta's night. Is the spinning done?",
        )),
        (1, 6) => out.push(note(
            "💧",
            "It's Epiphany. The waters are blessed and the kallikantzaroi flee.",
        )),
        (12, 21) => out.push(note(
            "🌙",
            "The winter solstice is here, or near enough: the longest night of the year.",
        )),
        (12, 25) => out.push(note(
            "🎁",
            "It's Christmas Day! The busiest morning of the year for stockings.",
        )),
        _ => {}
    }
    for a in anniversaries(d, creatures) {
        if let Some(film) = &a.creature.film {
            out.push(note(
                "🎬",
                format!(
                    "On this day in {}, {} was released: {} years ago tonight.",
                    film.year, film.title, a.years
                ),
            ));
        }
    }
    if is_full_moon(d) {
        out.push(note(
            "🌕",
            if d.month() == 12 {
                "The Cold Moon is full tonight. Bright enough to see who's on the roof."
            } else {
                "Full moon tonight. Bright enough to see who's on the roof."
            },
        ));
    }
    let lads = yule_lads_in_town(d);
    if lads > 0 {
        let lad = if lads == 1 {
            "Yule Lad is"
        } else {
            "Yule Lads are"
        };
        out.push(note("🥄", format!("{lads} {lad} in town.")));
    }
    if let Some(n) = twelve_days(d) {
        out.push(note(
            "✨",
            format!("Day {n} of the Twelve Days of Christmas."),
        ));
    }
    let sleeps = sleeps_until_christmas(d);
    if (1..=60).contains(&sleeps) {
        let noun = if sleeps == 1 { "sleep" } else { "sleeps" };
        out.push(note("🎄", format!("{sleeps} {noun} until Christmas.")));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sleeps_count_down_to_christmas_and_start_again() {
        assert_eq!(sleeps_until_christmas(date(2026, 12, 24)), 1);
        assert_eq!(sleeps_until_christmas(date(2026, 12, 25)), 0);
        assert_eq!(sleeps_until_christmas(date(2026, 12, 1)), 24);
        assert_eq!(sleeps_until_christmas(date(2026, 11, 12)), 43);
        assert_eq!(sleeps_until_christmas(date(2026, 12, 26)), 364);
        // 2028 is a leap year.
        assert_eq!(sleeps_until_christmas(date(2027, 12, 26)), 365);
    }

    #[test]
    fn advent_runs_from_the_1st_to_the_24th() {
        let v = Vault::load();
        assert!(advent_night(date(2026, 11, 30), &v).is_none());
        assert_eq!(advent_night(date(2026, 12, 1), &v).unwrap().night, 1);
        assert_eq!(advent_night(date(2026, 12, 5), &v).unwrap().id, "krampus");
        assert_eq!(advent_night(date(2026, 12, 24), &v).unwrap().night, 24);
        assert!(advent_night(date(2026, 12, 25), &v).is_none());
    }

    #[test]
    fn anniversaries_need_an_earlier_year() {
        let v = Vault::load();
        assert_eq!(anniversaries(date(2026, 10, 31), &v.creatures)[0].years, 75);
        assert!(anniversaries(date(1951, 10, 31), &v.creatures).is_empty());
        assert!(anniversaries(date(2026, 10, 30), &v.creatures).is_empty());
    }

    #[test]
    fn advent_guest_is_the_same_all_day() {
        let v = Vault::load();
        let a = advent_guest(date(2026, 12, 12), &v).unwrap();
        let b = advent_guest(date(2026, 12, 12), &v).unwrap();
        assert_eq!(a.1.id, "yule-lads");
        assert_eq!(a.2, b.2);
    }

    #[test]
    fn twelve_days_run_from_christmas_to_twelfth_night() {
        assert_eq!(twelve_days(date(2026, 12, 24)), None);
        assert_eq!(twelve_days(date(2026, 12, 25)), Some(1));
        assert_eq!(twelve_days(date(2026, 12, 31)), Some(7));
        assert_eq!(twelve_days(date(2027, 1, 5)), Some(12));
        assert_eq!(twelve_days(date(2027, 1, 6)), None);
    }

    #[test]
    fn yule_lads_arrive_and_leave_one_at_a_time() {
        assert_eq!(yule_lads_in_town(date(2026, 12, 11)), 0);
        assert_eq!(yule_lads_in_town(date(2026, 12, 12)), 1);
        assert_eq!(yule_lads_in_town(date(2026, 12, 24)), 13);
        assert_eq!(yule_lads_in_town(date(2026, 12, 25)), 12);
        assert_eq!(yule_lads_in_town(date(2027, 1, 5)), 1);
        assert_eq!(yule_lads_in_town(date(2027, 1, 6)), 0);
    }

    #[test]
    fn the_moon_is_full_when_the_almanac_says_so() {
        // Full moons: 24 December 2026 (01:28 UTC) and 13 December 2027.
        assert!(is_full_moon(date(2026, 12, 24)) || is_full_moon(date(2026, 12, 23)));
        assert!(is_full_moon(date(2027, 12, 13)));
        // New moons: 9 December 2026 and 6 January 2000.
        assert!(!is_full_moon(date(2026, 12, 9)));
        assert!(moon_age(date(2000, 1, 6)) < 1.0);
        let age = moon_age(date(2026, 12, 9));
        assert!(!(1.5..=28.0).contains(&age), "new moon, but age {age}");
        // Dates before the reference moon work too.
        assert!((0.0..SYNODIC_MONTH).contains(&moon_age(date(1843, 12, 19))));
    }

    #[test]
    fn the_big_nights_are_recognised() {
        assert!(is_krampusnacht(date(2026, 12, 5)));
        assert!(!is_krampusnacht(date(2026, 12, 6)));
        assert!(is_twelfth_night(date(2027, 1, 5)));
        assert!(!is_twelfth_night(date(2027, 1, 6)));
    }

    #[test]
    fn bad_dates_have_a_code_and_a_hint() {
        let e = parse_date(Some("christmas")).unwrap_err();
        assert_eq!(e.code, Code::BadDate);
        assert!(e.hint.is_some());
        assert_eq!(parse_date(Some("2026-12-24")).unwrap(), date(2026, 12, 24));
    }

    #[test]
    fn daily_seed_is_the_date_as_a_number() {
        assert_eq!(daily_seed(date(2026, 12, 24)), 20261224);
    }
}

#[cfg(test)]
mod note_tests {
    use super::*;

    #[test]
    fn notes_mark_the_special_nights() {
        let v = Vault::load();
        let texts = |d| {
            notes(d, &v.creatures)
                .into_iter()
                .map(|n| n.text)
                .collect::<Vec<_>>()
        };
        assert!(texts(date(2026, 12, 5))[0].contains("Krampusnacht"));
        assert!(texts(date(2026, 12, 19))[0].contains("183 years ago"));
        assert_eq!(texts(date(2026, 11, 12)), ["43 sleeps until Christmas."]);
        assert_eq!(
            texts(date(2026, 12, 24)).last().unwrap(),
            "1 sleep until Christmas."
        );
        assert!(texts(date(2026, 7, 1)).is_empty(), "midsummer is quiet");
        assert!(texts(date(2026, 12, 21))[0].contains("winter solstice"));
        // Scrooge opened in Britain on 31 October 1951.
        assert_eq!(
            texts(date(2026, 10, 31))[0],
            "On this day in 1951, Scrooge was released: 75 years ago tonight."
        );
        assert!(
            texts(date(2027, 12, 13))
                .iter()
                .any(|t| t.contains("Cold Moon"))
        );
    }

    #[test]
    fn every_note_emoji_is_two_columns_wide() {
        use unicode_width::UnicodeWidthStr;
        let v = Vault::load();
        let mut d = date(2026, 10, 1);
        while d < date(2027, 2, 1) {
            for n in notes(d, &v.creatures) {
                assert!(!n.emoji.contains('\u{FE0F}'), "{d}: {}", n.emoji);
                assert_eq!(n.emoji.width(), 2, "{d}: {}", n.emoji);
            }
            d = d.tomorrow().unwrap();
        }
        for (_, _, emoji) in PHASES {
            assert_eq!(emoji.width(), 2, "{emoji}");
        }
    }

    #[test]
    fn the_moon_has_a_name_and_a_face() {
        let full = moon(date(2027, 12, 13));
        assert_eq!((full.name, full.emoji), ("Full Moon", "🌕"));
        assert!(full.illumination > 0.97, "{}", full.illumination);
        let new = moon(date(2026, 12, 9));
        assert_eq!(new.name, "New Moon");
        assert!(new.illumination < 0.03, "{}", new.illumination);
        let waxing = moon(date(2026, 12, 16));
        assert!(waxing.name.starts_with("First Quarter") || waxing.name.starts_with("Waxing"));
        // Every day of a year has a phase, and the light is always a fraction.
        let mut d = date(2026, 1, 1);
        while d < date(2027, 1, 1) {
            let m = moon(d);
            assert!((0.0..=1.0).contains(&m.illumination) && !m.name.is_empty());
            d = d.tomorrow().unwrap();
        }
    }

    #[test]
    fn the_next_special_night_is_the_nearer_of_a_feast_and_a_film() {
        let v = Vault::load();
        let next = |d| next_special_night(d, &v.creatures).unwrap();
        let n = next(date(2026, 11, 12));
        assert_eq!((n.name.as_str(), n.sleeps), ("Krampusnacht", 23));
        assert_eq!(n.date, date(2026, 12, 5));
        // On the night itself, it's the one after.
        assert_eq!(next(date(2026, 12, 5)).name, "St Nicholas Day");
        assert_eq!(next(date(2026, 12, 31)).name, "Twelfth Night");
        assert_eq!(next(date(2026, 12, 31)).date, date(2027, 1, 5));
        // After Epiphany the next thing in the diary is Scrooge's birthday.
        let n = next(date(2027, 1, 6));
        assert_eq!(n.name, "Scrooge turns 76");
        assert_eq!(n.creature.as_deref(), Some("marley"));
        assert_eq!(n.date, date(2027, 10, 31));
        // With no films, the feasts still come round.
        assert_eq!(
            next_special_night(date(2027, 1, 6), &[]).unwrap().name,
            "Krampusnacht"
        );
    }
}
