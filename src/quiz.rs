//! The Christmas Cracker Quiz: multiple-choice questions built from the
//! creature data, so every new fact, film credit or myth becomes quiz
//! material for free.

use serde::Serialize;

use crate::creatures::{Creature, fold};
use crate::rng::Rng;

/// What a question is about. No kind may take more than a third of a quiz.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Fact,
    Quote,
    Debut,
    Myth,
    Film,
    Year,
}

/// One multiple-choice question.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Question {
    pub kind: Kind,
    pub prompt: String,
    /// Quoted text shown under the prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clue: Option<String>,
    pub options: Vec<String>,
    /// The index of the right option.
    pub answer: usize,
    pub explanation: String,
    pub creature_id: String,
}

impl Question {
    /// The text of the right answer.
    pub fn correct(&self) -> &str {
        &self.options[self.answer]
    }
}

/// Names a quiz score, from 0 to 100.
pub fn rank(percent: usize) -> &'static str {
    match percent {
        90.. => "Star of the Tree",
        70.. => "Carol Scholar",
        40.. => "Cracker Puller",
        _ => "Stocking Filler",
    }
}

/// What the host makes of a score, from 0 to 100.
pub fn final_word(percent: usize) -> &'static str {
    match percent {
        90.. => {
            "Magnificent! You must have been reading folklore by the fire since you were in your cradle."
        }
        70.. => "Very impressive. Our guests are a little nervous about how much you know.",
        40.. => "Not bad at all. Stay in for a few more specials and you'll be an expert.",
        _ => "Oh dear, a lump of coal. But every scholar starts somewhere. Pull another cracker.",
    }
}

/// Builds up to `n` varied questions. With a `focus`, every question is
/// about that creature.
pub fn generate(rng: &mut Rng, all: &[Creature], n: usize, focus: Option<&str>) -> Vec<Question> {
    let mut pool: Vec<Question> = all
        .iter()
        .filter(|c| focus.is_none_or(|id| c.id == id))
        .flat_map(|c| questions_about(rng, c, all))
        .collect();
    rng.shuffle(&mut pool);

    // Keep a mix: no kind may take more than a third of the quiz, unless we
    // would otherwise run short.
    let limit = n.div_ceil(3).max(1);
    let mut per_kind = [0usize; 6];
    let (mut out, mut spare) = (Vec::new(), Vec::new());
    for q in pool {
        let count = &mut per_kind[q.kind as usize];
        if out.len() < n && *count < limit {
            *count += 1;
            out.push(q);
        } else {
            spare.push(q);
        }
    }
    let short = n.saturating_sub(out.len());
    out.extend(spare.into_iter().take(short));
    out
}

/// Every question we can ask about a creature. The order is fixed, so a
/// seeded generator always asks the same questions.
fn questions_about(rng: &mut Rng, c: &Creature, all: &[Creature]) -> Vec<Question> {
    let others: Vec<&str> = all
        .iter()
        .filter(|o| o.id != c.id)
        .map(|o| o.name.as_str())
        .collect();
    let question = |kind, prompt: String, clue: Option<String>, explanation: String| Question {
        kind,
        prompt,
        clue,
        options: Vec::new(),
        answer: 0,
        explanation,
        creature_id: c.id.clone(),
    };
    let mut out = Vec::new();

    for fact in &c.facts {
        let q = question(
            Kind::Fact,
            "Which creature is this about?".into(),
            Some(mask(fact, c)),
            format!("{}: {fact}", c.name),
        );
        out.extend(choice(rng, q, &c.name, &others));
    }

    for quote in c.quotes.iter().filter(|q| q.text.chars().count() >= 25) {
        let q = question(
            Kind::Quote,
            "Whose story does this line come from?".into(),
            Some(format!("“{}”", mask(&quote.text, c))),
            format!("{}, {}", quote.speaker, quote.source),
        );
        out.extend(choice(rng, q, &c.name, &others));
    }

    // No "which is its weakness?" questions: weaknesses written for different
    // creatures overlap in meaning, so a wrong option could be just as right.
    const MEDIA: [&str; 3] = ["A book", "A film", "Folklore and legend"];
    if let Some(answer) = debut_kind(&c.debut.medium) {
        let mut q = question(
            Kind::Debut,
            format!("Where did {} first appear?", c.name),
            None,
            format!("First appearance: {}", c.first_appearance()),
        );
        q.options = MEDIA.iter().map(|m| m.to_string()).collect();
        q.answer = answer;
        out.push(q);
    }

    for myth in &c.myths {
        let mut q = question(
            Kind::Myth,
            "True or false?".into(),
            Some(myth.claim.clone()),
            myth.explanation.clone(),
        );
        q.options = vec!["True".into(), "False".into()];
        q.answer = usize::from(!myth.is_true);
        out.push(q);
    }

    if let Some(film) = &c.film {
        let title = format!("{} ({})", film.title, film.year);
        let films = || {
            all.iter()
                .filter(|o| o.id != c.id)
                .filter_map(|o| o.film.as_ref())
        };
        let stars: Vec<&str> = films().map(|f| f.star.as_str()).collect();
        let directors: Vec<&str> = films().map(|f| f.director.as_str()).collect();
        let q = question(
            Kind::Film,
            format!("Who played {} in {title}?", mid_sentence(&c.name)),
            None,
            format!("{} played {} in {title}.", film.star, mid_sentence(&c.name)),
        );
        out.extend(choice(rng, q, &film.star, &stars));
        let q = question(
            Kind::Film,
            format!("Who directed {title}?"),
            None,
            format!("{} directed {title}.", film.director),
        );
        out.extend(choice(rng, q, &film.director, &directors));
        let q = question(
            Kind::Year,
            format!("In what year was {} released?", film.title),
            None,
            format!("{} came out in {}.", film.title, film.year),
        );
        let years = nearby_years(rng, film.year);
        let years: Vec<&str> = years.iter().map(String::as_str).collect();
        out.extend(choice(rng, q, &film.year.to_string(), &years));
    }
    out
}

/// Fills in up to four shuffled options: the answer and three distractors.
/// Returns nothing if there aren't at least two distractors to choose from.
fn choice(rng: &mut Rng, mut q: Question, answer: &str, distractors: &[&str]) -> Option<Question> {
    let mut wrong: Vec<&str> = Vec::new();
    for d in distractors {
        if !d.eq_ignore_ascii_case(answer) && !wrong.contains(d) {
            wrong.push(d);
        }
    }
    if wrong.len() < 2 {
        return None;
    }
    rng.shuffle(&mut wrong);
    wrong.truncate(3);
    let mut options = vec![answer];
    options.extend(wrong);
    rng.shuffle(&mut options);
    q.answer = options.iter().position(|o| *o == answer)?;
    q.options = options.into_iter().map(str::to_string).collect();
    Some(q)
}

/// Three different years within eight of the real one, either side.
fn nearby_years(rng: &mut Rng, year: i32) -> Vec<String> {
    let mut offsets: Vec<i32> = (-8..=8).filter(|o| *o != 0).collect();
    rng.shuffle(&mut offsets);
    offsets
        .iter()
        .take(3)
        .map(|o| (year + o).to_string())
        .collect()
}

/// Whether a debut was in a book, a film or folklore, as an index into the
/// question's options.
fn debut_kind(medium: &str) -> Option<usize> {
    let m = medium.to_lowercase();
    let any = |words: &[&str]| words.iter().any(|w| m.contains(w));
    if any(&["film"]) {
        Some(1)
    } else if any(&["novel", "story", "book", "poem", "play"]) {
        Some(0)
    } else if any(&["folklore", "legend", "myth"]) {
        Some(2)
    } else {
        None
    }
}

/// Lowercases a leading "The", so a name reads naturally mid-sentence.
pub fn mid_sentence(name: &str) -> String {
    match name.strip_prefix("The ") {
        Some(rest) => format!("the {rest}"),
        None => name.to_string(),
    }
}

/// What a hidden name is replaced with.
const MASK: &str = "▒▒▒▒";

/// Hides a creature's names in text, so the clue doesn't give it away. Case
/// and accents don't matter, a possessive goes too ("Marley's"), and a
/// single-word name also hides its compounds ("Krampusnacht").
pub fn mask(text: &str, c: &Creature) -> String {
    let mut terms: Vec<Vec<char>> = std::iter::once(c.name.as_str())
        .chain(c.name.strip_prefix("The "))
        .chain(c.aliases.iter().map(String::as_str))
        .map(|t| t.chars().map(fold).collect::<Vec<char>>())
        .filter(|t| t.len() >= 3)
        .collect();
    terms.sort_by_key(|t| std::cmp::Reverse(t.len()));

    let chars: Vec<char> = text.chars().collect();
    let folded: Vec<char> = chars.iter().copied().map(fold).collect();
    let word = |ch: char| ch.is_alphanumeric();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        let at_word_start = i == 0 || !word(folded[i - 1]);
        let hit = at_word_start
            .then(|| {
                terms.iter().find_map(|term| {
                    let mut end = i + term.len();
                    if !folded[i..].starts_with(term) {
                        return None;
                    }
                    if term.contains(&' ') {
                        // A phrase must end at a word boundary.
                        if folded.get(end).is_some_and(|ch| word(*ch)) {
                            return None;
                        }
                    } else {
                        // A single word takes the rest of its compound with it.
                        while folded.get(end).is_some_and(|ch| word(*ch)) {
                            end += 1;
                        }
                    }
                    // And a possessive.
                    if matches!(folded.get(end), Some('\'' | '’'))
                        && folded.get(end + 1) == Some(&'s')
                        && !folded.get(end + 2).is_some_and(|ch| word(*ch))
                    {
                        end += 2;
                    }
                    Some(end)
                })
            })
            .flatten();
        match hit {
            Some(end) => {
                out.push_str(MASK);
                i = end;
            }
            None => {
                out.push(chars[i]);
                i += 1;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creatures::Vault;

    #[test]
    fn a_seeded_quiz_is_repeatable() {
        let v = Vault::load();
        let a = generate(&mut Rng::new(25), &v.creatures, 10, None);
        let b = generate(&mut Rng::new(25), &v.creatures, 10, None);
        assert_eq!(a, b);
        assert_eq!(a.len(), 10);
        assert_ne!(a, generate(&mut Rng::new(26), &v.creatures, 10, None));
    }

    #[test]
    fn every_question_is_well_formed() {
        let v = Vault::load();
        let all = generate(&mut Rng::new(1), &v.creatures, usize::MAX, None);
        assert!(all.len() > 150, "only {} questions", all.len());
        for q in &all {
            assert!((2..=4).contains(&q.options.len()), "{q:?}");
            assert!(q.answer < q.options.len(), "{q:?}");
            let mut unique = q.options.clone();
            unique.sort();
            unique.dedup();
            assert_eq!(unique.len(), q.options.len(), "repeated option: {q:?}");
            assert!(!q.explanation.is_empty() && v.get(&q.creature_id).is_some());
        }
    }

    #[test]
    fn clues_never_name_the_answer() {
        let v = Vault::load();
        for q in generate(&mut Rng::new(3), &v.creatures, usize::MAX, None) {
            if !matches!(q.kind, Kind::Fact | Kind::Quote) {
                continue;
            }
            let clue = q.clue.as_deref().unwrap().to_lowercase();
            let name = q.correct().to_lowercase();
            let name = name.strip_prefix("the ").unwrap_or(&name);
            assert!(!clue.contains(name), "{name} is named in: {clue}");
        }
    }

    #[test]
    fn no_kind_takes_more_than_a_third() {
        let v = Vault::load();
        for n in [3, 6, 10, 15] {
            let quiz = generate(&mut Rng::new(n as u64), &v.creatures, n, None);
            assert_eq!(quiz.len(), n);
            for kind in [Kind::Fact, Kind::Myth, Kind::Quote, Kind::Debut] {
                let count = quiz.iter().filter(|q| q.kind == kind).count();
                assert!(count <= n.div_ceil(3), "{count} {kind:?} questions of {n}");
            }
        }
    }

    #[test]
    fn a_focused_quiz_is_about_one_creature() {
        let v = Vault::load();
        let quiz = generate(&mut Rng::new(5), &v.creatures, 8, Some("marley"));
        assert!(!quiz.is_empty());
        assert!(quiz.iter().all(|q| q.creature_id == "marley"));
        // One creature can't fill every kind's share, so the spares top it up.
        assert_eq!(quiz.len(), 8);
    }

    #[test]
    fn too_few_creatures_means_no_who_is_it_questions() {
        let v = Vault::load().only_packs(&["dickens".into()]).unwrap();
        let quiz = generate(&mut Rng::new(5), &v.creatures, usize::MAX, None);
        assert!(!quiz.is_empty());
        assert!(
            quiz.iter().all(|q| q.kind != Kind::Fact),
            "one wrong option isn't a quiz"
        );
    }

    #[test]
    fn masking_hides_names_possessives_and_compounds() {
        let v = Vault::load();
        let krampus = v.get("krampus").unwrap();
        assert_eq!(
            mask(
                "His night is Krampusnacht, and Krampus's chains rattle.",
                krampus
            ),
            "His night is ▒▒▒▒, and ▒▒▒▒ chains rattle."
        );
        let cat = v.get("yule-cat").unwrap();
        assert_eq!(
            mask("The Yule Cat eats the lazy.", cat),
            "▒▒▒▒ eats the lazy."
        );
        assert_eq!(mask("fed the yule cat's dinner", cat), "fed ▒▒▒▒ dinner");
        let gryla = v.get("gryla").unwrap();
        assert_eq!(
            mask("Grýla lives in a cave.", gryla),
            "▒▒▒▒ lives in a cave."
        );
        // Only whole words: "scat" keeps its cat.
        assert_eq!(mask("scat", cat), "scat");
    }

    #[test]
    fn ranks_and_mid_sentence_names() {
        assert_eq!(rank(100), "Star of the Tree");
        assert_eq!(rank(70), "Carol Scholar");
        assert_eq!(rank(40), "Cracker Puller");
        assert_eq!(rank(0), "Stocking Filler");
        assert_eq!(mid_sentence("The Yule Cat"), "the Yule Cat");
        assert_eq!(mid_sentence("Krampus"), "Krampus");
    }

    #[test]
    fn debuts_sort_into_book_film_or_folklore() {
        assert_eq!(debut_kind("novella"), Some(0));
        assert_eq!(debut_kind("silent film"), Some(1));
        assert_eq!(debut_kind("folklore"), Some(2));
        assert_eq!(debut_kind("opera"), None);
    }
}
