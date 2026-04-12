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
    let output = brw()
        .args(["browsers", "--format", "table"])
        .assert()
        .success();

    // Check that output is valid - either headers + summary if browsers exist,
    // or "No browsers detected." if none exist
    output.stdout(
        predicate::str::contains("ID")
            .and(predicate::str::contains("NAME"))
            .and(predicate::str::contains("FAMILY"))
            .and(predicate::str::contains("VERSION"))
            .and(predicate::str::contains("browser(s) detected"))
            .or(predicate::str::contains("No browsers detected.")),
    );
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
    let output = brw()
        .args(["browsers", "--format", "plain"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8_lossy(&output);

    // Plain format should not contain table headers
    assert!(
        !stdout.contains("ID"),
        "plain output unexpectedly contains ID header"
    );
    assert!(
        !stdout.contains("NAME"),
        "plain output unexpectedly contains NAME header"
    );
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
        .stderr(predicate::str::contains(
            "Valid families: chromium, firefox, webkit, other",
        ));
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
fn open_without_context_fails_with_hint() {
    brw()
        .args(["open", "https://example.com"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no context specified"))
        .stderr(predicate::str::contains("--context"));
}

// ─── brw contexts ────────────────────────────────────────────────────────────

#[test]
fn contexts_subcommand_exists() {
    brw().arg("contexts").assert().success();
}

#[test]
fn contexts_table_has_headers() {
    let output = brw().arg("contexts").assert().success();
    // Either headers + count OR the empty message
    output.stdout(
        predicate::str::contains("SELECTOR")
            .and(predicate::str::contains("BROWSER"))
            .and(predicate::str::contains("PROFILE"))
            .and(predicate::str::contains("LAUNCH"))
            .and(predicate::str::contains("context(s) detected"))
            .or(predicate::str::contains("No browser contexts detected.")),
    );
}

#[test]
fn contexts_json_is_valid() {
    use serde_json::Value;

    let output = brw()
        .args(["contexts", "--format", "json"])
        .output()
        .expect("Failed to run command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Result<Value, _> = serde_json::from_str(&stdout);
    assert!(
        parsed.is_ok(),
        "contexts JSON output should be valid: {:?}\nstdout: {stdout}",
        parsed.err()
    );

    let json = parsed.unwrap();
    assert!(json["contexts"].is_array(), "contexts should be an array");
    assert!(json["count"].is_number(), "count should be a number");
}

#[test]
fn contexts_json_fields_present_when_contexts_exist() {
    use serde_json::Value;

    let output = brw()
        .args(["contexts", "--format", "json"])
        .output()
        .expect("Failed to run command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).unwrap();

    let contexts = json["contexts"].as_array().unwrap();
    if contexts.is_empty() {
        // No browsers detected in this CI environment — skip field check
        return;
    }

    let ctx = &contexts[0];
    assert!(ctx["browser"].is_object(), "missing browser field");
    assert!(ctx["selector"].is_string(), "missing selector field");
    assert!(ctx["capability"].is_object(), "missing capability field");
    let cap = &ctx["capability"];
    assert!(cap["discoverable"].is_boolean());
    assert!(cap["launchable"].is_boolean());
    assert!(cap["profile_launchable"].is_boolean());
    assert!(cap["requires_user_config"].is_boolean());
    assert!(cap["limitations"].is_array());
}

#[test]
fn contexts_plain_format_no_table_headers() {
    brw()
        .args(["contexts", "--format", "plain"])
        .assert()
        .success()
        .stdout(predicate::str::contains("SELECTOR").not())
        .stdout(predicate::str::contains("BROWSER").not());
}

#[test]
fn contexts_plain_format_selectors_are_stable() {
    // Plain format should output canonical selector strings (key=value pairs)
    // If any contexts are present, each line should look like a selector
    let output = brw()
        .args(["contexts", "--format", "plain"])
        .output()
        .expect("Failed to run command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        assert!(
            line.starts_with("brw open --context \""),
            "plain output line missing copy-paste hint: {line:?}"
        );
        assert!(
            line.contains("family=") && line.contains("browser="),
            "plain output line is not a canonical selector: {line:?}"
        );
    }
}

#[test]
fn contexts_global_format_flag() {
    brw()
        .args(["--format", "json", "contexts"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"contexts\":"));
}
