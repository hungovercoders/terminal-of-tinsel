//! Saving a player's progress (what's in the stocking, the badges and the
//! records) as JSON in their config directory.

use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Code, Result, TinselError};

/// Overrides where progress and community packs are kept.
pub const HOME_ENV: &str = "TERMINAL_OF_TINSEL_HOME";

const FILE: &str = "stocking.json";

/// Everything we remember about a player.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Progress {
    pub version: u32,
    /// Correct answers per creature id.
    pub knowledge: BTreeMap<String, u32>,
    /// Creature id to when it went in the stocking.
    pub captured: BTreeMap<String, String>,
    /// Badge id to when it was earned.
    pub badges: BTreeMap<String, String>,
    /// Creature pages visited in the explorer.
    pub seen: BTreeMap<String, bool>,

    pub quizzes_played: u32,
    /// Percent.
    pub best_quiz_score: u32,
    pub guesses_played: u32,
    /// Points.
    pub best_guess_score: u32,
    pub mashes_played: u32,
}

/// The app's config directory: `$TERMINAL_OF_TINSEL_HOME` if set, otherwise
/// the platform's config directory for terminal-of-tinsel.
pub fn dir() -> Result<PathBuf> {
    if let Some(home) = std::env::var_os(HOME_ENV).filter(|h| !h.is_empty()) {
        return Ok(PathBuf::from(home));
    }
    directories::ProjectDirs::from("", "", "terminal-of-tinsel")
        .map(|dirs| dirs.config_dir().to_path_buf())
        .ok_or_else(|| {
            TinselError::new(
                Code::Store,
                "couldn't work out where your home directory is",
            )
            .with_hint(format!("set {HOME_ENV} to a directory to keep progress in"))
        })
}

/// Where progress is saved, in a config directory.
pub fn path_in(dir: &Path) -> PathBuf {
    dir.join(FILE)
}

/// Reads saved progress, or empty progress if there is none yet.
pub fn load(dir: &Path) -> Result<Progress> {
    let path = path_in(dir);
    let mut progress = match fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str(&text).map_err(|e| {
            store_error(&path, "read", e).with_hint(
                "fix the file, or run 'terminal-of-tinsel stocking reset' to start again",
            )
        })?,
        Err(e) if e.kind() == io::ErrorKind::NotFound => Progress::default(),
        Err(e) => return Err(store_error(&path, "read", e)),
    };
    progress.version = 1;
    Ok(progress)
}

/// Writes progress, creating the directory if needed. The file is written
/// beside the real one and renamed over it, so a crash can't leave half a
/// stocking behind.
pub fn save(dir: &Path, progress: &Progress) -> Result<()> {
    let path = path_in(dir);
    let failed = |e: io::Error| store_error(&path, "save", e);
    fs::create_dir_all(dir).map_err(failed)?;
    let text = serde_json::to_string_pretty(progress).map_err(|e| store_error(&path, "save", e))?;
    let temp = dir.join(format!("{FILE}.{}.tmp", std::process::id()));
    let written = fs::File::create(&temp)
        .and_then(|mut f| {
            f.write_all(text.as_bytes())?;
            f.write_all(b"\n")?;
            f.sync_all()
        })
        .and_then(|()| fs::rename(&temp, &path));
    if written.is_err() {
        let _ = fs::remove_file(&temp);
    }
    written.map_err(failed)
}

/// Deletes saved progress. Having none already is not an error.
pub fn reset(dir: &Path) -> Result<()> {
    let path = path_in(dir);
    match fs::remove_file(&path) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(store_error(&path, "reset", e)),
        _ => Ok(()),
    }
}

fn store_error(path: &Path, verb: &str, e: impl std::fmt::Display) -> TinselError {
    TinselError::new(
        Code::Store,
        format!("couldn't {verb} your progress in {}: {e}", path.display()),
    )
}

/// Shortens a path under the home directory to `~/...`, the way people
/// usually write it. Windows paths are left alone: cmd.exe doesn't expand
/// `~`, so a shortened path couldn't be copied and used there.
pub fn tilde(path: &Path) -> String {
    let full = path.display().to_string();
    if cfg!(windows) {
        return full;
    }
    let home = directories::BaseDirs::new().map(|d| d.home_dir().to_path_buf());
    match home.as_deref().map(|h| (h, path.strip_prefix(h))) {
        Some((h, Ok(rest))) if h != Path::new("/") => {
            if rest.as_os_str().is_empty() {
                "~".to_string()
            } else {
                format!("~/{}", rest.display())
            }
        }
        _ => full,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A scratch directory that is removed when the test ends.
    pub struct Scratch(pub PathBuf);

    impl Scratch {
        pub fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("tinsel-{name}-{}", std::process::id()));
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
    fn nothing_saved_yet_is_an_empty_stocking() {
        let scratch = Scratch::new("empty");
        let p = load(&scratch.0).unwrap();
        assert_eq!(p.version, 1);
        assert!(p.captured.is_empty() && p.badges.is_empty());
        assert!(!scratch.0.exists(), "loading must not create anything");
    }

    #[test]
    fn progress_survives_a_save_and_a_load() {
        let scratch = Scratch::new("roundtrip");
        let mut p = load(&scratch.0).unwrap();
        p.knowledge.insert("krampus".into(), 2);
        p.captured
            .insert("tomte".into(), "2026-12-24T21:00:00".into());
        p.seen.insert("marley".into(), true);
        p.best_quiz_score = 90;
        save(&scratch.0, &p).unwrap();
        assert_eq!(load(&scratch.0).unwrap(), p);
        let text = fs::read_to_string(path_in(&scratch.0)).unwrap();
        assert!(text.contains("\"bestQuizScore\": 90"), "{text}");
        assert!(text.ends_with("}\n"));
        // Nothing is left beside it.
        let files: Vec<_> = fs::read_dir(&scratch.0).unwrap().collect();
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn an_older_file_with_fields_missing_still_loads() {
        let scratch = Scratch::new("older");
        fs::create_dir_all(&scratch.0).unwrap();
        fs::write(
            path_in(&scratch.0),
            r#"{"mashesPlayed": 3, "futureField": true}"#,
        )
        .unwrap();
        let p = load(&scratch.0).unwrap();
        assert_eq!(p.mashes_played, 3);
        assert!(p.knowledge.is_empty());
    }

    #[test]
    fn a_broken_file_is_a_store_error_with_a_way_out() {
        let scratch = Scratch::new("broken");
        fs::create_dir_all(&scratch.0).unwrap();
        fs::write(path_in(&scratch.0), "{ not json").unwrap();
        let e = load(&scratch.0).unwrap_err();
        assert_eq!(e.code, Code::Store);
        assert!(e.hint.unwrap().contains("stocking reset"));
    }

    #[test]
    fn reset_empties_the_stocking_and_can_be_repeated() {
        let scratch = Scratch::new("reset");
        let p = Progress {
            mashes_played: 1,
            ..Progress::default()
        };
        save(&scratch.0, &p).unwrap();
        reset(&scratch.0).unwrap();
        reset(&scratch.0).unwrap();
        assert_eq!(load(&scratch.0).unwrap().mashes_played, 0);
    }

    #[test]
    fn tilde_shortens_only_paths_under_home() {
        if cfg!(windows) {
            return;
        }
        let Some(dirs) = directories::BaseDirs::new() else {
            return;
        };
        let home = dirs.home_dir();
        if home == Path::new("/") {
            return;
        }
        assert_eq!(tilde(&home.join(".config/x.json")), "~/.config/x.json");
        assert_eq!(tilde(home), "~");
        assert_eq!(tilde(Path::new("/var/tmp/x.json")), "/var/tmp/x.json");
    }
}
