//! Making a community pack: `packs new <id>` writes a pack with an example
//! creature to edit.

use std::fs;
use std::path::{Path, PathBuf};

use crate::creatures::{BUILT_IN, Vault};
use crate::error::{Code, Result, TinselError};

/// The example creature's portrait: a present, waiting to be drawn over.
const EXAMPLE_ART: &str = r"     _\/_
   .-'--'-.
   |  ||  |
   |==::==|
   |  ||  |
   '------'
";

/// The id of the example creature in a new pack. It's based on the pack id,
/// so creatures from different packs never share an id.
pub fn example_id(pack: &str) -> String {
    format!("{pack}-creature")
}

/// Creates `<packs_dir>/<id>/` with a `pack.json`, an example creature and
/// its portrait, and returns the directory.
pub fn scaffold(vault: &Vault, packs_dir: &Path, id: &str) -> Result<PathBuf> {
    let well_formed = id.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if !well_formed {
        return Err(
            TinselError::new(Code::Usage, format!("{id:?} won't do as a pack id")).with_hint(
                "pack ids use lowercase letters, numbers and dashes, e.g. 'office-party'",
            ),
        );
    }
    // A community pack can never share an id with another: the loader would
    // skip it on every run.
    if let Some(taken) = vault.packs.iter().find(|p| p.id == id) {
        let whose = if taken.source == BUILT_IN {
            "a built-in pack"
        } else {
            "already one of your packs"
        };
        return Err(TinselError::new(Code::Usage, format!("{id:?} is {whose}"))
            .with_hint("choose another id"));
    }
    // The example creature's id must be free too, or the loader would skip
    // it on the next run and the new pack, left empty, wouldn't be listed.
    let creature = example_id(id);
    if let Some(taken) = vault.get(&creature) {
        return Err(TinselError::new(
            Code::Usage,
            format!(
                "the example creature would be called {creature:?}, and the {:?} pack already has one of those",
                taken.pack
            ),
        )
        .with_hint("choose another pack id"));
    }
    let dir = packs_dir.join(id);
    if dir.exists() {
        return Err(
            TinselError::new(Code::Usage, format!("{} already exists", dir.display()))
                .with_hint("choose another id, or edit the pack that's there"),
        );
    }

    let name = format!("My {id} creature");
    let pack = format!(
        "{{\n  \"id\": \"{id}\",\n  \"name\": \"My {id} pack\",\n  \"description\": \"Describe your pack here.\",\n  \"order\": [\"{creature}\"]\n}}\n"
    );
    let failed = |e: std::io::Error| {
        TinselError::new(
            Code::Store,
            format!("couldn't create {}: {e}", dir.display()),
        )
    };
    fs::create_dir_all(&dir).map_err(failed)?;
    fs::write(dir.join("pack.json"), pack).map_err(failed)?;
    fs::write(
        dir.join(format!("{creature}.json")),
        example_creature(&creature, &name),
    )
    .map_err(failed)?;
    fs::write(dir.join(format!("{creature}.txt")), EXAMPLE_ART).map_err(failed)?;
    Ok(dir)
}

/// An example creature with every field, to edit or delete from. It's a
/// template rather than built with `json!`, so the fields come in the order
/// a person would fill them in, not the alphabet's.
fn example_creature(id: &str, name: &str) -> String {
    const TEMPLATE: &str = r##"{
  "id": "{id}",
  "name": "{name}",
  "aliases": [],
  "emoji": "🎁",
  "description": "A one-line description of your creature",
  "origin": "Where the tale comes from",
  "debut": {"medium": "folklore", "title": "Where it first appeared", "era": "when"},
  "legend": "A paragraph about the real history or folklore behind the creature.",
  "myths": [
    {"claim": "A popular belief about the creature.", "true": false, "explanation": "What's really going on."}
  ],
  "quotes": [],
  "powers": ["A festive power"],
  "weaknesses": ["A surprising weakness"],
  "stats": {"strength": 5, "speed": 5, "cunning": 5, "dread": 5},
  "legacy": [],
  "facts": [
    "A fascinating fact",
    "Another fascinating fact",
    "Add at least three facts so the quiz has plenty to ask"
  ],
  "hostIntro": "What Auntie Aerial says to introduce your creature.",
  "theme": {"primary": "#E63946", "accent": "#E9C46A"}
}
"##;
    TEMPLATE.replace("{id}", id).replace("{name}", name)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            let dir =
                std::env::temp_dir().join(format!("tinsel-new-{name}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            Self(dir)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn a_new_pack_loads_cleanly_and_plays() {
        let scratch = Scratch::new("loads");
        let dir = scaffold(&Vault::load(), &scratch.0, "office-party").unwrap();
        assert!(dir.ends_with("office-party"));
        let mut vault = Vault::load();
        let warnings = vault.add_community(&scratch.0);
        assert!(warnings.is_empty(), "{warnings:?}");
        let pack = vault.packs.last().unwrap();
        assert_eq!(pack.id, "office-party");
        assert_eq!(pack.name, "My office-party pack");
        let c = vault.get("office-party-creature").unwrap();
        assert_eq!(c.pack, "office-party");
        assert_eq!(c.facts.len(), 3);
        assert!(c.ascii.contains("|==::==|"));
        // The files read top to bottom the way a person fills them in.
        let text = fs::read_to_string(dir.join("office-party-creature.json")).unwrap();
        assert!(text.starts_with("{\n  \"id\": \"office-party-creature\",\n  \"name\":"));
        assert!(
            fs::read_to_string(dir.join("pack.json"))
                .unwrap()
                .ends_with("}\n")
        );
        // The second attempt finds it's taken, whichever way it looks.
        let e = scaffold(&vault, &scratch.0, "office-party").unwrap_err();
        assert!(
            e.message.contains("already one of your packs"),
            "{}",
            e.message
        );
        let e = scaffold(&Vault::load(), &scratch.0, "office-party").unwrap_err();
        assert!(e.message.contains("already exists"), "{}", e.message);
    }

    #[test]
    fn built_in_and_badly_formed_ids_are_refused() {
        let scratch = Scratch::new("refused");
        let vault = Vault::load();
        let e = scaffold(&vault, &scratch.0, "nordic").unwrap_err();
        assert_eq!(e.code, Code::Usage);
        assert!(e.message.contains("built-in"), "{}", e.message);
        for bad in [
            "",
            "Office",
            "-party",
            "office party",
            "office_party",
            "../up",
            "café",
        ] {
            let e = scaffold(&vault, &scratch.0, bad).unwrap_err();
            assert_eq!(e.code, Code::Usage, "{bad:?}");
            assert!(e.hint.unwrap().contains("lowercase"), "{bad:?}");
        }
        assert!(!scratch.0.exists(), "a refusal creates nothing");
    }

    #[test]
    fn a_pack_whose_example_creature_is_already_taken_is_refused() {
        let scratch = Scratch::new("taken");
        // Somebody's attic already holds a creature with the id that
        // 'packs new office-party' would give its example.
        let attic = scratch.0.join("attic");
        fs::create_dir_all(&attic).unwrap();
        fs::write(
            attic.join("office-party-creature.json"),
            r#"{"name": "Early Guest", "description": "Got here first", "facts": ["It's early"]}"#,
        )
        .unwrap();
        let mut vault = Vault::load();
        assert!(vault.add_community(&scratch.0).is_empty());
        let e = scaffold(&vault, &scratch.0, "office-party").unwrap_err();
        assert_eq!(e.code, Code::Usage);
        assert!(
            e.message.contains("\"attic\" pack already has one"),
            "{}",
            e.message
        );
        assert_eq!(e.hint.as_deref(), Some("choose another pack id"));
        assert!(
            !scratch.0.join("office-party").exists(),
            "nothing is written"
        );
    }

    #[test]
    fn the_example_portrait_is_plain_ascii() {
        assert!(
            EXAMPLE_ART
                .chars()
                .all(|c| c == '\n' || (' '..='~').contains(&c))
        );
    }
}
