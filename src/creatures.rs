//! The creatures of Christmas folklore, their packs, and finding them by name.
//!
//! The built-in data lives in `packs/` as JSON and is compiled into the
//! binary, so the CLI needs nothing on disk to run. Community packs are the
//! same files in the player's config directory, read by the same code.
//!
//! Every field has a default, so a community creature can be three lines
//! long. The data tests hold the built-in creatures to the full set.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use include_dir::{Dir, include_dir};
use serde::{Deserialize, Serialize};

use crate::error::{Code, Result, TinselError};

static PACKS_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/packs");

/// The order packs appear in, from the saint's companions to Dickens.
pub const PACK_ORDER: &[&str] = &["companions", "nordic", "midwinter", "dickens"];

/// A single creature and everything we know about it.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
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
    /// The portrait, from `<id>.txt` beside the creature's JSON file.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub ascii: String,
    /// Filled in from the directory the creature was loaded from.
    #[serde(default)]
    pub pack: String,
}

impl Creature {
    /// Where and when the creature first appeared, as a readable line.
    pub fn first_appearance(&self) -> String {
        let d = &self.debut;
        match (&d.creator, d.year, &d.era) {
            (Some(who), Some(year), _) => format!("{}, {} ({year})", d.title, who),
            (_, _, Some(era)) => era.clone(),
            (_, Some(year), _) => year.to_string(),
            _ => d.title.clone(),
        }
    }
}

/// Where a creature first appeared.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
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
    /// When the film opened, as YYYY-MM-DD, for anniversaries.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release_date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub studio: Option<String>,
    pub director: String,
    /// Who played the creature.
    pub star: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub makeup: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub silent: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl Film {
    /// The day the film opened, if the data gives one.
    pub fn released(&self) -> Option<jiff::civil::Date> {
        self.release_date.as_deref()?.parse().ok()
    }
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
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(default)]
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Theme {
    pub primary: String,
    pub accent: String,
}

/// A pack of creatures, such as the Nordic Yule.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Pack {
    pub id: String,
    pub name: String,
    pub description: String,
    pub order: Vec<String>,
    /// Where the pack was loaded from: "built-in", or a directory.
    #[serde(skip)]
    pub source: String,
    /// How the explorer draws the pack's pages. "engraving" gives sepia ink
    /// and drifting snow, like the plates of a Victorian Christmas book.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
}

impl Pack {
    /// Whether the explorer draws this pack as an engraving.
    pub fn is_engraving(&self) -> bool {
        self.style.as_deref() == Some("engraving")
    }
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
            let (pack, members, problems) = read_pack(&Embedded(dir), pack_id, BUILT_IN);
            assert!(problems.is_empty(), "packs/{pack_id}: {problems:?}");
            creatures.extend(members);
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

    /// Adds the community packs in `dir`, where each subdirectory is a pack.
    /// Anything broken, incomplete or already taken is skipped and reported;
    /// a missing `dir` simply means no community packs.
    pub fn add_community(&mut self, dir: &Path) -> Vec<String> {
        let mut warnings = Vec::new();
        let mut names: Vec<String> = match std::fs::read_dir(dir) {
            Ok(entries) => entries
                .flatten()
                .filter(|e| e.path().is_dir())
                .filter_map(|e| e.file_name().into_string().ok())
                .collect(),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return warnings,
            Err(e) => return vec![format!("couldn't read {}: {e}", dir.display())],
        };
        names.sort();
        for name in names {
            let path = dir.join(&name);
            let source = path.display().to_string();
            let (pack, members, problems) = read_pack(&OnDisk(path), &name, &source);
            warnings.extend(problems.iter().map(|p| format!("pack {name:?}: {p}")));
            if self.packs.iter().any(|p| p.id == pack.id) {
                warnings.push(format!(
                    "pack {:?}: a pack with that id already exists",
                    pack.id
                ));
                continue;
            }
            let before = self.creatures.len();
            for mut c in members {
                let problems = validate(&c);
                if !problems.is_empty() {
                    warnings.push(format!(
                        "pack {:?}: skipping {:?}: {}",
                        pack.id,
                        c.id,
                        problems.join("; ")
                    ));
                } else if self.get(&c.id).is_some() {
                    warnings.push(format!(
                        "pack {:?}: skipping {:?}: a creature with that id already exists",
                        pack.id, c.id
                    ));
                } else {
                    if !is_plain_art(&c.ascii) {
                        warnings.push(format!(
                            "pack {:?}: {:?}: the portrait must be printable ASCII, so it's left out",
                            pack.id, c.id
                        ));
                        c.ascii.clear();
                    }
                    apply_defaults(&mut c);
                    self.creatures.push(c);
                }
            }
            // A pack with nothing playable in it isn't listed.
            if self.creatures.len() > before {
                self.packs.push(pack);
            }
        }
        warnings
    }

    /// Keeps only the creatures in the given packs.
    pub fn only_packs(mut self, ids: &[String]) -> Result<Self> {
        if ids.is_empty() {
            return Ok(self);
        }
        let known: Vec<&str> = self.packs.iter().map(|p| p.id.as_str()).collect();
        for id in ids {
            if !known.contains(&id.as_str()) {
                let nearest = known
                    .iter()
                    .map(|k| (levenshtein(k, &id.to_lowercase()), *k))
                    .filter(|(distance, _)| *distance <= 2)
                    .min();
                let message = match nearest {
                    Some((_, k)) => format!("no pack called {id:?}. Did you mean {k}?"),
                    None => format!("no pack called {id:?}"),
                };
                return Err(TinselError::new(Code::BadPack, message)
                    .with_hint(format!("choose from: {}", known.join(", "))));
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

    /// The pack a creature belongs to.
    pub fn pack_of(&self, c: &Creature) -> Option<&Pack> {
        self.packs.iter().find(|p| p.id == c.pack)
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
            return Err(unknown(query, &[]));
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
            [] => Err(unknown(query, &self.suggest(&key))),
            many => {
                let names: Vec<&str> = many.iter().map(|c| c.name.as_str()).collect();
                Err(TinselError::new(
                    Code::AmbiguousCreature,
                    format!("{query:?} could be any of: {}", names.join(", ")),
                ))
            }
        }
    }

    /// Up to three creature names within a few edits of a query that matched
    /// nothing, nearest first.
    fn suggest(&self, key: &str) -> Vec<&str> {
        let mut scored: Vec<(usize, &str)> = self
            .creatures
            .iter()
            .filter_map(|c| {
                std::iter::once(&c.id)
                    .chain(std::iter::once(&c.name))
                    .chain(&c.aliases)
                    .map(|n| levenshtein(&normalize(n), key))
                    .min()
                    .map(|d| (d, c.name.as_str()))
            })
            .filter(|(d, _)| *d <= 3.max(key.len() / 3))
            .collect();
        scored.sort_by_key(|(d, _)| *d);
        scored.into_iter().take(3).map(|(_, name)| name).collect()
    }
}

/// What a pack's `source` says when it was compiled in.
pub const BUILT_IN: &str = "built-in";

/// Where a pack's files come from: compiled into the binary, or a directory
/// on disk. Go has `fs.FS` for this; here it's a two-method trait.
trait PackFiles {
    /// The text of a file in the pack, if it's there and it's UTF-8.
    fn read(&self, name: &str) -> Option<String>;
    /// The names of the pack's `.json` files, without the extension, sorted.
    fn json_stems(&self) -> Vec<String>;
}

struct Embedded<'a>(&'a Dir<'a>);

impl PackFiles for Embedded<'_> {
    fn read(&self, name: &str) -> Option<String> {
        let file = self.0.get_file(self.0.path().join(name))?;
        file.contents_utf8().map(str::to_string)
    }

    fn json_stems(&self) -> Vec<String> {
        let mut stems: Vec<String> = self.0.files().filter_map(|f| json_stem(f.path())).collect();
        stems.sort();
        stems
    }
}

struct OnDisk(PathBuf);

impl PackFiles for OnDisk {
    fn read(&self, name: &str) -> Option<String> {
        std::fs::read_to_string(self.0.join(name)).ok()
    }

    fn json_stems(&self) -> Vec<String> {
        let mut stems: Vec<String> = std::fs::read_dir(&self.0)
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| json_stem(&e.path()))
            .collect();
        stems.sort();
        stems
    }
}

fn json_stem(path: &Path) -> Option<String> {
    if path.extension()? != "json" {
        return None;
    }
    path.file_stem()?.to_str().map(str::to_string)
}

/// Reads one pack: its `pack.json` (optional: a directory of creatures is
/// pack enough), one `<id>.json` per creature and an optional `<id>.txt`
/// portrait. Files that can't be read are skipped and reported, so one bad
/// creature doesn't sink the pack. Creatures come back in the pack's `order`,
/// then any others by id.
fn read_pack(files: &dyn PackFiles, dir: &str, source: &str) -> (Pack, Vec<Creature>, Vec<String>) {
    let mut problems = Vec::new();
    let mut pack = match files.read("pack.json") {
        None => Pack {
            name: dir.to_string(),
            ..Pack::default()
        },
        Some(text) => match serde_json::from_str::<Pack>(&text) {
            Ok(pack) => pack,
            Err(e) => return (Pack::default(), Vec::new(), vec![format!("pack.json: {e}")]),
        },
    };
    if pack.id.is_empty() {
        pack.id = dir.to_string();
    }
    if pack.name.is_empty() {
        pack.name = pack.id.clone();
    }
    pack.source = source.to_string();

    let mut found: BTreeMap<String, Creature> = BTreeMap::new();
    // Which file each id came from, to name it if another file claims it too.
    let mut claimed: BTreeMap<String, String> = BTreeMap::new();
    for stem in files.json_stems().into_iter().filter(|s| s != "pack") {
        let parsed = files
            .read(&format!("{stem}.json"))
            .ok_or_else(|| "isn't readable text".to_string())
            .and_then(|text| serde_json::from_str::<Creature>(&text).map_err(|e| e.to_string()));
        match parsed {
            Ok(mut c) => {
                if c.id.is_empty() {
                    c.id = stem.clone();
                }
                c.pack = pack.id.clone();
                c.ascii = files
                    .read(&format!("{}.txt", c.id))
                    .map(|art| tidy_art(&art))
                    .unwrap_or_default();
                // Two files can declare the same id. The first keeps it:
                // quietly replacing it could lose a good creature to a bad one.
                if let Some(first) = claimed.get(&c.id) {
                    problems.push(format!(
                        "{stem}.json: the id {:?} is already used by {first}.json, so this file is skipped",
                        c.id
                    ));
                    continue;
                }
                claimed.insert(c.id.clone(), stem.clone());
                found.insert(c.id.clone(), c);
            }
            Err(e) => problems.push(format!("{stem}.json: {e}")),
        }
    }
    let mut creatures = Vec::new();
    for id in &pack.order {
        match found.remove(id) {
            Some(c) => creatures.push(c),
            None => problems.push(format!("pack.json lists {id:?}, but there's no {id}.json")),
        }
    }
    creatures.extend(found.into_values());
    (pack, creatures, problems)
}

/// A portrait without its trailing newline and with Windows line endings
/// normalised: a stray `\r` would garble the layout.
fn tidy_art(art: &str) -> String {
    art.replace("\r\n", "\n")
        .replace('\r', "")
        .trim_end_matches('\n')
        .to_string()
}

/// Whether a portrait is printable ASCII only. Anything else risks a
/// double-width cell, which would misalign it.
fn is_plain_art(art: &str) -> bool {
    art.chars()
        .all(|ch| ch == '\n' || (' '..='~').contains(&ch))
}

/// What a community creature is missing to be playable.
fn validate(c: &Creature) -> Vec<&'static str> {
    let mut problems = Vec::new();
    if c.name.trim().is_empty() {
        problems.push("needs a name");
    }
    if c.description.trim().is_empty() {
        problems.push("needs a description");
    }
    if c.facts.is_empty() {
        problems.push("needs at least one fact");
    }
    if c.stats.labelled().iter().any(|(_, v)| *v > 10) {
        problems.push("stats must be between 0 and 10");
    }
    problems
}

/// Fills in the optional fields, so a community creature works everywhere.
fn apply_defaults(c: &mut Creature) {
    if c.emoji.is_empty() {
        c.emoji = "🎁".to_string();
    }
    if c.origin.is_empty() {
        c.origin = "Unknown".to_string();
    }
    if c.debut.title.is_empty() {
        c.debut.title = c.name.clone();
    }
    if c.debut.year.is_none() && c.debut.era.is_none() {
        c.debut.era = Some("date unknown".to_string());
    }
    for stat in [
        &mut c.stats.strength,
        &mut c.stats.speed,
        &mut c.stats.cunning,
        &mut c.stats.dread,
    ] {
        if *stat == 0 {
            *stat = 5;
        }
    }
    if c.powers.is_empty() {
        c.powers = vec!["Sheer festive spirit".to_string()];
    }
    if c.weaknesses.is_empty() {
        c.weaknesses = vec!["An early night".to_string()];
    }
    if crate::render::parse_hex(&c.theme.primary).is_none() {
        c.theme.primary = crate::render::GOLD.to_string();
    }
    if crate::render::parse_hex(&c.theme.accent).is_none() {
        c.theme.accent = crate::render::GREEN.to_string();
    }
}

fn unknown(query: &str, suggestions: &[&str]) -> TinselError {
    let mut msg = format!("no creature called {query:?} is out tonight");
    match suggestions {
        [] => {}
        [one] => msg.push_str(&format!(". Did you mean {one}?")),
        [first @ .., last] => {
            msg.push_str(&format!(". Did you mean {} or {last}?", first.join(", ")));
        }
    }
    TinselError::new(Code::UnknownCreature, msg)
        .with_hint("try 'terminal-of-tinsel list' for tonight's guests")
}

/// Lowercases, folds common accents, drops a leading "the" and punctuation,
/// so "the Yule-Cat" and "yule cat" match, and so do "Grýla" and "gryla".
pub fn normalize(s: &str) -> String {
    let folded: String = s.trim().chars().map(fold).collect();
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

/// Lowercases one character and folds common accents, always to exactly one
/// character, so positions in folded text line up with the original.
pub fn fold(ch: char) -> char {
    match ch.to_lowercase().next().unwrap_or(ch) {
        'á' | 'à' | 'â' | 'ä' => 'a',
        'é' | 'è' | 'ê' | 'ë' => 'e',
        'í' | 'ì' | 'î' | 'ï' => 'i',
        'ý' | 'ÿ' => 'y',
        'ó' | 'ò' | 'ô' | 'ö' | 'ø' => 'o',
        'ú' | 'ù' | 'û' | 'ü' => 'u',
        'ð' => 'd',
        'þ' => 't',
        'æ' => 'a',
        'č' => 'c',
        c => c,
    }
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
            // Every field has a default for community packs, so the built-in
            // creatures are held to the full set here.
            assert!(
                !c.description.is_empty() && !c.origin.is_empty() && !c.host_intro.is_empty(),
                "{id}: description, origin and hostIntro"
            );
            assert!(
                !c.debut.medium.is_empty()
                    && !c.debut.title.is_empty()
                    && (c.debut.year.is_some() || c.debut.era.is_some()),
                "{id}: debut needs a medium, a title and a year or an era"
            );
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
    fn every_creature_has_a_portrait_of_plain_characters() {
        for c in vault().creatures {
            let id = &c.id;
            assert!(!c.ascii.is_empty(), "{id}: needs a portrait in {id}.txt");
            // Anything but printable ASCII risks a double-width cell, which
            // would misalign the portrait and the frost in the guessing game.
            assert!(
                c.ascii
                    .chars()
                    .all(|ch| ch == '\n' || (' '..='~').contains(&ch)),
                "{id}: portraits use printable ASCII only"
            );
            let lines: Vec<&str> = c.ascii.lines().collect();
            assert!(lines.len() <= 20, "{id}: portrait is over 20 lines");
            assert!(
                lines.iter().all(|l| l.len() <= 46 && !l.ends_with(' ')),
                "{id}: portrait lines are at most 46 columns, with no trailing spaces"
            );
        }
    }

    #[test]
    fn only_dickens_is_an_engraving() {
        let v = vault();
        let styled: Vec<&str> = v
            .packs
            .iter()
            .filter(|p| p.is_engraving())
            .map(|p| p.id.as_str())
            .collect();
        assert_eq!(styled, ["dickens"]);
    }

    #[test]
    fn first_appearance_reads_naturally() {
        let v = vault();
        assert_eq!(
            v.get("marley").unwrap().first_appearance(),
            "A Christmas Carol, Charles Dickens (1843)"
        );
        assert!(!v.get("gryla").unwrap().first_appearance().is_empty());
    }

    #[test]
    fn film_credits_are_complete_and_dated_consistently() {
        for c in vault().creatures {
            let Some(f) = &c.film else { continue };
            let id = &c.id;
            assert!(
                !f.title.is_empty() && !f.director.is_empty() && !f.star.is_empty(),
                "{id}: a film needs a title, a director and a star"
            );
            assert!(
                (1888..=2100).contains(&f.year),
                "{id}: film year {}",
                f.year
            );
            if let Some(date) = &f.release_date {
                let released = f.released();
                assert!(
                    released.is_some(),
                    "{id}: releaseDate {date} isn't YYYY-MM-DD"
                );
                assert_eq!(
                    i32::from(released.unwrap().year()),
                    f.year,
                    "{id}: releaseDate and year disagree"
                );
            }
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
        // Several near misses are all offered, nearest first.
        assert_eq!(
            unknown("x", &["A", "B", "C"])
                .message
                .matches("A, B or C")
                .count(),
            1
        );
    }

    #[test]
    fn every_built_in_pack_is_described_and_marked() {
        let v = vault();
        for p in &v.packs {
            assert!(!p.name.is_empty() && !p.description.is_empty(), "{}", p.id);
            assert_eq!(p.source, BUILT_IN);
            let members = v.creatures.iter().filter(|c| c.pack == p.id).count();
            assert_eq!(members, p.order.len(), "{}: order and files agree", p.id);
        }
    }

    /// A scratch directory of community packs, removed when the test ends.
    struct Community(PathBuf);

    impl Community {
        fn new(name: &str) -> Self {
            let dir =
                std::env::temp_dir().join(format!("tinsel-packs-{name}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            Self(dir)
        }

        fn write(&self, file: &str, text: &str) {
            let path = self.0.join(file);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
    }

    impl Drop for Community {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    const ELF: &str = r#"{"name": "The Shelf Inspector", "description": "Checks for dust", "facts": ["It never blinks"]}"#;

    #[test]
    fn no_community_directory_means_no_community_packs() {
        let dir = Community::new("none");
        let mut v = vault();
        assert!(v.add_community(&dir.0).is_empty());
        assert_eq!(v.packs.len(), PACK_ORDER.len());
    }

    #[test]
    fn a_three_line_creature_is_enough_and_gets_defaults() {
        let dir = Community::new("minimal");
        dir.write("attic/shelf-inspector.json", ELF);
        dir.write("attic/shelf-inspector.txt", "  o\r\n /|\\\r\n\r\n");
        let mut v = vault();
        let warnings = v.add_community(&dir.0);
        assert!(warnings.is_empty(), "{warnings:?}");
        let pack = v.packs.last().unwrap();
        assert_eq!((pack.id.as_str(), pack.name.as_str()), ("attic", "attic"));
        assert!(pack.source.ends_with("attic"), "{}", pack.source);
        let c = v.get("shelf-inspector").unwrap();
        assert_eq!(c.pack, "attic");
        assert_eq!(c.emoji, "🎁");
        assert_eq!(c.stats.labelled().map(|(_, v)| v), [5, 5, 5, 5]);
        assert_eq!(c.first_appearance(), "date unknown");
        assert!(!c.powers.is_empty() && !c.weaknesses.is_empty());
        assert!(crate::render::parse_hex(&c.theme.primary).is_some());
        assert_eq!(c.ascii, "  o\n /|\\");
        // It joins everything: lookup, the pack filter, the games.
        assert_eq!(v.find("shelf inspector").unwrap().id, "shelf-inspector");
        let only = v.only_packs(&["attic".into()]).unwrap();
        assert_eq!(only.creatures.len(), 1);
    }

    #[test]
    fn pack_json_names_and_orders_a_community_pack() {
        let dir = Community::new("ordered");
        dir.write(
            "attic/pack.json",
            r#"{"id": "loft", "name": "The Loft", "description": "Up the ladder", "order": ["b", "a", "ghost"]}"#,
        );
        for id in ["a", "b", "c"] {
            dir.write(
                &format!("attic/{id}.json"),
                &ELF.replace("The Shelf Inspector", id),
            );
        }
        let mut v = vault();
        let warnings = v.add_community(&dir.0);
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(warnings[0].contains("lists \"ghost\""), "{warnings:?}");
        assert_eq!(v.packs.last().unwrap().name, "The Loft");
        let ids: Vec<&str> = v
            .creatures
            .iter()
            .filter(|c| c.pack == "loft")
            .map(|c| c.id.as_str())
            .collect();
        assert_eq!(ids, ["b", "a", "c"], "the order first, then the rest by id");
    }

    #[test]
    fn broken_and_incomplete_creatures_are_skipped_with_a_warning() {
        let dir = Community::new("broken");
        dir.write("attic/good.json", ELF);
        dir.write("attic/typo.json", "{ not json");
        dir.write("attic/empty.json", "{}");
        dir.write(
            "attic/strong.json",
            &ELF.replace('}', r#", "stats": {"strength": 11}}"#),
        );
        dir.write("attic/krampus.json", ELF);
        dir.write(
            "attic/emoji.json",
            &ELF.replace("The Shelf Inspector", "Emoji Art"),
        );
        dir.write("attic/emoji.txt", "🎄🎄🎄");
        dir.write("nordic/extra.json", ELF);
        dir.write("hollow/pack.json", r#"{"name": "Nothing here"}"#);
        dir.write("notes.txt", "not a pack");
        let mut v = vault();
        let warnings = v.add_community(&dir.0).join("\n");
        for expected in [
            "typo.json:",
            "skipping \"empty\": needs a name; needs a description; needs at least one fact",
            "skipping \"strong\": stats must be between 0 and 10",
            "skipping \"krampus\": a creature with that id already exists",
            "\"emoji\": the portrait must be printable ASCII",
            "pack \"nordic\": a pack with that id already exists",
        ] {
            assert!(
                warnings.contains(expected),
                "missing {expected:?} in:\n{warnings}"
            );
        }
        let attic: Vec<&str> = v
            .creatures
            .iter()
            .filter(|c| c.pack == "attic")
            .map(|c| c.id.as_str())
            .collect();
        assert_eq!(attic, ["emoji", "good"]);
        assert!(v.get("emoji").unwrap().ascii.is_empty());
        assert_eq!(v.get("krampus").unwrap().pack, "companions");
        assert!(v.get("extra").is_none());
        assert!(
            !v.packs.iter().any(|p| p.id == "hollow"),
            "an empty pack isn't listed"
        );
        assert_eq!(v.creatures.iter().filter(|c| c.pack == "nordic").count(), 4);
    }

    #[test]
    fn two_files_with_one_id_keep_the_first_and_warn() {
        let dir = Community::new("twins");
        dir.write("attic/a.json", &ELF.replace('{', r#"{"id": "guest", "#));
        // The second is incomplete as well: it must not cost us the first.
        dir.write("attic/b.json", r#"{"id": "guest", "name": "Impostor"}"#);
        let mut v = vault();
        let warnings = v.add_community(&dir.0);
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(
            warnings[0].contains("b.json: the id \"guest\" is already used by a.json"),
            "{warnings:?}"
        );
        assert_eq!(v.get("guest").unwrap().name, "The Shelf Inspector");
    }

    #[test]
    fn a_malformed_colour_falls_back_instead_of_panicking() {
        let dir = Community::new("colours");
        for (id, colour) in [
            ("accent", "#aé123"),
            ("short", "#fff"),
            ("word", "tinsel"),
            ("wide", "#ééé"),
        ] {
            let theme = format!(r#", "theme": {{"primary": "{colour}", "accent": "{colour}"}}}}"#);
            dir.write(
                &format!("attic/{id}.json"),
                &ELF.replace("The Shelf Inspector", id).replace('}', &theme),
            );
        }
        let mut v = vault();
        let warnings = v.add_community(&dir.0);
        assert!(warnings.is_empty(), "{warnings:?}");
        for id in ["accent", "short", "word", "wide"] {
            let c = v.get(id).unwrap();
            assert_eq!(c.theme.primary, crate::render::GOLD, "{id}");
            assert_eq!(c.theme.accent, crate::render::GREEN, "{id}");
        }
    }

    #[test]
    fn an_unknown_pack_gets_a_suggestion_and_the_list() {
        let e = vault().only_packs(&["nordik".into()]).unwrap_err();
        assert_eq!(e.code, Code::BadPack);
        assert!(e.message.contains("Did you mean nordic?"), "{}", e.message);
        assert_eq!(
            e.hint.as_deref(),
            Some("choose from: companions, nordic, midwinter, dickens")
        );
        let e = vault().only_packs(&["elves".into()]).unwrap_err();
        assert!(!e.message.contains("Did you mean"), "{}", e.message);
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
