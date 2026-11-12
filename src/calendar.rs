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
pub fn notes(d: Date) -> Vec<Note> {
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
        _ => {}
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
        let texts = |d| notes(d).into_iter().map(|n| n.text).collect::<Vec<_>>();
        assert!(texts(date(2026, 12, 5))[0].contains("Krampusnacht"));
        assert!(texts(date(2026, 12, 19))[0].contains("183 years ago"));
        assert!(texts(date(2026, 11, 12)).is_empty());
    }
}
