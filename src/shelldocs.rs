//! The man pages and completion scripts that ship in the release archives.
//!
//! cargo-dist copies `man/` and `completions/` into each archive, but it has
//! no step that could generate them first, so they are committed, and the
//! test here fails when they no longer match the command tree. To write
//! them afresh after changing a command, a flag or the version:
//!
//! ```text
//! UPDATE_SHELLDOCS=1 cargo test shell_docs
//! ```

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use clap::CommandFactory;
use clap_complete::env::Shells;

use crate::cli::Cli;

const BIN: &str = "terminal-of-tinsel";

/// Writes one kind of file into a directory.
type Generator = fn(&Path);

/// Every file under a directory, as path (relative, with forward slashes)
/// to contents. Carriage returns are dropped, so a Windows checkout compares
/// equal.
fn files_in(dir: &Path) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if let Ok(text) = fs::read_to_string(entry.path()) {
            out.insert(name, text.replace('\r', ""));
        }
    }
    out
}

/// Generates the man pages into `dir`: one for the program and one for each
/// command and subcommand.
fn write_man_pages(dir: &Path) {
    fs::create_dir_all(dir).unwrap();
    clap_mangen::generate_to(Cli::command(), dir).unwrap();
}

/// Generates a completion script for each shell into `dir`. Each is a few
/// lines that ask the program itself what to offer.
fn write_completions(dir: &Path) {
    fs::create_dir_all(dir).unwrap();
    for (shell, extension) in [
        ("bash", "bash"),
        ("elvish", "elv"),
        ("fish", "fish"),
        ("powershell", "ps1"),
        ("zsh", "zsh"),
    ] {
        let mut script = Vec::new();
        Shells::builtins()
            .completer(shell)
            .unwrap()
            .write_registration("COMPLETE", BIN, BIN, BIN, &mut script)
            .unwrap();
        fs::write(dir.join(format!("{BIN}.{extension}")), script).unwrap();
    }
}

#[test]
fn shell_docs_are_up_to_date() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let scratch: PathBuf =
        std::env::temp_dir().join(format!("tinsel-shelldocs-{}", std::process::id()));
    let _ = fs::remove_dir_all(&scratch);
    let generators: [(&str, Generator); 2] =
        [("man", write_man_pages), ("completions", write_completions)];

    for (name, generate) in generators {
        let committed = root.join(name);
        if std::env::var_os("UPDATE_SHELLDOCS").is_some() {
            let _ = fs::remove_dir_all(&committed);
            generate(&committed);
            continue;
        }
        let fresh_dir = scratch.join(name);
        generate(&fresh_dir);
        let (fresh, on_disk) = (files_in(&fresh_dir), files_in(&committed));
        let names =
            |files: &BTreeMap<String, String>| -> Vec<String> { files.keys().cloned().collect() };
        assert_eq!(
            names(&on_disk),
            names(&fresh),
            "{name}/ has the wrong files: run UPDATE_SHELLDOCS=1 cargo test shell_docs"
        );
        for (file, text) in &fresh {
            assert!(
                on_disk[file] == *text,
                "{name}/{file} is out of date: run UPDATE_SHELLDOCS=1 cargo test shell_docs"
            );
        }
    }
    let _ = fs::remove_dir_all(&scratch);
}

#[test]
fn there_is_a_man_page_for_every_command() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let pages = files_in(&root.join("man"));
    for command in Cli::command().get_subcommands() {
        let page = format!("{BIN}-{}.1", command.get_name());
        assert!(pages.contains_key(&page), "no {page}");
    }
    for page in ["terminal-of-tinsel.1", "terminal-of-tinsel-packs-new.1"] {
        assert!(pages.contains_key(page), "no {page}");
    }
}
