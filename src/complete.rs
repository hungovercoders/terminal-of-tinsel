//! What the shell offers when you press tab: creature names and pack ids,
//! built-in and community alike. `CompleteEnv` in `main` calls these while
//! the shell waits, so they are quick and never print.

use clap_complete::CompletionCandidate;

use crate::creatures::Vault;
use crate::store;

/// Every pack, with warnings about broken community packs kept quiet: a
/// completion must not write to the terminal.
fn vault() -> Vault {
    let mut vault = Vault::load();
    if let Ok(dir) = store::dir() {
        let _ = vault.add_community(&store::packs_dir(&dir));
    }
    vault
}

/// Creature ids and one-word nicknames, each with the creature's name
/// beside it. Names of several words are left out: a shell would need them
/// quoted, and the id always works.
pub fn creatures() -> Vec<CompletionCandidate> {
    let mut seen = Vec::new();
    let mut out = Vec::new();
    for c in &vault().creatures {
        let keys = std::iter::once(c.id.clone()).chain(c.aliases.iter().map(|a| a.to_lowercase()));
        for key in keys.filter(|k| !k.contains(' ')) {
            if !seen.contains(&key) {
                seen.push(key.clone());
                out.push(CompletionCandidate::new(key).help(Some(c.name.clone().into())));
            }
        }
    }
    out
}

/// Pack ids, each with the pack's name beside it.
pub fn packs() -> Vec<CompletionCandidate> {
    vault()
        .packs
        .iter()
        .map(|p| CompletionCandidate::new(&p.id).help(Some(p.name.clone().into())))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values(candidates: Vec<CompletionCandidate>) -> Vec<String> {
        candidates
            .iter()
            .map(|c| c.get_value().to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn creatures_complete_by_id_and_one_word_nickname() {
        let all = values(creatures());
        for expected in ["krampus", "yule-cat", "nisse", "marley"] {
            assert!(all.contains(&expected.to_string()), "{expected} in {all:?}");
        }
        assert!(all.iter().all(|v| !v.contains(' ')), "{all:?}");
        let mut unique = all.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), all.len(), "no value is offered twice");
    }

    #[test]
    fn packs_complete_with_their_names() {
        let packs = packs();
        let nordic = packs.iter().find(|c| c.get_value() == "nordic").unwrap();
        assert_eq!(nordic.get_help().unwrap().to_string(), "Nordic Yule");
    }
}
