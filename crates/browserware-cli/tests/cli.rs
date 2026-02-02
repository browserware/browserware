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
fn browsers_table_format() {
    brw()
        .args(["browsers", "--format", "table"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ID"))
        .stdout(predicate::str::contains("NAME"))
        .stdout(predicate::str::contains("FAMILY"))
        .stdout(predicate::str::contains("VERSION"))
        .stdout(predicate::str::contains("browser(s) detected"));
}

#[test]
fn browsers_json_format() {
    brw()
        .args(["browsers", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"browsers\":"))
        .stdout(predicate::str::contains("\"count\":"));
}

#[test]
fn browsers_plain_format() {
    brw()
        .args(["browsers", "--format", "plain"])
        .assert()
        .success();
    // Plain format should not contain table headers
    brw()
        .args(["browsers", "--format", "plain"])
        .assert()
        .stdout(predicate::str::contains("ID").not())
        .stdout(predicate::str::contains("NAME").not());
}

#[test]
fn browsers_json_is_valid() {
    use serde_json::Value;

    let output = brw()
        .args(["browsers", "--format", "json"])
        .output()
        .expect("Failed to run command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Result<Value, _> = serde_json::from_str(&stdout);
    assert!(
        parsed.is_ok(),
        "JSON output should be valid: {}",
        parsed.err().unwrap()
    );

    let json = parsed.unwrap();
    assert!(json["browsers"].is_array(), "browsers should be an array");
    assert!(json["count"].is_number(), "count should be a number");
}

#[test]
fn browsers_family_filter_chromium() {
    brw()
        .args(["browsers", "--family", "chromium"])
        .assert()
        .success();
}

#[test]
fn browsers_family_filter_firefox() {
    brw()
        .args(["browsers", "--family", "firefox"])
        .assert()
        .success();
}

#[test]
fn browsers_family_filter_webkit() {
    brw()
        .args(["browsers", "--family", "webkit"])
        .assert()
        .success();
}

#[test]
fn browsers_family_filter_invalid() {
    brw()
        .args(["browsers", "--family", "invalid-family"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown browser family"))
        .stderr(predicate::str::contains("Valid families: chromium, firefox, webkit, other"));
}

#[test]
fn browsers_global_format_flag() {
    // Test that global --format flag works
    brw()
        .args(["--format", "json", "browsers"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"browsers\":"));
}

#[test]
fn browsers_short_family_flag() {
    // Test short form of family flag
    brw()
        .args(["browsers", "-F", "chromium"])
        .assert()
        .success();
}

#[test]
fn open_subcommand_exists() {
    brw()
        .args(["open", "https://example.com"])
        .assert()
        .success();
}
