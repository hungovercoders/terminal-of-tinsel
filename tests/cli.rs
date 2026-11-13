//! End-to-end tests: run the real binary the way a player or a script would.

use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

/// A scratch config directory, so the tests never touch real progress.
/// Each test that cares about what's saved makes its own with `home`.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("tinsel-cli-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn tinsel() -> Command {
    tinsel_at(&scratch("shared"))
}

fn tinsel_at(home: &Path) -> Command {
    let mut cmd = Command::cargo_bin("terminal-of-tinsel").unwrap();
    cmd.env("TERMINAL_OF_TINSEL_SEED", "25")
        .env("TERMINAL_OF_TINSEL_HOME", home)
        .env_remove("NO_COLOR");
    cmd
}

fn json(args: &[&str]) -> Value {
    let out = tinsel().args(args).arg("--json").output().unwrap();
    assert!(
        out.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).unwrap()
}

fn json_error(args: &[&str]) -> Value {
    let out = tinsel().args(args).arg("--json").output().unwrap();
    assert!(!out.status.success(), "{args:?} should fail");
    assert!(out.stdout.is_empty());
    serde_json::from_slice(&out.stderr).unwrap()
}

#[test]
fn every_command_runs() {
    for args in [
        vec![],
        vec!["list"],
        vec!["creature", "krampus"],
        vec!["creature", "mari", "lwyd", "--reveal"],
        vec!["creature", "--json"],
        vec!["quiz", "--json"],
        vec!["quiz", "marley", "-n", "3", "--json"],
        vec!["guess", "--json"],
        vec!["mash"],
        vec!["mash", "krampus", "yule cat", "--fast"],
        vec!["mash", "gryla", "--json"],
        vec!["stocking"],
        vec!["stocking", "--json"],
        vec!["stocking", "reset", "-y"],
        vec!["random"],
        vec!["random", "--daily"],
        vec!["countdown"],
        vec!["advent"],
        vec!["completions", "zsh"],
    ] {
        tinsel()
            .args(&args)
            .assert()
            .success()
            .stdout(predicate::str::is_empty().not());
    }
}

#[test]
fn piped_output_is_plain() {
    tinsel()
        .args(["creature", "krampus"])
        .assert()
        .success()
        .stdout(predicate::str::contains("KRAMPUS"))
        .stdout(predicate::str::contains("\u{1b}[").not())
        .stdout(predicate::str::contains("😈").not());
}

#[test]
fn list_json_is_every_creature() {
    let v = json(&["list"]);
    let all = v.as_array().unwrap();
    assert_eq!(all.len(), 14);
    assert!(
        all.iter()
            .all(|c| c["pack"].is_string() && c["facts"].is_array())
    );
}

#[test]
fn pack_filter_narrows_the_list() {
    let v = json(&["list", "--pack", "dickens"]);
    let ids: Vec<&str> = v
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["marley", "ghost-yet-to-come"]);
}

#[test]
fn creature_json_by_alias() {
    let v = json(&["creature", "jolakotturinn"]);
    assert_eq!(v["id"], "yule-cat");
    assert_eq!(v["myths"][0]["true"], false);
}

#[test]
fn creature_json_without_a_name_is_every_creature() {
    let v = json(&["creature"]);
    assert_eq!(v.as_array().unwrap().len(), 14);
    let v = json(&["creature", "--pack", "nordic"]);
    assert_eq!(v.as_array().unwrap().len(), 4);
}

#[test]
fn creature_json_carries_the_portrait() {
    let v = json(&["creature", "tomte"]);
    let art = v["ascii"].as_str().unwrap();
    assert!(art.lines().count() > 5);
    assert!(!art.contains('\r') && !art.ends_with('\n'));
}

#[test]
fn the_explorer_refuses_to_run_without_a_terminal() {
    // Under `cargo test` neither stdin nor stdout is a terminal, as in a
    // script, a pipe or CI.
    for args in [
        vec!["creature"],
        vec!["creature", "--all", "--no-intro"],
        vec!["creature", "krampus", "--explore"],
    ] {
        tinsel()
            .args(&args)
            .assert()
            .failure()
            .stdout(predicate::str::is_empty())
            .stderr(predicate::str::contains("needs a terminal"))
            .stderr(predicate::str::contains("hint: use --json for the data"));
    }
    // A name the vault doesn't know is reported before the terminal is.
    tinsel()
        .args(["creature", "rudolph", "--explore"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no creature called"));
}

#[test]
fn quiz_json_is_seeded_and_carries_its_answers() {
    let a = json(&["quiz", "-n", "6"]);
    assert_eq!(
        a,
        json(&["quiz", "-n", "6"]),
        "the same seed, the same quiz"
    );
    let questions = a.as_array().unwrap();
    assert_eq!(questions.len(), 6);
    for q in questions {
        let options = q["options"].as_array().unwrap();
        assert!((q["answer"].as_u64().unwrap() as usize) < options.len());
        assert!(q["kind"].is_string() && q["prompt"].is_string() && q["creatureId"].is_string());
    }
    let focused = json(&["quiz", "yule", "cat", "--questions", "4"]);
    assert!(
        focused
            .as_array()
            .unwrap()
            .iter()
            .all(|q| q["creatureId"] == "yule-cat")
    );
}

#[test]
fn guess_json_has_rounds_with_four_options() {
    let v = json(&["guess", "--rounds", "3"]);
    let rounds = v.as_array().unwrap();
    assert_eq!(rounds.len(), 3);
    for r in rounds {
        let options = r["options"].as_array().unwrap();
        assert_eq!(options.len(), 4);
        assert_eq!(options[r["answer"].as_u64().unwrap() as usize], r["name"]);
    }
    assert_eq!(v, json(&["guess", "--rounds", "3"]));
}

#[test]
fn mash_json_names_both_corners_and_a_winner() {
    let v = json(&["mash", "krampus", "yule cat"]);
    assert_eq!(v["a"]["id"], "krampus");
    assert_eq!(v["b"]["id"], "yule-cat");
    assert_eq!(v["rounds"].as_array().unwrap().len(), 3);
    let winner = v["winner"].as_str().unwrap();
    assert!(winner == "krampus" || winner == "yule-cat");
    assert!(v["rounds"][0]["rollA"].is_u64() && v["rounds"][0]["move"].is_string());
    assert!(v["finale"].as_str().unwrap().contains("trudged off"));
    assert_eq!(v, json(&["mash", "krampus", "yule cat"]));
    // One name, or none: the other corner is filled, never with the same creature.
    let v = json(&["mash", "tomte"]);
    assert_eq!(v["a"]["id"], "tomte");
    assert_ne!(v["b"]["id"], "tomte");
}

#[test]
fn piped_mash_is_plain_and_does_not_pause() {
    let started = std::time::Instant::now();
    tinsel()
        .args(["mash", "marley", "gryla"])
        .assert()
        .success()
        .stdout(predicate::str::contains("THE SNOWBALL SHOWDOWN"))
        .stdout(predicate::str::contains("WINNER:"))
        .stdout(predicate::str::contains("\u{1b}[").not())
        .stdout(predicate::str::contains("🥊").not());
    assert!(started.elapsed().as_secs() < 3, "a pipe should not wait");
}

#[test]
fn the_games_refuse_to_run_without_a_terminal() {
    for (args, name) in [
        (vec!["quiz"], "the Christmas Cracker Quiz"),
        (vec!["guess"], "Guess the Guest"),
    ] {
        tinsel()
            .args(&args)
            .assert()
            .failure()
            .stdout(predicate::str::is_empty())
            .stderr(predicate::str::contains(format!("{name} needs a terminal")))
            .stderr(predicate::str::contains("hint: use --json"));
    }
}

#[test]
fn a_bad_seed_warns_and_is_ignored() {
    tinsel()
        .env("TERMINAL_OF_TINSEL_SEED", "tinsel")
        .args(["mash", "--fast"])
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "warning: ignoring TERMINAL_OF_TINSEL_SEED",
        ));
}

#[test]
fn a_showdown_goes_in_the_stocking_and_reset_empties_it() {
    let home = scratch("stocking");
    let stocking = || -> Value {
        let out = tinsel_at(&home)
            .args(["stocking", "--json"])
            .output()
            .unwrap();
        serde_json::from_slice(&out.stdout).unwrap()
    };
    assert_eq!(stocking()["mashesPlayed"], 0);
    assert!(!home.exists(), "looking in the stocking creates nothing");

    tinsel_at(&home)
        .args(["mash", "krampus", "gryla"])
        .assert()
        .success()
        .stdout(predicate::str::contains("BADGE: Snowball Referee"))
        .stdout(predicate::str::contains("terminal-of-tinsel stocking"));
    let saved = stocking();
    assert_eq!(saved["mashesPlayed"], 1);
    assert_eq!(saved["version"], 1);
    assert!(saved["badges"]["snowball-referee"].is_string());

    // The badge is announced once, and --pack doesn't hide it.
    tinsel_at(&home)
        .args(["mash", "--pack", "nordic"])
        .assert()
        .success()
        .stdout(predicate::str::contains("BADGE: Snowball Referee").not());
    assert_eq!(stocking()["mashesPlayed"], 2);

    // --json prints the showdown and records nothing, as in Terror.
    tinsel_at(&home).args(["mash", "--json"]).assert().success();
    assert_eq!(stocking()["mashesPlayed"], 2);

    // How many badges depends on the night: a showdown on Krampusnacht, under
    // a full moon or after midnight earns more than the referee's.
    let badges = stocking()["badges"].as_object().unwrap().len();
    assert!(badges >= 1);
    tinsel_at(&home)
        .arg("stocking")
        .assert()
        .success()
        .stdout(predicate::str::contains("THE STOCKING"))
        .stdout(predicate::str::contains(format!(
            "0 of 14 creatures caught · {badges} of 17 badges"
        )))
        .stdout(predicate::str::contains("Snowball Showdowns staged: 2"))
        .stdout(predicate::str::contains("Progress is saved in"))
        .stdout(predicate::str::contains("stocking.json"));

    // Without a terminal to confirm on, reset needs -y.
    let e = tinsel_at(&home)
        .args(["stocking", "reset", "--json"])
        .output()
        .unwrap();
    assert!(!e.status.success());
    let e: Value = serde_json::from_slice(&e.stderr).unwrap();
    assert_eq!(e["error"]["code"], "no-terminal");
    assert_eq!(stocking()["mashesPlayed"], 2);

    tinsel_at(&home)
        .args(["stocking", "reset", "-y"])
        .assert()
        .success()
        .stdout(predicate::str::contains("has been reset"));
    assert_eq!(stocking()["mashesPlayed"], 0);
    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn a_broken_save_file_stops_the_stocking_but_not_a_game() {
    let home = scratch("broken");
    std::fs::create_dir_all(&home).unwrap();
    std::fs::write(home.join("stocking.json"), "{ not json").unwrap();
    let out = tinsel_at(&home)
        .args(["stocking", "--json"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let e: Value = serde_json::from_slice(&out.stderr).unwrap();
    assert_eq!(e["error"]["code"], "store");
    assert!(
        e["error"]["hint"]
            .as_str()
            .unwrap()
            .contains("stocking reset")
    );

    tinsel_at(&home)
        .args(["mash", "--fast"])
        .assert()
        .success()
        .stdout(predicate::str::contains("WINNER:"))
        .stderr(predicate::str::contains(
            "warning: this session wasn't saved",
        ));
    // The broken file is left for its owner to mend.
    assert_eq!(
        std::fs::read_to_string(home.join("stocking.json")).unwrap(),
        "{ not json"
    );
    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn daily_fact_is_repeatable_and_follows_advent() {
    let a = json(&["random", "--daily", "--date", "2026-12-05"]);
    let b = json(&["random", "--daily", "--date", "2026-12-05"]);
    assert_eq!(a, b);
    assert_eq!(a["id"], "krampus");
    assert_eq!(a["date"], "2026-12-05");
}

#[test]
fn countdown_json() {
    let v = json(&["countdown", "--date", "2026-12-24"]);
    assert_eq!(v["sleepsUntilChristmas"], 1);
    assert_eq!(v["advent"]["night"], 24);
    assert_eq!(v["yuleLadsInTown"], 13);
    let v = json(&["countdown", "--date", "2026-12-25"]);
    assert_eq!(v["christmas"], true);
    assert_eq!(v["twelveDays"], 1);
}

#[test]
fn advent_json_has_24_doors() {
    let v = json(&["advent", "--date", "2026-12-10"]);
    let doors = v["doors"].as_array().unwrap();
    assert_eq!(doors.len(), 24);
    assert_eq!(doors[8]["door"], "open");
    assert_eq!(doors[9]["door"], "tonight");
    assert_eq!(doors[10]["door"], "shut");
}

#[test]
fn errors_have_stable_codes() {
    assert_eq!(
        json_error(&["creature", "rudolph"])["error"]["code"],
        "unknown-creature"
    );
    assert_eq!(
        json_error(&["creature", "yule"])["error"]["code"],
        "ambiguous-creature"
    );
    assert_eq!(
        json_error(&["countdown", "--date", "soon"])["error"]["code"],
        "bad-date"
    );
    assert_eq!(
        json_error(&["list", "--pack", "elves"])["error"]["code"],
        "bad-pack"
    );
    assert_eq!(
        json_error(&["list", "--sparkle"])["error"]["code"],
        "bad-flag"
    );
    assert_eq!(
        json_error(&["quiz", "-n", "0"])["error"]["code"],
        "bad-flag"
    );
    assert_eq!(
        json_error(&["guess", "--rounds", "0"])["error"]["code"],
        "bad-flag"
    );
    assert_eq!(
        json_error(&["quiz", "rudolph"])["error"]["code"],
        "unknown-creature"
    );
    for args in [
        vec!["mash", "krampus", "krampus"],
        vec!["guess", "--pack", "dickens"],
    ] {
        assert_eq!(json_error(&args)["error"]["code"], "usage", "{args:?}");
    }
}

#[test]
fn errors_without_json_are_friendly() {
    tinsel()
        .args(["creature", "krampas"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Did you mean Krampus?"))
        .stderr(predicate::str::contains(
            "hint: try 'terminal-of-tinsel list'",
        ));
}

#[test]
fn version_and_help() {
    tinsel()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("terminal-of-tinsel "));
    tinsel()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Auntie Aerial"));
}
