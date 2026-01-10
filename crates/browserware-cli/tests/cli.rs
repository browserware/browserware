#![allow(missing_docs)]

use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

fn brw() -> Command {
    cargo_bin_cmd!("brw")
}

#[test]
fn help_works() {
    brw()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Smart browser routing CLI"));
}

#[test]
fn version_works() {
    brw()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn browsers_subcommand_exists() {
    brw().arg("browsers").assert().success();
}

#[test]
fn open_subcommand_exists() {
    brw()
        .args(["open", "https://example.com"])
        .assert()
        .success();
}
