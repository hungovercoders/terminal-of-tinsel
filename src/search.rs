//! Searching the creatures' names and lore, for the explorer's `/`.

use crate::creatures::{Creature, fold};

/// A creature matching a search, with where and why it matched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    /// The creature's position in the list that was searched.
    pub index: usize,
    /// The field that matched: name, alias, fact and so on.
    pub field: &'static str,
    /// The matching text, trimmed to a window around the match.
    pub snippet: String,
}

/// Finds creatures whose name, aliases or lore mention the query. Case and
/// accents don't matter. Each creature appears at most once, at its
/// best-ranked matching field.
pub fn search(creatures: &[Creature], query: &str) -> Vec<Hit> {
    let needle: Vec<char> = query.trim().chars().map(fold).collect();
    if needle.is_empty() {
        return Vec::new();
    }
    creatures
        .iter()
        .enumerate()
        .filter_map(|(index, c)| {
            fields(c).into_iter().find_map(|(field, text)| {
                let chars: Vec<char> = text.chars().collect();
                let folded: Vec<char> = chars.iter().copied().map(fold).collect();
                let at = folded.windows(needle.len()).position(|w| w == needle)?;
                Some(Hit {
                    index,
                    field,
                    snippet: snippet(&chars, at, needle.len()),
                })
            })
        })
        .collect()
}

/// What is searched, most important first.
fn fields(c: &Creature) -> Vec<(&'static str, String)> {
    let mut out = vec![("name", c.name.clone())];
    out.extend(c.aliases.iter().map(|a| ("alias", a.clone())));
    out.push(("description", c.description.clone()));
    out.push(("origin", c.origin.clone()));
    if let Some(f) = &c.film {
        out.push(("film", format!("{} · {} · {}", f.title, f.director, f.star)));
        // The note is on the On Screen page, so it should be findable too.
        out.extend(f.note.iter().map(|note| ("film", note.clone())));
    }
    out.extend(c.facts.iter().map(|f| ("fact", f.clone())));
    out.push(("legend", c.legend.clone()));
    out.extend(
        c.myths
            .iter()
            .map(|m| ("myth", format!("{} {}", m.claim, m.explanation))),
    );
    out.extend(
        c.quotes
            .iter()
            .map(|q| ("quote", format!("{} ({})", q.text, q.source))),
    );
    out.extend(c.legacy.iter().map(|l| ("legacy", l.clone())));
    out
}

/// Trims long text to a window around the match.
fn snippet(chars: &[char], at: usize, len: usize) -> String {
    const PAD: usize = 30;
    let from = at.saturating_sub(PAD);
    let to = (at + len + PAD).min(chars.len());
    let mut out = String::new();
    if from > 0 {
        out.push('…');
    }
    out.extend(&chars[from..to]);
    if to < chars.len() {
        out.push('…');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creatures::Vault;

    fn ids(query: &str) -> Vec<String> {
        let v = Vault::load();
        search(&v.creatures, query)
            .iter()
            .map(|h| v.creatures[h.index].id.clone())
            .collect()
    }

    #[test]
    fn names_rank_before_lore() {
        let v = Vault::load();
        let hits = search(&v.creatures, "krampus");
        let first = &hits[0];
        assert_eq!(v.creatures[first.index].id, "krampus");
        assert_eq!(first.field, "name");
    }

    #[test]
    fn case_and_accents_do_not_matter() {
        assert!(ids("GRYLA").contains(&"gryla".to_string()));
        assert!(ids("grýla").contains(&"gryla".to_string()));
        assert!(ids("pere fouettard").contains(&"pere-fouettard".to_string()));
    }

    #[test]
    fn lore_is_searched_and_each_creature_appears_once() {
        let found = ids("porridge");
        assert!(found.contains(&"tomte".to_string()), "{found:?}");
        let mut unique = found.clone();
        unique.dedup();
        assert_eq!(found, unique);
        assert!(ids("zzzz").is_empty());
        assert!(ids("   ").is_empty());
    }

    #[test]
    fn film_notes_are_searched() {
        // Marley's film note names the actor who played Scrooge.
        let v = Vault::load();
        let hits = search(&v.creatures, "alastair sim");
        assert_eq!(hits.len(), 1, "{hits:?}");
        assert_eq!(v.creatures[hits[0].index].id, "marley");
        assert_eq!(hits[0].field, "film");
    }

    #[test]
    fn snippets_are_a_window_round_the_match() {
        let text: Vec<char> = format!("{}needle{}", "a".repeat(50), "b".repeat(50))
            .chars()
            .collect();
        let s = snippet(&text, 50, 6);
        assert!(s.starts_with('…') && s.ends_with('…'));
        assert!(s.contains("needle"));
        assert_eq!(s.chars().count(), 30 + 6 + 30 + 2);
        let short: Vec<char> = "needle".chars().collect();
        assert_eq!(snippet(&short, 0, 6), "needle");
    }
}
