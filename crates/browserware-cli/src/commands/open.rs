//! `brw open` subcommand.

use anyhow::{Context, Result};
use browserware_launch::{build_command, format_command, launch};
use browserware_types::BrowserContext;
use browserware_types::selector::{AmbiguityPolicy, ContextSelector};
use url::Url;

use crate::OutputFormat;
use crate::commands::contexts::discover_contexts;

/// Run `brw open`.
#[allow(clippy::redundant_pub_crate)]
pub(crate) fn run(
    _format: OutputFormat,
    context_selector: Option<&str>,
    urls: &[String],
    dry_run: bool,
) {
    match open_context(&discover_contexts(), context_selector, urls, dry_run) {
        Ok(Some(line)) => println!("{line}"),
        Ok(None) => {}
        Err(e) => {
            eprintln!("{e:#}");
            std::process::exit(1);
        }
    }
}

/// Resolve a selector against `contexts` and launch or dry-run.
///
/// Returns `Ok(Some(command_line))` when `dry_run` is true, `Ok(None)` after a successful
/// launch, or `Err` on failure.
pub(crate) fn open_context(
    contexts: &[BrowserContext],
    context_selector: Option<&str>,
    urls: &[String],
    dry_run: bool,
) -> Result<Option<String>> {
    let parsed_urls: Vec<Url> = urls
        .iter()
        .map(|s| {
            Url::parse(s).with_context(|| {
                format!("error: invalid URL\n  cause: could not parse URL {s:?}\n  hint: pass a full URL such as https://example.com")
            })
        })
        .collect::<Result<_>>()?;

    let Some(sel_str) = context_selector else {
        anyhow::bail!(
            "error: no context specified\n  cause: `--context` was not provided\n  hint: use `brw open <url> --context <selector>` to open in a specific browser context\n  hint: run `brw contexts` to see available selectors\n  note: rules-based routing (`brw open <url>` without `--context`) is coming in PR3"
        );
    };

    let selector = ContextSelector::parse(sel_str).with_context(|| {
        format!(
            "error: invalid context selector\n  cause: could not parse selector {sel_str:?}\n  hint: run `brw contexts` and copy a selector or use `browser:profile` form"
        )
    })?;

    let selected = selector.select(contexts, AmbiguityPolicy::First).with_context(|| {
        "error: could not resolve selector\n  hint: run `brw contexts` to list installed contexts"
    })?;

    let Some(ctx) = selected else {
        anyhow::bail!(
            "error: no context matches selector {sel_str:?}\n  cause: nothing on this machine satisfies the selector\n  hint: run `brw contexts` to copy a valid `--context` value"
        );
    };

    if dry_run {
        let cmd = build_command(ctx, &parsed_urls).map_err(|e| map_launch_err(e, ctx))?;
        return Ok(Some(format_command(&cmd)));
    }

    launch(ctx, &parsed_urls).map_err(|e| map_launch_err(e, ctx))?;
    Ok(None)
}

fn map_launch_err(e: browserware_launch::LaunchError, ctx: &BrowserContext) -> anyhow::Error {
    match e {
        browserware_launch::LaunchError::NotLaunchable { limitations } => anyhow::anyhow!(
            "error: this context cannot be launched\n  cause: {limitations:?}\n  hint: run `brw contexts` to find a context with launch support (selector: {})",
            ctx.selector()
        ),
        browserware_launch::LaunchError::ExecutableNotFound(p) => anyhow::anyhow!(
            "error: browser executable missing\n  cause: {}\n  hint: reinstall the browser or check `brw browsers` paths",
            p.display()
        ),
        browserware_launch::LaunchError::EmptyUrls => {
            anyhow::anyhow!(
                "error: no URLs to open\n  hint: pass at least one URL after the options"
            )
        }
        browserware_launch::LaunchError::Io(err) => anyhow::anyhow!(
            "error: failed to start browser\n  cause: {err}\n  hint: try `brw open --dry-run --context ...` to inspect the command"
        ),
        browserware_launch::LaunchError::ProcessFailed(status) => anyhow::anyhow!(
            "error: browser exited with status {status}\n  hint: run with `RUST_LOG=info` for details"
        ),
        browserware_launch::LaunchError::FirefoxProfilePathMissing { profile_id } => {
            anyhow::anyhow!(
                "error: cannot launch Firefox profile\n  cause: profile '{profile_id}' has no resolved path in profiles.ini\n  hint: open Firefox once to populate the profile, then retry"
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use browserware_types::{
        Browser, BrowserVariant, ChromiumChannel, LaunchCapability, ProfileRef,
    };

    use super::*;

    fn chrome_ctx(profile_id: &str, display_name: &str) -> BrowserContext {
        BrowserContext::new(
            Browser::new("chrome", "Google Chrome", PathBuf::from("/bin/sh"))
                .with_variant(BrowserVariant::Chromium(ChromiumChannel::Stable)),
            Some(ProfileRef {
                id: profile_id.to_string(),
                display_name: display_name.to_string(),
                path: None,
            }),
            LaunchCapability::full(),
        )
    }

    #[cfg(unix)]
    #[test]
    fn open_context_dry_run_returns_command() {
        let contexts = vec![chrome_ctx("Work", "Work")];
        let urls = vec!["https://example.com".to_string()];
        let line = open_context(&contexts, Some("chrome:Work"), &urls, true)
            .unwrap()
            .expect("dry-run should return a line");
        assert!(
            line.contains("--profile-directory=Work") && line.contains("https://example.com"),
            "{line}"
        );
    }

    /// `--context chrome:Work` where the internal id is "Profile 1" and the user-visible
    /// display name is "Work".  This is the common Chrome case that was broken.
    #[cfg(unix)]
    #[test]
    fn open_context_dry_run_matches_by_display_name() {
        let contexts = vec![chrome_ctx("Profile 1", "Work")];
        let urls = vec!["https://example.com".to_string()];
        let line = open_context(&contexts, Some("chrome:Work"), &urls, true)
            .unwrap()
            .expect("dry-run should return a line");
        // Launch args must use the internal id, not the display name.
        assert!(
            line.contains("--profile-directory=Profile 1") && line.contains("https://example.com"),
            "{line}"
        );
    }

    /// A partial selector (`chrome`) with multiple matching profiles must not error;
    /// it should silently pick the first match (AmbiguityPolicy::First).
    #[cfg(unix)]
    #[test]
    fn open_context_partial_selector_picks_first_silently() {
        let contexts = vec![
            chrome_ctx("Profile 1", "Work"),
            chrome_ctx("Profile 2", "Personal"),
        ];
        let urls = vec!["https://example.com".to_string()];
        let line = open_context(&contexts, Some("chrome"), &urls, true)
            .unwrap()
            .expect("dry-run should return a line");
        assert!(line.contains("--profile-directory=Profile 1"), "{line}");
    }

    #[test]
    fn open_context_requires_context_flag() {
        let contexts: Vec<BrowserContext> = vec![];
        let urls = vec!["https://example.com".to_string()];
        let err = open_context(&contexts, None, &urls, false).unwrap_err();
        assert!(err.to_string().contains("no context specified"), "{err:#}");
    }

    #[test]
    fn open_context_no_match_returns_error() {
        // Selector is syntactically valid but matches nothing in the context list.
        let contexts = vec![chrome_ctx("Profile 1", "Work")];
        let urls = vec!["https://example.com".to_string()];
        let err = open_context(&contexts, Some("chrome:nonexistent"), &urls, false).unwrap_err();
        assert!(
            err.to_string().contains("no context matches"),
            "{err:#}"
        );
    }
}
