//! End-to-end tests: run the real binary the way a player or a script would.

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

fn tinsel() -> Command {
    let mut cmd = Command::cargo_bin("terminal-of-tinsel").unwrap();
    cmd.env("TERMINAL_OF_TINSEL_SEED", "25")
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
