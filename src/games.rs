//! The game screens: the Christmas Cracker Quiz and Guess the Guest. The
//! questions and rounds are made in `quiz` and `guess`; this is the playing.

use std::collections::BTreeMap;

use ratatui::Frame;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};
use unicode_width::UnicodeWidthStr;

use crate::guess::{self, Round};
use crate::host;
use crate::quiz::{self, Question};
use crate::render::{GREEN, RED, SNOW, truncate};
use crate::rng::Rng;
use crate::tui::{self, App, Ink, Lines, beside, boxed, join_fit, rgb, wrap};

/// The games are drawn at most this wide, centred on wider terminals.
const MAX_WIDTH: usize = 100;

// ── What the two games share ───────────────────────────────────────────────

/// The keys that leave a game. `ctrl+c` always does.
fn is_quit(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
        || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
}

/// The keys that move on after an answer.
fn is_next(code: KeyCode) -> bool {
    matches!(
        code,
        KeyCode::Enter | KeyCode::Char(' ' | 'n') | KeyCode::Right
    )
}

/// Moves the cursor for up and down, and returns the option chosen by
/// enter, by its number or by its letter.
fn pick_option(code: KeyCode, cursor: &mut usize, n: usize) -> Option<usize> {
    match code {
        KeyCode::Up | KeyCode::Char('k') => *cursor = (*cursor + n - 1) % n,
        KeyCode::Down | KeyCode::Char('j') => *cursor = (*cursor + 1) % n,
        KeyCode::Enter => return Some(*cursor),
        KeyCode::Char(c @ '1'..='9') => return Some(c as usize - '1' as usize).filter(|i| *i < n),
        KeyCode::Char(c @ 'a'..='d') => return Some(c as usize - 'a' as usize).filter(|i| *i < n),
        _ => {}
    }
    None
}

/// One multiple-choice option, marked right or wrong once the player has
/// chosen.
fn option_line(
    ink: Ink,
    i: usize,
    text: &str,
    selected: bool,
    answered: bool,
    is_answer: bool,
    is_chosen: bool,
) -> Line<'static> {
    let label = format!("{}. {text}", i + 1);
    let (marker, marker_style, style) = if answered && is_answer {
        ("✔ ", ink.good(), ink.good())
    } else if answered && is_chosen {
        ("✘ ", ink.bad(), ink.bad())
    } else if answered {
        ("  ", Style::new(), ink.dim())
    } else if selected {
        ("▶ ", ink.fg(rgb(RED)), ink.heading())
    } else {
        ("  ", Style::new(), Style::new())
    };
    Line::from(vec![
        Span::styled(marker, marker_style),
        Span::styled(label, style),
    ])
}

fn keys(ink: Ink, items: &[&str], width: usize) -> Lines {
    let spans = items
        .iter()
        .map(|k| Span::styled(k.to_string(), ink.dim()))
        .collect();
    join_fit(spans, " · ", width)
}

fn host_line(ink: Ink, text: &str, width: usize) -> Lines {
    wrap(&format!("📺 {text}"), width, ink.host())
}

/// The width to draw at, assuming 80 columns until told otherwise.
fn game_width(width: u16) -> usize {
    match usize::from(width) {
        0 => 80,
        w => w.min(MAX_WIDTH),
    }
}

/// Lays a game's lines on the frame, centred on a wide terminal.
fn place(lines: Lines, width: usize, frame: &mut Frame) {
    let area = frame.area();
    let width = u16::try_from(width).unwrap_or(u16::MAX).min(area.width);
    let x = area.x + (area.width - width) / 2;
    Paragraph::new(lines).render(Rect::new(x, area.y, width, area.height), frame.buffer_mut());
}

// ── The Christmas Cracker Quiz ─────────────────────────────────────────────

/// How a quiz went.
#[derive(Debug, Clone)]
pub struct QuizResult {
    pub questions: Vec<Question>,
    /// The option chosen for each question, once it has been answered.
    pub answers: Vec<Option<usize>>,
    /// Whether the quiz was played to the end.
    pub completed: bool,
}

impl QuizResult {
    /// How many answers were right, and how many were given.
    pub fn score(&self) -> (usize, usize) {
        let given = self.answers.iter().flatten().count();
        let right = self
            .questions
            .iter()
            .zip(&self.answers)
            .filter(|(q, a)| **a == Some(q.answer))
            .count();
        (right, given)
    }

    /// Correct answers per creature id, for the stocking.
    pub fn correct_by(&self) -> BTreeMap<String, u32> {
        let mut out = BTreeMap::new();
        for (q, a) in self.questions.iter().zip(&self.answers) {
            if *a == Some(q.answer) {
                *out.entry(q.creature_id.clone()).or_default() += 1;
            }
        }
        out
    }

    /// The score as a percentage of the questions answered.
    pub fn percent(&self) -> usize {
        match self.score() {
            (_, 0) => 0,
            (right, given) => right * 100 / given,
        }
    }
}

pub struct QuizApp {
    result: QuizResult,
    /// The question being asked.
    i: usize,
    cursor: usize,
    answered: bool,
    finished: bool,
    quip: &'static str,
    rng: Rng,
    ink: Ink,
    width: u16,
    quitting: bool,
}

impl QuizApp {
    /// The quiz must have at least one question.
    pub fn new(questions: Vec<Question>, rng: Rng, colour: bool) -> Self {
        Self {
            result: QuizResult {
                answers: vec![None; questions.len()],
                questions,
                completed: false,
            },
            i: 0,
            cursor: 0,
            answered: false,
            finished: false,
            quip: "",
            rng,
            ink: Ink { colour },
            width: 0,
            quitting: false,
        }
    }

    pub fn into_result(self) -> QuizResult {
        self.result
    }

    fn lines(&self) -> Lines {
        let ink = self.ink;
        let w = game_width(self.width);
        let total = self.result.questions.len();
        let (right, _) = self.result.score();
        let mut out = vec![
            tui::title_bar(
                ink,
                "🎉 THE CHRISTMAS CRACKER QUIZ",
                &format!("Question {} of {total} · Score {right}", self.i + 1),
                w,
            ),
            Line::default(),
        ];
        if self.finished {
            let pct = self.result.percent();
            out.push(Line::styled(
                format!("Final score: {right} of {total} ({pct}%)"),
                ink.heading(),
            ));
            out.push(Line::styled(
                format!("Your rank: {}", quiz::rank(pct)),
                ink.good(),
            ));
            out.push(Line::default());
            let word = format!("{}: “{}”", host::NAME, quiz::final_word(pct));
            out.extend(host_line(ink, &word, w));
            out.push(Line::default());
            out.push(Line::styled("Press any key to leave the studio", ink.dim()));
            return out;
        }

        let q = &self.result.questions[self.i];
        let chosen = self.result.answers[self.i];
        out.extend(wrap(
            &q.prompt,
            w,
            Style::new().add_modifier(Modifier::BOLD),
        ));
        if let Some(clue) = &q.clue {
            let style = ink.fg(rgb(SNOW)).add_modifier(Modifier::ITALIC);
            for mut line in wrap(clue, w.saturating_sub(2), style) {
                line.spans.insert(0, Span::raw("  "));
                out.push(line);
            }
        }
        out.push(Line::default());
        for (i, option) in q.options.iter().enumerate() {
            out.push(option_line(
                ink,
                i,
                option,
                self.cursor == i,
                self.answered,
                i == q.answer,
                chosen == Some(i),
            ));
        }
        out.push(Line::default());
        if self.answered {
            if chosen == Some(q.answer) {
                out.push(Line::styled("✔ Snap! Correct.", ink.good()));
            } else {
                // Don't double the full stop after an answer that ends in one.
                let answer = q.correct().trim_end_matches('.');
                let text = format!("✘ Not quite. The answer is {answer}.");
                out.extend(wrap(&text, w, ink.bad()));
            }
            out.extend(wrap(&q.explanation, w, ink.fg(rgb(SNOW))));
            out.extend(host_line(ink, self.quip, w));
            out.push(Line::default());
            out.extend(keys(ink, &["enter next question", "q quit"], w));
        } else {
            out.extend(keys(
                ink,
                &["1-4 or a-d answer", "↑/↓ + enter", "q quit"],
                w,
            ));
        }
        out
    }
}

impl App for QuizApp {
    fn on_key(&mut self, key: KeyEvent) {
        if is_quit(key) || self.finished {
            self.quitting = true;
        } else if self.answered {
            if is_next(key.code) {
                self.answered = false;
                self.cursor = 0;
                if self.i + 1 == self.result.questions.len() {
                    self.finished = true;
                    self.result.completed = true;
                } else {
                    self.i += 1;
                }
            }
        } else {
            let code = match key.code {
                KeyCode::Char(' ') => KeyCode::Enter,
                code => code,
            };
            let options = self.result.questions[self.i].options.len();
            if let Some(n) = pick_option(code, &mut self.cursor, options) {
                self.result.answers[self.i] = Some(n);
                self.answered = true;
                self.cursor = n;
                self.quip = host::quip(&mut self.rng);
            }
        }
    }

    fn resize(&mut self, width: u16, _height: u16) {
        self.width = width;
    }

    fn draw(&self, frame: &mut Frame) {
        place(self.lines(), game_width(self.width), frame);
    }

    fn quitting(&self) -> bool {
        self.quitting
    }
}

// ── Guess the Guest ────────────────────────────────────────────────────────

/// How a game of Guess the Guest went.
#[derive(Debug, Clone)]
pub struct GuessResult<'a> {
    pub rounds: Vec<Round<'a>>,
    /// Whether the game was played to the end.
    pub completed: bool,
}

impl GuessResult<'_> {
    /// The total points.
    pub fn score(&self) -> usize {
        self.rounds.iter().map(Round::points).sum()
    }

    /// The most points the rounds could have scored.
    pub fn best(&self) -> usize {
        self.rounds.len() * guess::POINTS[0]
    }

    /// Correct guesses per creature id, for the stocking.
    pub fn correct_by(&self) -> BTreeMap<String, u32> {
        let mut out = BTreeMap::new();
        for r in self.rounds.iter().filter(|r| r.correct()) {
            *out.entry(r.creature.id.clone()).or_default() += 1;
        }
        out
    }

    /// Whether any guest was named through the thickest frost.
    pub fn frost_reader(&self) -> bool {
        self.rounds.iter().any(|r| r.correct() && r.stage == 0)
    }

    /// How many rounds were guessed at, right or wrong.
    pub fn played(&self) -> usize {
        self.rounds.iter().filter(|r| r.chosen.is_some()).count()
    }
}

pub struct GuessApp<'a> {
    result: GuessResult<'a>,
    /// The round being played.
    i: usize,
    /// How far the frost has cleared this round.
    stage: usize,
    cursor: usize,
    answered: bool,
    finished: bool,
    quip: &'static str,
    rng: Rng,
    ink: Ink,
    width: u16,
    quitting: bool,
}

impl<'a> GuessApp<'a> {
    /// The game must have at least one round.
    pub fn new(rounds: Vec<Round<'a>>, rng: Rng, colour: bool) -> Self {
        Self {
            result: GuessResult {
                rounds,
                completed: false,
            },
            i: 0,
            stage: 0,
            cursor: 0,
            answered: false,
            finished: false,
            quip: "",
            rng,
            ink: Ink { colour },
            width: 0,
            quitting: false,
        }
    }

    pub fn into_result(self) -> GuessResult<'a> {
        self.result
    }

    /// The portrait in its box: frosted while the round is open, and in the
    /// creature's own colour once it's over.
    fn window(&self, round: &Round) -> (Lines, usize) {
        let ink = self.ink;
        let stage = if self.answered {
            guess::CLEAR
        } else {
            self.stage
        };
        let primary = ink.fg(rgb(&round.creature.theme.primary));
        let shown = if self.answered {
            primary
        } else {
            ink.fg(rgb(SNOW))
        };
        let cells = round.frosted(stage);
        let inner = cells.first().map_or(0, Vec::len);
        let lines = cells
            .iter()
            .enumerate()
            .map(|(y, row)| {
                Line::from(
                    row.iter()
                        .enumerate()
                        .map(|(x, cell)| match cell {
                            Some(ch) => Span::styled(ch.to_string(), shown),
                            None => Span::styled(guess::frost(x, y).to_string(), ink.dim()),
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .collect();
        let accent = ink.fg(rgb(&round.creature.theme.accent));
        (boxed(lines, inner, 1, 0, accent), inner + 4)
    }

    fn lines(&self) -> Lines {
        let ink = self.ink;
        let w = game_width(self.width);
        let total = self.result.rounds.len();
        let mut out = vec![
            tui::title_bar(
                ink,
                "🔍 GUESS THE GUEST",
                &format!(
                    "Round {} of {total} · {} pts",
                    self.i + 1,
                    self.result.score()
                ),
                w,
            ),
            Line::default(),
        ];
        if self.finished {
            let (score, best) = (self.result.score(), self.result.best());
            let text = format!("Final score: {score} of a possible {best} points");
            out.extend(wrap(&text, w, ink.heading()));
            out.push(Line::default());
            let word = quiz::final_word(score * 100 / best.max(1));
            out.extend(host_line(ink, &format!("{}: “{word}”", host::NAME), w));
            out.push(Line::default());
            out.push(Line::styled("Press any key to leave the studio", ink.dim()));
            return out;
        }

        let round = &self.result.rounds[self.i];
        let (window, window_width) = self.window(round);

        // Put the options beside the portrait only if the widest possible
        // line fits. Judging the whole round up front means a new clue can
        // never flip the layout.
        let need = round
            .options
            .iter()
            .map(|o| o.width() + 5)
            .chain([
                guess::status(guess::CLEAR, false).width(),
                guess::status(guess::CLEAR, true).width(),
                24,
            ])
            .max()
            .unwrap_or(24);
        let room = w.saturating_sub(window_width + 3);
        let side_by_side = room >= need;
        let side_width = if side_by_side { room } else { w };

        let mut side = vec![Line::styled(
            "Who's at the window?",
            Style::new().add_modifier(Modifier::BOLD),
        )];
        let stage = if self.answered {
            round.stage
        } else {
            self.stage
        };
        side.extend(wrap(
            &guess::status(stage, self.answered),
            side_width,
            ink.dim(),
        ));
        side.push(Line::default());
        for (i, option) in round.options.iter().enumerate() {
            // "✔ 1. " takes five columns before the name.
            side.push(option_line(
                ink,
                i,
                &truncate(option, side_width.saturating_sub(5).max(10)),
                self.cursor == i,
                self.answered,
                i == round.answer,
                round.chosen == Some(i),
            ));
        }
        side.push(Line::default());
        for clue in round.clues(self.stage) {
            let style = ink.fg(rgb(GREEN)).add_modifier(Modifier::ITALIC);
            side.extend(wrap(&format!("Clue: {clue}"), side_width, style));
        }

        if side_by_side {
            out.extend(beside(window, window_width, 3, side));
        } else {
            out.extend(window);
            out.push(Line::default());
            out.extend(side);
        }
        out.push(Line::default());

        let c = round.creature;
        if self.answered {
            if round.correct() {
                let name = quiz::mid_sentence(&c.name);
                let text = format!("✔ It's {name}! +{} points", round.points());
                out.extend(wrap(&text, w, ink.good()));
            } else {
                let name = quiz::mid_sentence(&c.name);
                out.extend(wrap(&format!("✘ It was {name}."), w, ink.bad()));
            }
            let about = format!(
                "{}. First appeared: {}.",
                c.description.trim_end_matches('.'),
                c.first_appearance().trim_end_matches('.')
            );
            out.extend(wrap(&about, w, ink.fg(rgb(SNOW))));
            out.extend(host_line(ink, self.quip, w));
            out.push(Line::default());
            out.extend(keys(ink, &["enter next round", "q quit"], w));
        } else {
            out.extend(keys(
                ink,
                &[
                    "space clear the frost",
                    "1-4 guess",
                    "↑/↓ + enter",
                    "q quit",
                ],
                w,
            ));
        }
        out
    }
}

impl App for GuessApp<'_> {
    fn on_key(&mut self, key: KeyEvent) {
        if is_quit(key) || self.finished {
            self.quitting = true;
        } else if self.answered {
            if is_next(key.code) {
                if self.i + 1 == self.result.rounds.len() {
                    self.finished = true;
                    self.result.completed = true;
                } else {
                    self.i += 1;
                    self.stage = 0;
                    self.cursor = 0;
                    self.answered = false;
                }
            }
        } else if matches!(key.code, KeyCode::Char(' ' | 'f')) {
            self.stage = (self.stage + 1).min(guess::CLEAR);
        } else {
            let round = &mut self.result.rounds[self.i];
            if let Some(n) = pick_option(key.code, &mut self.cursor, round.options.len()) {
                round.chosen = Some(n);
                round.stage = self.stage;
                self.answered = true;
                self.cursor = n;
                self.quip = host::quip(&mut self.rng);
            }
        }
    }

    fn resize(&mut self, width: u16, _height: u16) {
        self.width = width;
    }

    fn draw(&self, frame: &mut Frame) {
        place(self.lines(), game_width(self.width), frame);
    }

    fn quitting(&self) -> bool {
        self.quitting
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creatures::Vault;
    use crate::tui::snapshot;

    fn press(app: &mut impl App, code: KeyCode) {
        app.on_key(KeyEvent::new(code, KeyModifiers::NONE));
    }

    fn quiz_app(v: &Vault, n: usize) -> QuizApp {
        let mut rng = Rng::new(25);
        let questions = quiz::generate(&mut rng, &v.creatures, n, None);
        QuizApp::new(questions, rng, true)
    }

    fn guess_app(v: &Vault, n: usize) -> GuessApp<'_> {
        let mut rng = Rng::new(25);
        let rounds = guess::rounds(&mut rng, &v.creatures, n);
        GuessApp::new(rounds, rng, true)
    }

    #[test]
    fn a_perfect_quiz_ends_at_the_top_of_the_tree() {
        let v = Vault::load();
        let mut app = quiz_app(&v, 5);
        for i in 0..5 {
            let text = snapshot(&mut app, 80, 24);
            assert!(
                text.contains(&format!("Question {} of 5 · Score {i}", i + 1)),
                "{text}"
            );
            let answer = app.result.questions[i].answer;
            press(&mut app, KeyCode::Char(char::from(b'1' + answer as u8)));
            let text = snapshot(&mut app, 80, 24);
            assert!(text.contains("✔ Snap! Correct."), "{text}");
            assert!(text.contains("enter next question"), "{text}");
            press(&mut app, KeyCode::Enter);
        }
        let text = snapshot(&mut app, 80, 24);
        assert!(text.contains("Final score: 5 of 5 (100%)"), "{text}");
        assert!(text.contains("Your rank: Star of the Tree"), "{text}");
        assert!(!app.quitting());
        press(&mut app, KeyCode::Char('x'));
        assert!(app.quitting());
        let result = app.into_result();
        assert!(result.completed);
        assert_eq!(result.score(), (5, 5));
        assert_eq!(result.correct_by().values().sum::<u32>(), 5);
    }

    #[test]
    fn a_wrong_answer_is_explained_with_one_full_stop() {
        let v = Vault::load();
        let mut app = quiz_app(&v, 3);
        let q = app.result.questions[0].clone();
        let wrong = (q.answer + 1) % q.options.len();
        // Move the cursor to the wrong option and choose it with space.
        for _ in 0..wrong {
            press(&mut app, KeyCode::Down);
        }
        press(&mut app, KeyCode::Char(' '));
        let text = snapshot(&mut app, 80, 24);
        let answer = q.correct().trim_end_matches('.');
        assert!(text.contains("✘ Not quite. The answer is"), "{text}");
        assert!(!text.contains(&format!("{answer}..")), "{text}");
        assert!(text.contains(&format!("✘ {}. ", wrong + 1)), "{text}");
        assert!(text.contains(&format!("✔ {}. ", q.answer + 1)), "{text}");
        assert_eq!(app.result.percent(), 0);
    }

    #[test]
    fn quitting_early_leaves_the_quiz_incomplete() {
        let v = Vault::load();
        let mut app = quiz_app(&v, 3);
        press(&mut app, KeyCode::Char('a'));
        press(&mut app, KeyCode::Char('q'));
        assert!(app.quitting());
        let result = app.into_result();
        assert!(!result.completed);
        assert_eq!(result.score().1, 1);
        let mut app = quiz_app(&v, 3);
        app.on_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(app.quitting());
    }

    #[test]
    fn keys_that_are_not_options_do_nothing() {
        let v = Vault::load();
        let mut app = quiz_app(&v, 3);
        let options = app.result.questions[0].options.len();
        press(&mut app, KeyCode::Char('9'));
        press(&mut app, KeyCode::Char('z'));
        assert!(!app.answered);
        press(&mut app, KeyCode::Up);
        assert_eq!(app.cursor, options - 1);
    }

    #[test]
    fn clearing_the_frost_costs_points_and_brings_clues() {
        let v = Vault::load();
        let mut app = guess_app(&v, 2);
        let text = snapshot(&mut app, 90, 30);
        assert!(
            text.contains("Frost cleared: 15% · worth 100 pts"),
            "{text}"
        );
        assert!(!text.contains("Clue:"), "{text}");
        for _ in 0..9 {
            press(&mut app, KeyCode::Char(' '));
        }
        let text = snapshot(&mut app, 90, 30);
        assert!(
            text.contains("Frost cleared: 100% · worth 20 pts"),
            "{text}"
        );
        assert!(text.contains("Clue: Origin:"), "{text}");
        let answer = app.result.rounds[0].answer;
        press(&mut app, KeyCode::Char(char::from(b'1' + answer as u8)));
        let text = snapshot(&mut app, 90, 30);
        assert!(text.contains("+20 points"), "{text}");
        assert!(
            text.contains("You guessed with 100% of the frost cleared"),
            "{text}"
        );
        press(&mut app, KeyCode::Enter);
        assert_eq!(app.stage, 0, "the next window starts frosted");
        assert!(snapshot(&mut app, 90, 30).contains("Round 2 of 2 · 20 pts"));
    }

    #[test]
    fn a_guess_through_the_thickest_frost_scores_a_hundred() {
        let v = Vault::load();
        let mut app = guess_app(&v, 1);
        let answer = app.result.rounds[0].answer;
        press(&mut app, KeyCode::Char(char::from(b'1' + answer as u8)));
        press(&mut app, KeyCode::Enter);
        let text = snapshot(&mut app, 90, 30);
        assert!(
            text.contains("Final score: 100 of a possible 100 points"),
            "{text}"
        );
        press(&mut app, KeyCode::Enter);
        let result = app.into_result();
        assert!(result.completed);
        assert_eq!((result.score(), result.played()), (100, 1));
        assert!(result.frost_reader());
        assert_eq!(result.correct_by().len(), 1);
        assert_eq!(result.rounds[0].stage, 0);
    }

    #[test]
    fn the_portrait_only_shows_in_full_after_the_guess() {
        let v = Vault::load();
        let mut app = guess_app(&v, 1);
        let art: Vec<String> = app.result.rounds[0]
            .creature
            .ascii
            .lines()
            .map(str::to_string)
            .collect();
        let widest = art
            .iter()
            .max_by_key(|l| l.trim().len())
            .unwrap()
            .trim()
            .to_string();
        assert!(!snapshot(&mut app, 90, 30).contains(&widest));
        press(&mut app, KeyCode::Char('2'));
        assert!(snapshot(&mut app, 90, 30).contains(&widest));
    }

    #[test]
    fn the_games_draw_at_every_size() {
        let v = Vault::load();
        for (width, height) in [(1, 1), (20, 6), (40, 12), (60, 40), (80, 24), (200, 60)] {
            let mut quiz = quiz_app(&v, 4);
            let mut guess = guess_app(&v, 14);
            for step in 0..40 {
                snapshot(&mut quiz, width, height);
                snapshot(&mut guess, width, height);
                let code = match step % 4 {
                    0 => KeyCode::Char(' '),
                    1 => KeyCode::Char('1'),
                    _ => KeyCode::Enter,
                };
                if !quiz.quitting() {
                    press(&mut quiz, code);
                }
                if !guess.quitting() {
                    press(&mut guess, code);
                }
            }
        }
    }

    #[test]
    fn options_sit_beside_the_window_when_there_is_room() {
        let v = Vault::load();
        let mut app = guess_app(&v, 14);
        // Every creature gets a turn, so the widest portrait is tried too.
        for _ in 0..14 {
            let wide = snapshot(&mut app, 120, 40);
            assert!(
                wide.lines()
                    .any(|l| l.contains('╭') && l.contains("Who's at the window?")),
                "{wide}"
            );
            let narrow = snapshot(&mut app, 40, 60);
            assert!(
                !narrow
                    .lines()
                    .any(|l| l.contains('╭') && l.contains("Who's")),
                "{narrow}"
            );
            press(&mut app, KeyCode::Char('1'));
            press(&mut app, KeyCode::Enter);
        }
    }

    /// `DUMP=1 cargo test dump_games -- --nocapture` prints sample screens.
    #[test]
    fn dump_games() {
        if std::env::var_os("DUMP").is_none() {
            return;
        }
        let v = Vault::load();
        let mut quiz = quiz_app(&v, 3);
        println!("── quiz ──\n{}\n", snapshot(&mut quiz, 90, 20));
        press(&mut quiz, KeyCode::Char('2'));
        println!("── quiz, answered ──\n{}\n", snapshot(&mut quiz, 90, 20));
        let mut guess = guess_app(&v, 3);
        println!("── guess ──\n{}\n", snapshot(&mut guess, 90, 28));
        press(&mut guess, KeyCode::Char(' '));
        press(&mut guess, KeyCode::Char(' '));
        println!("── guess, stage 3 ──\n{}\n", snapshot(&mut guess, 90, 28));
        press(&mut guess, KeyCode::Char('1'));
        println!("── guess, answered ──\n{}\n", snapshot(&mut guess, 90, 28));
    }
}
