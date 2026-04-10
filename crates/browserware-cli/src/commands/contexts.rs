//! `brw contexts` subcommand.

use std::fmt::Write as _;

use browserware_detect::detect_browsers;
use browserware_profiles::discover_profiles;
use browserware_types::{BrowserContext, LaunchCapability};

use crate::OutputFormat;

/// Run `brw contexts`.
///
/// Discovers all browser contexts on the current machine and renders them
/// in the requested output format.
#[allow(clippy::redundant_pub_crate)]
pub(crate) fn run(format: OutputFormat) {
    let contexts = discover_contexts();
    match format {
        OutputFormat::Table => print!("{}", format_table(&contexts)),
        OutputFormat::Json => print!("{}", format_json(&contexts)),
        OutputFormat::Plain => print!("{}", format_plain(&contexts)),
    }
}

/// Discover all browser contexts by combining browser detection with profile discovery.
fn discover_contexts() -> Vec<BrowserContext> {
    let browsers = detect_browsers();
    let mut contexts = Vec::new();

    for browser in browsers {
        let discovery = discover_profiles(&browser);
        if discovery.profiles.is_empty() {
            contexts.push(BrowserContext::new(browser, None, discovery.capability));
        } else {
            for profile in discovery.profiles {
                contexts.push(BrowserContext::new(
                    browser.clone(),
                    Some(profile),
                    discovery.capability.clone(),
                ));
            }
        }
    }

    contexts.sort_by(|a, b| a.selector().cmp(b.selector()));
    contexts
}

/// Format a list of browser contexts as a human-readable table.
///
/// Returns a string with a header row, separator, one row per context,
/// and a footer with the context count. Returns a "no contexts" message
/// when the slice is empty.
pub(crate) fn format_table(contexts: &[BrowserContext]) -> String {
    if contexts.is_empty() {
        return "No browser contexts detected.\n".to_string();
    }

    let selector_width = contexts
        .iter()
        .map(|c| c.selector().len())
        .max()
        .unwrap_or(0)
        .max(8);

    let browser_width = contexts
        .iter()
        .map(|c| c.browser.name.len())
        .max()
        .unwrap_or(0)
        .max(7);

    let profile_width = contexts
        .iter()
        .map(|c| {
            c.profile
                .as_ref()
                .map_or(1, |p| p.display_name.len())
        })
        .max()
        .unwrap_or(0)
        .max(7);

    let launch_width: usize = 12;

    let sw = selector_width;
    let bw = browser_width;
    let pw = profile_width;
    let lw = launch_width;

    let mut out = String::new();

    // Header
    let _ = writeln!(
        out,
        "{:<sw$}  {:<bw$}  {:<pw$}  {:<lw$}",
        "SELECTOR", "BROWSER", "PROFILE", "LAUNCH",
        sw = sw, bw = bw, pw = pw, lw = lw,
    );

    // Separator
    let _ = writeln!(
        out,
        "{:-<sw$}  {:-<bw$}  {:-<pw$}  {:-<lw$}",
        "", "", "", "",
        sw = sw, bw = bw, pw = pw, lw = lw,
    );

    // Rows
    for ctx in contexts {
        let profile_name = ctx
            .profile
            .as_ref()
            .map_or("-", |p| p.display_name.as_str());
        let label = launch_label(&ctx.capability);

        let _ = writeln!(
            out,
            "{:<sw$}  {:<bw$}  {:<pw$}  {:<lw$}",
            ctx.selector(),
            ctx.browser.name,
            profile_name,
            label,
            sw = sw, bw = bw, pw = pw, lw = lw,
        );
    }

    // Footer
    out.push('\n');
    let _ = writeln!(out, "{} context(s) detected", contexts.len());

    out
}

/// Format a list of browser contexts as plain text, one selector per line.
///
/// Returns an empty string for an empty slice. Otherwise returns each
/// selector on its own line with a trailing newline.
pub(crate) fn format_plain(contexts: &[BrowserContext]) -> String {
    if contexts.is_empty() {
        return String::new();
    }
    let mut out = contexts
        .iter()
        .map(BrowserContext::selector)
        .collect::<Vec<_>>()
        .join("\n");
    out.push('\n');
    out
}

/// Format a list of browser contexts as pretty-printed JSON.
///
/// Returns a JSON object with `contexts` array and `count` field.
/// On serialisation error returns a JSON error object.
pub(crate) fn format_json(contexts: &[BrowserContext]) -> String {
    #[derive(serde::Serialize)]
    struct Output<'a> {
        contexts: &'a [BrowserContext],
        count: usize,
    }

    let output = Output {
        contexts,
        count: contexts.len(),
    };

    match serde_json::to_string_pretty(&output) {
        Ok(json) => json,
        Err(e) => format!(r#"{{"error": "{e}"}}"#),
    }
}

/// Return a human-readable launch capability label.
///
/// - `"profile"` when the capability supports profile-specific launch
/// - `"basic"` when only basic launch is supported
/// - `"limited"` when even basic launch is not available
const fn launch_label(cap: &LaunchCapability) -> &'static str {
    if cap.profile_launchable {
        "profile"
    } else if cap.launchable {
        "basic"
    } else {
        "limited"
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use browserware_types::{
        Browser, BrowserContext, BrowserVariant, ChromiumChannel, FirefoxChannel, LaunchCapability,
        ProfileRef, WebKitChannel,
    };

    use super::{format_json, format_plain, format_table};

    // ── Test helpers ──────────────────────────────────────────────────────────

    fn chrome_ctx(profile_id: &str, display_name: &str) -> BrowserContext {
        BrowserContext::new(
            Browser::new("chrome", "Google Chrome", PathBuf::from("/usr/bin/chrome"))
                .with_variant(BrowserVariant::Chromium(ChromiumChannel::Stable))
                .with_version("120.0.0"),
            Some(ProfileRef {
                id: profile_id.to_string(),
                display_name: display_name.to_string(),
            }),
            LaunchCapability::full(),
        )
    }

    fn safari_ctx() -> BrowserContext {
        BrowserContext::new(
            Browser::new("safari", "Safari", PathBuf::from("/Applications/Safari.app"))
                .with_variant(BrowserVariant::WebKit(WebKitChannel::Stable)),
            None,
            LaunchCapability::launch_only("WebKit/Safari does not support profile-specific launch"),
        )
    }

    fn firefox_ctx() -> BrowserContext {
        BrowserContext::new(
            Browser::new("firefox", "Firefox", PathBuf::from("/usr/bin/firefox"))
                .with_variant(BrowserVariant::Firefox(FirefoxChannel::Stable)),
            Some(ProfileRef {
                id: "default-release".to_string(),
                display_name: "default-release".to_string(),
            }),
            LaunchCapability::full(),
        )
    }

    // ── Table tests ───────────────────────────────────────────────────────────

    #[test]
    fn table_output_has_headers() {
        let contexts = vec![chrome_ctx("Default", "Personal")];
        let out = format_table(&contexts);
        assert!(out.contains("SELECTOR"), "missing SELECTOR header");
        assert!(out.contains("BROWSER"), "missing BROWSER header");
        assert!(out.contains("PROFILE"), "missing PROFILE header");
        assert!(out.contains("LAUNCH"), "missing LAUNCH header");
    }

    #[test]
    fn table_output_has_count_line() {
        let contexts = vec![chrome_ctx("Default", "Personal"), safari_ctx()];
        let out = format_table(&contexts);
        assert!(out.contains("2 context(s)"), "missing count line");
    }

    #[test]
    fn table_output_shows_selector() {
        let ctx = chrome_ctx("Profile 1", "Work");
        let expected_selector = ctx.selector().to_string();
        let contexts = vec![ctx];
        let out = format_table(&contexts);
        assert!(
            out.contains(&expected_selector),
            "output does not contain selector: {expected_selector}"
        );
    }

    #[test]
    fn table_output_no_contexts_message() {
        let out = format_table(&[]);
        assert!(out.contains("No browser contexts detected."));
    }

    #[test]
    fn table_output_dash_for_no_profile() {
        let ctx = safari_ctx();
        let contexts = vec![ctx];
        let out = format_table(&contexts);
        // The row must contain the browser name and a "-" for the absent profile
        assert!(out.contains("Safari"), "missing Safari row");
        assert!(out.contains(" - ") || out.contains("  -  "), "missing '-' for absent profile");
    }

    // ── Plain tests ───────────────────────────────────────────────────────────

    #[test]
    fn plain_output_one_selector_per_line() {
        let contexts = vec![chrome_ctx("Default", "Personal"), safari_ctx()];
        let out = format_plain(&contexts);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 2, "expected 2 lines, got: {lines:?}");
        assert!(
            lines[0].contains("family=chromium"),
            "line 0 missing 'family=chromium': {}",
            lines[0]
        );
        assert!(
            lines[1].contains("family=webkit"),
            "line 1 missing 'family=webkit': {}",
            lines[1]
        );
    }

    #[test]
    fn plain_output_empty_is_empty() {
        let out = format_plain(&[]);
        assert!(out.trim().is_empty(), "expected empty string for empty slice");
    }

    // ── JSON tests ────────────────────────────────────────────────────────────

    #[test]
    fn json_output_is_valid() {
        let contexts = vec![chrome_ctx("Default", "Personal"), safari_ctx(), firefox_ctx()];
        let out = format_json(&contexts);
        let parsed: serde_json::Value = serde_json::from_str(&out).expect("invalid JSON");
        assert!(parsed["contexts"].is_array());
        assert_eq!(parsed["count"], 3);
    }

    #[test]
    fn json_output_context_has_required_fields() {
        let contexts = vec![chrome_ctx("Default", "Personal")];
        let out = format_json(&contexts);
        let parsed: serde_json::Value = serde_json::from_str(&out).expect("invalid JSON");
        let ctx = &parsed["contexts"][0];
        assert!(ctx["browser"].is_object(), "missing browser field");
        assert!(ctx["profile"].is_object(), "missing profile field");
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
    fn json_output_null_profile_for_no_profile_context() {
        let contexts = vec![safari_ctx()];
        let out = format_json(&contexts);
        let parsed: serde_json::Value = serde_json::from_str(&out).expect("invalid JSON");
        assert!(
            parsed["contexts"][0]["profile"].is_null(),
            "expected null profile for Safari context"
        );
    }

    #[test]
    fn json_output_profile_launchable_false_for_basic() {
        let contexts = vec![safari_ctx()];
        let out = format_json(&contexts);
        let parsed: serde_json::Value = serde_json::from_str(&out).expect("invalid JSON");
        assert_eq!(
            parsed["contexts"][0]["capability"]["profile_launchable"],
            false,
            "expected profile_launchable=false for Safari"
        );
    }

    // ── Golden JSON test ──────────────────────────────────────────────────────

    #[test]
    fn json_golden_output() {
        let contexts = vec![
            chrome_ctx("Default", "Personal"),
            chrome_ctx("Profile 1", "Work"),
        ];
        let out = format_json(&contexts);
        let parsed: serde_json::Value = serde_json::from_str(&out).expect("invalid JSON");

        assert_eq!(parsed["count"], 2);

        let c0 = &parsed["contexts"][0];
        assert_eq!(c0["browser"]["id"], "chrome");
        assert_eq!(c0["browser"]["name"], "Google Chrome");
        assert_eq!(c0["profile"]["id"], "Default");
        assert_eq!(c0["profile"]["display_name"], "Personal");
        assert_eq!(
            c0["selector"],
            "family=chromium,browser=chrome,profile=Default"
        );
        assert_eq!(c0["capability"]["discoverable"], true);
        assert_eq!(c0["capability"]["profile_launchable"], true);
        assert_eq!(
            c0["capability"]["limitations"],
            serde_json::json!([]),
            "limitations should be empty"
        );

        let c1 = &parsed["contexts"][1];
        assert_eq!(c1["profile"]["id"], "Profile 1");
    }
}
