//! The creatures of Christmas folklore, their packs, and finding them by name.
//!
//! The data lives in `packs/` as JSON and is compiled into the binary, so the
//! CLI needs nothing on disk to run.

use std::collections::BTreeMap;

use include_dir::{Dir, include_dir};
use serde::{Deserialize, Serialize};

use crate::error::{Code, Result, TinselError};

static PACKS_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/packs");

/// The order packs appear in, from the saint's companions to Dickens.
pub const PACK_ORDER: &[&str] = &["companions", "nordic", "midwinter", "dickens"];

/// A single creature and everything we know about it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Creature {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
    pub emoji: String,
    pub description: String,
    pub origin: String,
    pub debut: Debut,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub film: Option<Film>,
    pub legend: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub myths: Vec<Myth>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub quotes: Vec<Quote>,
    pub powers: Vec<String>,
    pub weaknesses: Vec<String>,
    pub stats: Stats,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub legacy: Vec<String>,
    pub facts: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub host_intro: String,
    pub theme: Theme,
    /// Filled in from the directory the creature was loaded from.
    #[serde(default)]
    pub pack: String,
}

/// Where a creature first appeared.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Debut {
    pub medium: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub creator: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub year: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub era: Option<String>,
}

/// A creature's classic screen outing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Film {
    pub title: String,
    pub year: i32,
    pub director: String,
    pub star: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// A popular belief with a verdict and the real story.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Myth {
    pub claim: String,
    #[serde(rename = "true")]
    pub is_true: bool,
    pub explanation: String,
}

/// A line from a public-domain source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub text: String,
    pub speaker: String,
    pub source: String,
}

/// Each stat is 1-10 and purely for fun.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Stats {
    pub strength: u8,
    pub speed: u8,
    pub cunning: u8,
    pub dread: u8,
}

impl Stats {
    /// The stats with their labels, in display order.
    pub fn labelled(&self) -> [(&'static str, u8); 4] {
        [
            ("Strength", self.strength),
            ("Speed", self.speed),
            ("Cunning", self.cunning),
            ("Dread", self.dread),
        ]
    }
}

/// A creature's colours, as `#RRGGBB`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub primary: String,
    pub accent: String,
}

/// A pack of creatures, such as the Nordic Yule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pack {
    pub id: String,
    pub name: String,
    pub description: String,
    pub order: Vec<String>,
}

/// One door of the advent calendar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdventNight {
    pub night: u8,
    pub id: String,
    pub note: String,
}

#[derive(Debug, Clone, Deserialize)]
struct AdventFile {
    nights: Vec<AdventNight>,
}

/// Every creature, pack and advent door, loaded from the embedded data.
#[derive(Debug, Clone)]
pub struct Vault {
    pub packs: Vec<Pack>,
    pub creatures: Vec<Creature>,
    pub advent: Vec<AdventNight>,
}

impl Vault {
    /// Loads the built-in packs. The data is compiled in and checked by the
    /// tests, so a failure here is a bug, not a player's mistake.
    pub fn load() -> Self {
        let mut packs = Vec::new();
        let mut creatures = Vec::new();
        for pack_id in PACK_ORDER {
            let dir = PACKS_DIR
                .get_dir(pack_id)
                .unwrap_or_else(|| panic!("missing pack directory {pack_id}"));
            let pack: Pack = parse(dir, &format!("{pack_id}/pack.json"));
            for id in &pack.order {
                let mut c: Creature = parse(dir, &format!("{pack_id}/{id}.json"));
                c.pack = pack.id.clone();
                creatures.push(c);
            }
            packs.push(pack);
        }
        let advent: AdventFile = serde_json::from_str(
            PACKS_DIR
                .get_file("advent.json")
                .and_then(|f| f.contents_utf8())
                .expect("missing packs/advent.json"),
        )
        .expect("invalid packs/advent.json");
        Self {
            packs,
            creatures,
            advent: advent.nights,
        }
    }

    /// Keeps only the creatures in the given packs.
    pub fn only_packs(mut self, ids: &[String]) -> Result<Self> {
        if ids.is_empty() {
            return Ok(self);
        }
        for id in ids {
            if !self.packs.iter().any(|p| &p.id == id) {
                return Err(
                    TinselError::new(Code::BadPack, format!("no pack called {id:?}"))
                        .with_hint(format!("choose from: {}", PACK_ORDER.join(", "))),
                );
            }
        }
        self.packs.retain(|p| ids.contains(&p.id));
        self.creatures.retain(|c| ids.contains(&c.pack));
        Ok(self)
    }

    /// A creature by its exact id.
    pub fn get(&self, id: &str) -> Option<&Creature> {
        self.creatures.iter().find(|c| c.id == id)
    }

    /// The creatures grouped by pack, in pack order.
    pub fn by_pack(&self) -> Vec<(&Pack, Vec<&Creature>)> {
        let mut groups: BTreeMap<usize, Vec<&Creature>> = BTreeMap::new();
        for c in &self.creatures {
            if let Some(i) = self.packs.iter().position(|p| p.id == c.pack) {
                groups.entry(i).or_default().push(c);
            }
        }
        groups
            .into_iter()
            .map(|(i, cs)| (&self.packs[i], cs))
            .collect()
    }

    /// Finds a creature by id, name, alias or part of one. Returns the
    /// creature, or the candidates if the query matches several.
    pub fn find(&self, query: &str) -> Result<&Creature> {
        let key = normalize(query);
        if key.is_empty() {
            return Err(unknown(query, None));
        }
        let names = |c: &Creature| {
            let mut v = vec![normalize(&c.id), normalize(&c.name)];
            v.extend(c.aliases.iter().map(|a| normalize(a)));
            v
        };
        // An exact match on any name wins outright.
        if let Some(c) = self.creatures.iter().find(|c| names(c).contains(&key)) {
            return Ok(c);
        }
        // Otherwise, every word of the query must appear in one of its names,
        // so "ghost future" finds the Ghost of Christmas Future.
        let words: Vec<&str> = key.split(' ').collect();
        let matches: Vec<&Creature> = self
            .creatures
            .iter()
            .filter(|c| names(c).iter().any(|n| words.iter().all(|w| n.contains(w))))
            .collect();
        match matches.as_slice() {
            [one] => Ok(one),
            [] => Err(unknown(query, self.suggest(&key))),
            many => {
                let names: Vec<&str> = many.iter().map(|c| c.name.as_str()).collect();
                Err(TinselError::new(
                    Code::AmbiguousCreature,
                    format!("{query:?} could be any of: {}", names.join(", ")),
                ))
            }
        }
    }

    /// The closest creature name to a query that matched nothing.
    fn suggest(&self, key: &str) -> Option<&str> {
        self.creatures
            .iter()
            .flat_map(|c| {
                std::iter::once(&c.name)
                    .chain(&c.aliases)
                    .map(move |n| (levenshtein(&normalize(n), key), c.name.as_str()))
            })
            .filter(|(d, _)| *d <= 3.max(key.len() / 3))
            .min_by_key(|(d, _)| *d)
            .map(|(_, name)| name)
    }
}

fn parse<T: serde::de::DeserializeOwned>(dir: &Dir, path: &str) -> T {
    let text = dir
        .get_file(path)
        .and_then(|f| f.contents_utf8())
        .unwrap_or_else(|| panic!("missing packs/{path}"));
    serde_json::from_str(text).unwrap_or_else(|e| panic!("invalid packs/{path}: {e}"))
}

fn unknown(query: &str, suggestion: Option<&str>) -> TinselError {
    let mut msg = format!("no creature called {query:?} is out tonight");
    if let Some(s) = suggestion {
        msg.push_str(&format!(". Did you mean {s}?"));
    }
    TinselError::new(Code::UnknownCreature, msg)
        .with_hint("try 'terminal-of-tinsel list' for tonight's guests")
}

/// Lowercases, folds common accents, drops a leading "the" and punctuation,
/// so "the Yule-Cat" and "yule cat" match, and so do "Grýla" and "gryla".
pub fn normalize(s: &str) -> String {
    let folded: String = s
        .trim()
        .to_lowercase()
        .chars()
        .map(|ch| match ch {
            'á' | 'à' | 'â' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ý' | 'ÿ' => 'y',
            'ó' | 'ò' | 'ô' | 'ö' | 'ø' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ð' => 'd',
            'þ' => 't',
            'æ' => 'a',
            c => c,
        })
        .collect();
    let folded = folded.strip_prefix("the ").unwrap_or(&folded);
    let cleaned: String = folded
        .chars()
        .filter_map(|ch| match ch {
            'a'..='z' | '0'..='9' => Some(ch),
            ' ' | '-' | '_' | '\'' => Some(' '),
            _ => None,
        })
        .collect();
    cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Edit distance, for "did you mean".
fn levenshtein(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut cur = vec![i + 1; b.len() + 1];
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != *cb);
            cur[j + 1] = (prev[j] + cost).min(prev[j + 1] + 1).min(cur[j] + 1);
        }
        prev = cur;
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vault() -> Vault {
        Vault::load()
    }

    #[test]
    fn every_pack_directory_is_in_the_pack_order() {
        for dir in PACKS_DIR.dirs() {
            let name = dir.path().to_str().unwrap();
            assert!(
                PACK_ORDER.contains(&name),
                "pack {name} isn't in PACK_ORDER"
            );
        }
    }

    #[test]
    fn every_creature_file_is_listed_in_its_pack() {
        let v = vault();
        for dir in PACKS_DIR.dirs() {
            for f in dir.files() {
                let stem = f.path().file_stem().unwrap().to_str().unwrap();
                if stem != "pack" {
                    assert!(v.get(stem).is_some(), "{stem} isn't in its pack's order");
                }
            }
        }
    }

    #[test]
    fn every_creature_is_complete() {
        for c in vault().creatures {
            let id = &c.id;
            assert!(
                !c.name.is_empty() && !c.emoji.is_empty(),
                "{id}: name and emoji"
            );
            assert!(c.legend.len() > 100, "{id}: the legend needs telling");
            assert!(c.facts.len() >= 5, "{id}: needs 5+ facts");
            assert!(c.myths.len() >= 3, "{id}: needs 3+ myths");
            assert!(
                !c.powers.is_empty() && !c.weaknesses.is_empty(),
                "{id}: powers and weaknesses"
            );
            for (label, v) in c.stats.labelled() {
                assert!((1..=10).contains(&v), "{id}: {label} must be 1-10");
            }
            for hex in [&c.theme.primary, &c.theme.accent] {
                assert!(
                    crate::render::parse_hex(hex).is_some(),
                    "{id}: bad colour {hex}"
                );
            }
            // Emoji with a variation selector have no agreed width and break boxes.
            assert!(
                !c.emoji.contains('\u{FE0F}'),
                "{id}: emoji needs a variation selector"
            );
            assert_eq!(
                unicode_width::UnicodeWidthStr::width(c.emoji.as_str()),
                2,
                "{id}: emoji width"
            );
        }
    }

    #[test]
    fn ids_are_unique() {
        let v = vault();
        let mut ids: Vec<_> = v.creatures.iter().map(|c| &c.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), v.creatures.len());
    }

    #[test]
    fn the_advent_calendar_has_24_doors_of_real_creatures() {
        let v = vault();
        let nights: Vec<u8> = v.advent.iter().map(|n| n.night).collect();
        assert_eq!(nights, (1..=24).collect::<Vec<_>>());
        for n in &v.advent {
            assert!(
                v.get(&n.id).is_some(),
                "night {}: unknown creature {}",
                n.night,
                n.id
            );
        }
    }

    #[test]
    fn find_by_name_alias_partial_and_accent() {
        let v = vault();
        assert_eq!(v.find("Krampus").unwrap().id, "krampus");
        assert_eq!(v.find("the yule cat").unwrap().id, "yule-cat");
        assert_eq!(v.find("gryla").unwrap().id, "gryla");
        assert_eq!(v.find("Grýla").unwrap().id, "gryla");
        assert_eq!(v.find("jacob marley").unwrap().id, "marley");
        assert_eq!(v.find("nisse").unwrap().id, "tomte");
        assert_eq!(v.find("caga").unwrap().id, "tio-de-nadal");
        assert_eq!(v.find("pere fouettard").unwrap().id, "pere-fouettard");
        assert_eq!(v.find("ghost future").unwrap().id, "ghost-yet-to-come");
        assert_eq!(v.find("lads").unwrap().id, "yule-lads");
    }

    #[test]
    fn ambiguous_and_unknown_names_explain_themselves() {
        let v = vault();
        let e = v.find("yule").unwrap_err();
        assert_eq!(e.code, Code::AmbiguousCreature);
        let e = v.find("krampas").unwrap_err();
        assert_eq!(e.code, Code::UnknownCreature);
        assert!(e.message.contains("Did you mean Krampus?"), "{}", e.message);
        assert_eq!(v.find("rudolph").unwrap_err().code, Code::UnknownCreature);
    }

    #[test]
    fn pack_filter_keeps_only_those_packs() {
        let v = vault().only_packs(&["nordic".into()]).unwrap();
        assert!(v.creatures.iter().all(|c| c.pack == "nordic"));
        assert_eq!(v.creatures.len(), 4);
        let e = vault().only_packs(&["elves".into()]).unwrap_err();
        assert_eq!(e.code, Code::BadPack);
    }

    #[test]
    fn normalize_folds_accents_and_the() {
        assert_eq!(normalize("The Yule-Cat"), "yule cat");
        assert_eq!(normalize("Tió de Nadal"), "tio de nadal");
        assert_eq!(normalize("Jólakötturinn"), "jolakotturinn");
        assert_eq!(normalize("Père Fouettard"), "pere fouettard");
    }
}
