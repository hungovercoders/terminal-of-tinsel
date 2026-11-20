# Night 21 · The Longest Night

> 📺 *"Tonight is the winter solstice, give or take a day, and the longest night we have. Plenty of time, then, to teach the programme what day it is. By morning it will count the sleeps, read the moon and know who is behind tonight's door, and it will do all of it without once looking at the clock."*

**Tonight you'll learn**
- Dates with jiff: why a `civil::Date` is not a moment in time
- Pure functions of a date, and why they make the tests trustworthy
- `match` on a tuple with ranges and `@` bindings
- The moon's phase from one known new moon, and how far to trust it
- Daily seeds: a random choice that is the same for everyone all day
- Big block digits built from a table of strings

**Where we are:** the games are finished and the stocking remembers them. Tonight is the calendar: `countdown`, `advent`, `tonight` and `random --daily`.

## A date is not a moment

Rust's standard library can measure time (`std::time::Instant`, `SystemTime`) but it has no calendar. For that the program uses the **jiff** crate, and mostly one type from it, `jiff::civil::Date`: a year, a month and a day, with no time of day and no time zone.

"Is it Christmas?" is a question about a date. With a full timestamp, an hour's difference between two time zones could give two answers. So [`src/calendar.rs`](../src/calendar.rs) touches the clock in two small functions, `today()` and `now()`, and everything else takes a `Date`. `today()` is one line, `jiff::Zoned::now().date()`: the moment and the zone, with both thrown away and the day kept. Here is the function that decides which date a command is about:

```rust
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
```

`parse::<Date>()` works because jiff implements the standard `FromStr` trait for `Date`, the same trait behind `"25".parse::<u64>()`. `map_err` swaps jiff's error for a `TinselError` with the `bad-date` code and a hint the player can act on.

`countdown`, `advent`, `tonight` and `random` all start here. That is what `--date` is for: it lets you, and the tests, pretend it's any night of the year.

## Pure functions of a date

Here is the function behind the big number:

```rust
/// Sleeps until Christmas: 1 on Christmas Eve, 0 on Christmas Day, and from
/// Boxing Day the count starts again for next year.
pub fn sleeps_until_christmas(d: Date) -> i64 {
    // `Date::new` rather than `date`, which panics: after Christmas in the
    // year 9999 there is no next year to count to, and the answer is 0.
    let christmas = |year: i16| Date::new(year, 12, 25).ok();
    christmas(d.year())
        .filter(|this_year| d <= *this_year)
        .or_else(|| christmas(d.year().checked_add(1)?))
        .and_then(|target| d.until(target).ok())
        .map_or(0, |span| i64::from(span.get_days()))
}
```

It takes a date and returns a number. It reads no clock, no file and no global, so the same date always gives the same answer. That is what "pure" means, and it's why the test can be this plain:

```rust
    #[test]
    fn sleeps_count_down_to_christmas_and_start_again() {
        assert_eq!(sleeps_until_christmas(date(2026, 12, 24)), 1);
        assert_eq!(sleeps_until_christmas(date(2026, 12, 25)), 0);
        // ...
        assert_eq!(sleeps_until_christmas(date(2026, 12, 26)), 364);
        // 2028 is a leap year.
        assert_eq!(sleeps_until_christmas(date(2027, 12, 26)), 365);
    }
```

Had the function called `today()` itself, a test of it would pass in November and fail on Boxing Day. Because the date is an argument, the test covers Christmas Eve, Christmas Day and a leap year on any day you run it.

The body is a chain of `Option` methods, read top to bottom: this year's Christmas, if the date can be built; kept only if we haven't passed it (`filter`); otherwise next year's (`or_else`); then the span between the two dates (`and_then`); then its days, or 0 if anything along the way was `None` (`map_or`).

In jiff, `Date::new` returns a `Result`, because 30 February isn't a date and neither is anything after the year 9999. The code has to say what happens then, and a test called `the_far_end_of_the_calendar_does_not_panic` checks that the answer is never a crash.

## Rules as patterns

Some of the season's rules are small tables, and `match` writes them that way:

```rust
/// Which of the Twelve Days of Christmas this is (25 December is day 1, and
/// 5 January, Twelfth Night, is day 12), if any.
pub fn twelve_days(d: Date) -> Option<u8> {
    match (d.month(), d.day()) {
        (12, day @ 25..=31) => Some((day - 24) as u8),
        (1, day @ 1..=5) => Some((day + 7) as u8),
        _ => None,
    }
}
```

The value matched is a tuple of month and day. `25..=31` is a range pattern, and `day @ 25..=31` means "match a day in that range and call it `day`". Most dates aren't one of the Twelve Days at all, hence `Option<u8>`. This is the commonest reckoning, with Christmas Day as the first day; some traditions start counting on 26 December, which puts Twelfth Night on 6 January.

## The moon, near enough

The program works offline, so it can't look the moon up. It calculates it:

```rust
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
```

Count the days since a known new moon, then take the remainder after dividing by the length of a lunar month. `rem_euclid` is used instead of `%` because it never returns a negative number, so dates before 2000 work too.

This is an approximation and the doc comment says so. The real time between new moons varies by several hours either side of the average, because the moon's orbit is not a circle. Expect the answer to be right to within about a day: good enough to say "the Cold Moon is full tonight" (the Cold Moon being a traditional name for December's full moon), not good enough to plan an eclipse. The test only asks for what the method can deliver: the full moon of 24 December 2026 must be reported on the 23rd or the 24th.

`moon(d)` turns the age into a named phase and an illumination from 0 to 1.

## The same all day

`random` on its own seeds the generator from the clock (unless `TERMINAL_OF_TINSEL_SEED` is set), so every run is different. `random --daily` is the Fact of the Night, and it should be the same all day, for everyone. The whole trick is the seed:

```rust
/// The seed for the Fact of the Night: the same all day, for everyone.
pub fn daily_seed(d: Date) -> u64 {
    (i64::from(d.year()) * 10_000 + i64::from(d.month()) * 100 + i64::from(d.day())) as u64
}
```

Christmas Eve 2026 becomes the number 20261224. Feed that to the `Rng` you wrote on Night 4 and the "random" pick is fixed for the day and changes at midnight. This is also why the project keeps its own SplitMix64: a library generator may change its sequence between versions, and then two people on different releases would get different facts.

In advent the fact comes from the guest behind the night's door: `advent_guest` finds the door, then picks one of that creature's facts with `Rng::new(daily_seed(d))`. The doors themselves are data, in [`packs/advent.json`](../packs/advent.json): 24 entries, each with a night, a creature id and a note.

`tonight`'s double bill uses a related idea. `double_bill` shuffles the creatures once with a fixed seed, `Rng::new(25).shuffle(&mut order)`, so the running order is the same for everyone, then moves along it one place a night by counting days since 1 January 2000. A test checks every night of more than two years for a repeated bill.

## Big digits and a ticket

The drawing lives in [`src/render.rs`](../src/render.rs). The countdown's number is built from a table: ten digits, each five rows of three cells.

```rust
const DIGITS: [[&str; 5]; 10] = [
    ["###", "#.#", "#.#", "#.#", "###"],
    [".#.", "##.", ".#.", ".#.", "###"],
    // ...
];

/// A number in big block digits, five rows tall.
pub fn big_number(p: &Painter, n: i64) -> Vec<String> {
    let block = if p.plain { "##" } else { "██" };
    let digits: Vec<usize> = n
        .to_string()
        .chars()
        .filter_map(|c| c.to_digit(10).map(|d| d as usize))
        .collect();
    (0..5)
        .map(|row| {
            digits
                .iter()
                .map(|&d| {
                    DIGITS[d][row]
                        .chars()
                        .map(|c| if c == '#' { block } else { "  " })
                        .collect::<String>()
                })
                .collect::<Vec<_>>()
                .join("  ")
        })
        .collect()
}
```

A terminal prints a line at a time, so the function works row by row: for row 0, take row 0 of every digit and join them. Each `#` becomes two characters wide because terminal cells are about twice as tall as they are wide.

Here it is, piped and so in plain mode:

```bash
terminal-of-tinsel countdown --date 2026-12-21 | cat
```

```
     *         ##  ##
    /o\        ##  ##
   /.*.\       ######
  /o.*.o\          ##
 /.*.o.*.\         ##
/o.*.o.*.o\
    |_|        SLEEPS UNTIL CHRISTMAS

-- ADVENT · DOOR 21 ------------------------------------------------------------
Tonight's guest: The Wild Hunt
The longest night of the year. If you hear horns overhead, don't look up.
Jacob Grimm wrote about it as the Wilde Jagd in his Deutsche Mythologie of 1835

The winter solstice is here, or near enough: the longest night of the year.
10 Yule Lads are in town.

Tonight's moon: Waxing Gibbous (94% lit)
Next special night: Christmas Eve on 24 December (3 sleeps)
```

The lines under the door come from `calendar::notes`, which gathers whatever is true of the date: a feast, a film anniversary, a full moon, the Yule Lads, the Twelve Days. The solstice note is hedged in the code as well as on screen: the solstice usually falls on 21 or 22 December, depending on the year, and the table holds one fixed date. (Under the old Julian calendar the longest night had drifted to about 13 December, which is why St Lucy's night is also Lussi's long night.)

`render::ticket` draws `tonight`'s double bill as a cinema ticket: the box from Night 9 with a double border and rows of `- - -` for the perforation. As everywhere, `calendar.rs` decides what is true of a date and `render.rs` only turns those values into lines.

## Try it

- Run `cargo run -- countdown --date 2026-12-24`, then `--date 2026-12-25` and `--date 2026-12-26`. One sleep, none, then 364.
- Run `cargo run -- random --daily` three times, then `cargo run -- random` three times.
- Run `cargo test calendar` and read the names of the tests that pass.

## 🎄 Festive fact

jiff was written by Andrew Gallant, who also wrote ripgrep and Rust's `regex` crate, and its design owes a great deal to Temporal, JavaScript's newer date and time API. Both keep "civil" dates, which have no time zone, apart from zoned moments, so a date can't be compared with a moment by accident.

## 🕯️ Before morning

Outside December, `render::advent` works out the sleeps until the first door inline, with `jiff::civil::date(today.year(), 12, 1) - today`. On a branch of your own, move that sum into `calendar.rs` as `pub fn sleeps_until_advent(d: Date) -> i64` and give it a test.

*Hint:* copy the shape of `sleeps_until_christmas`, with `Date::new(...).ok()` in place of `date(...)` so that nothing can panic. `sleeps_until_advent(date(2026, 11, 30))` should be 1.

> 📺 *"There. The longest night, and the programme knew it before you did. Tomorrow we open the doors: to scripts that want our data, and to guests you've invented yourselves. Four sleeps to go."*

[← Night 20](night-20.md) · [Index](README.md) · [Night 22 →](night-22.md)
