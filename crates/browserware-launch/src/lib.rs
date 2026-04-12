//! Browser launching for the browserware ecosystem.
//!
//! Builds platform-appropriate [`std::process::Command`] values from a
//! [`browserware_types::BrowserContext`] and opens one or more URLs in a single
//! invocation (multiple tabs when the browser supports it).
//!
//! On macOS, when [`browserware_types::Browser::bundle_id`] is set, this crate uses
//! `open -b <bundle_id> <urls>... --args <profile flags>`. Profile flags are passed
//! after `--args` so URLs stay in the `open` position list. Some macOS builds have
//! been observed to drop or mishandle `--args`; use CLI `--dry-run` to inspect the
//! command before launching.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod error;

pub use error::LaunchError;

use std::ffi::OsString;
use std::process::Command;

use browserware_types::{BrowserContext, BrowserFamily};
use url::Url;

/// Format a [`Command`] as a single shell-style string (for `--dry-run` and logging).
#[must_use]
pub fn format_command(cmd: &Command) -> String {
    let mut parts: Vec<String> = Vec::new();
    parts.push(cmd.get_program().to_string_lossy().into_owned());
    for a in cmd.get_args() {
        parts.push(a.to_string_lossy().into_owned());
    }
    parts.join(" ")
}

/// Build the process command that would launch `context` with `urls`.
///
/// # Errors
///
/// Returns [`LaunchError::EmptyUrls`] when `urls` is empty,
/// [`LaunchError::NotLaunchable`] when the context is not launchable, or
/// [`LaunchError::ExecutableNotFound`] when using direct executable launch and the
/// path is missing.
#[must_use = "building a command has no effect if not spawned"]
pub fn build_command(context: &BrowserContext, urls: &[Url]) -> Result<Command, LaunchError> {
    if urls.is_empty() {
        return Err(LaunchError::EmptyUrls);
    }
    if !context.capability.launchable {
        return Err(LaunchError::NotLaunchable {
            limitations: context.capability.limitations.clone(),
        });
    }

    #[cfg(target_os = "macos")]
    if let Some(bundle_id) = context.browser.bundle_id.as_deref() {
        // `open -b <id> URL --args <flags>` only delivers flags when launching a *fresh*
        // process. If the app is already running, macOS forwards the URL via Apple Events
        // and silently drops `--args`, so `--profile-directory` is ignored.
        //
        // For profile-targeted launches we therefore fall through to `build_direct_command`.
        // Chromium honours `--profile-directory` even when an instance is already running
        // (it forwards the request to the existing process via its own IPC). Direct
        // invocation is reliable; `open -b` is not.
        //
        // For launches with no profile args (WebKit/Other, or Chromium without a profile)
        // we keep `open -b` because it avoids Gatekeeper friction for URL-only opens.
        if profile_launch_args(context).is_empty() {
            return Ok(build_macos_bundle_command(context, urls, bundle_id));
        }
    }

    build_direct_command(context, urls)
}

/// Launch `context` opening all `urls` in one browser invocation.
///
/// # Errors
///
/// See [`build_command`]. Additionally returns [`LaunchError::ProcessFailed`] when
/// the child exits non-zero, and [`LaunchError::Io`] when spawn fails.
pub fn launch(context: &BrowserContext, urls: &[Url]) -> Result<(), LaunchError> {
    let mut cmd = build_command(context, urls)?;
    tracing::info!(command = %format_command(&cmd), "spawning browser");
    let status = cmd.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(LaunchError::ProcessFailed(status))
    }
}

fn profile_launch_args(context: &BrowserContext) -> Vec<OsString> {
    let mut v = Vec::new();
    if !context.capability.profile_launchable {
        return v;
    }
    match context.browser.family() {
        BrowserFamily::Chromium => {
            if let Some(p) = &context.profile {
                let mut a = OsString::from("--profile-directory=");
                a.push(p.id.as_str());
                v.push(a);
            }
        }
        BrowserFamily::Firefox => {
            if let Some(p) = &context.profile
                && let Some(path) = &p.path
            {
                v.push(OsString::from("--profile"));
                v.push(path.clone().into_os_string());
                v.push(OsString::from("--no-remote"));
            }
        }
        BrowserFamily::WebKit | BrowserFamily::Other => {}
    }
    v
}

fn build_direct_command(context: &BrowserContext, urls: &[Url]) -> Result<Command, LaunchError> {
    let exe = &context.browser.executable;
    if !exe.exists() {
        return Err(LaunchError::ExecutableNotFound(exe.clone()));
    }
    let mut cmd = Command::new(exe);
    cmd.args(profile_launch_args(context));
    for u in urls {
        cmd.arg(u.as_str());
    }
    Ok(cmd)
}

#[cfg(target_os = "macos")]
fn build_macos_bundle_command(context: &BrowserContext, urls: &[Url], bundle_id: &str) -> Command {
    let extras = profile_launch_args(context);
    let mut cmd = Command::new("open");
    cmd.arg("-b");
    cmd.arg(bundle_id);
    for u in urls {
        cmd.arg(u.as_str());
    }
    if !extras.is_empty() {
        cmd.arg("--args");
        cmd.args(extras);
    }
    cmd
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use browserware_types::{
        Browser, BrowserContext, BrowserVariant, ChromiumChannel, FirefoxChannel, LaunchCapability,
        ProfileRef,
    };

    use super::*;

    fn example_url() -> Url {
        Url::parse("https://example.com").unwrap()
    }

    fn example_url_str() -> String {
        example_url().as_str().to_string()
    }

    fn chrome_ctx(
        executable: PathBuf,
        profile: Option<ProfileRef>,
        cap: LaunchCapability,
        bundle_id: Option<&str>,
    ) -> BrowserContext {
        let mut b = Browser::new("chrome", "Google Chrome", executable)
            .with_variant(BrowserVariant::Chromium(ChromiumChannel::Stable));
        if let Some(bid) = bundle_id {
            b = b.with_bundle_id(bid);
        }
        BrowserContext::new(b, profile, cap)
    }

    #[cfg(unix)]
    #[test]
    fn chromium_profile_args() {
        let ctx = chrome_ctx(
            PathBuf::from("/bin/sh"),
            Some(ProfileRef {
                id: "Work".into(),
                display_name: "Work".into(),
                path: None,
            }),
            LaunchCapability::full(),
            None,
        );
        let cmd = build_command(&ctx, &[example_url()]).unwrap();
        let args = cmd
            .get_args()
            .map(|a| a.to_string_lossy().to_string())
            .collect::<Vec<_>>();
        assert!(
            args.contains(&"--profile-directory=Work".to_string()),
            "args={args:?}"
        );
        assert!(args.contains(&example_url_str()));
    }

    #[cfg(unix)]
    #[test]
    fn firefox_profile_args() {
        let executable = PathBuf::from("/bin/sh");
        let browser = Browser::new("firefox", "Firefox", executable)
            .with_variant(BrowserVariant::Firefox(FirefoxChannel::Stable));
        let ctx = BrowserContext::new(
            browser,
            Some(ProfileRef {
                id: "rel".into(),
                display_name: "rel".into(),
                path: Some(PathBuf::from("/abs/firefox-profile")),
            }),
            LaunchCapability::full(),
        );
        let cmd = build_command(&ctx, &[example_url()]).unwrap();
        let args = cmd
            .get_args()
            .map(|a| a.to_string_lossy().to_string())
            .collect::<Vec<_>>();
        assert_eq!(
            args,
            vec![
                "--profile".to_string(),
                "/abs/firefox-profile".to_string(),
                "--no-remote".to_string(),
                example_url_str(),
            ]
        );
    }

    #[cfg(unix)]
    #[test]
    fn firefox_no_profile_args() {
        let executable = PathBuf::from("/bin/sh");
        let browser = Browser::new("firefox", "Firefox", executable)
            .with_variant(BrowserVariant::Firefox(FirefoxChannel::Stable));
        let ctx = BrowserContext::new(
            browser,
            Some(ProfileRef {
                id: "rel".into(),
                display_name: "rel".into(),
                path: None,
            }),
            LaunchCapability::full(),
        );
        let cmd = build_command(&ctx, &[example_url()]).unwrap();
        let args = cmd
            .get_args()
            .map(|a| a.to_string_lossy().to_string())
            .collect::<Vec<_>>();
        assert_eq!(args, vec![example_url_str()]);
    }

    /// macOS, bundle_id set, NO profile: must use `open -b` (Gatekeeper-safe URL open).
    #[cfg(target_os = "macos")]
    #[test]
    fn macos_bundle_id_no_profile_uses_open_b() {
        let ctx = chrome_ctx(
            PathBuf::from("/unused"),
            None,
            LaunchCapability::full(),
            Some("com.google.Chrome"),
        );
        let cmd = build_command(&ctx, &[example_url()]).unwrap();
        assert_eq!(cmd.get_program().to_string_lossy(), "open");
        let args = cmd
            .get_args()
            .map(|a| a.to_string_lossy().to_string())
            .collect::<Vec<_>>();
        assert_eq!(args[0], "-b");
        assert_eq!(args[1], "com.google.Chrome");
        assert_eq!(args[2], example_url_str());
        assert!(!args.contains(&"--args".to_string()), "no --args when no profile");
    }

    /// macOS, bundle_id set, profile present: must use direct executable.
    /// `open -b --args` silently drops flags for already-running apps, so
    /// `--profile-directory` would be ignored. Direct invocation is reliable.
    #[cfg(target_os = "macos")]
    #[test]
    fn macos_bundle_id_with_profile_uses_direct_exec() {
        let ctx = chrome_ctx(
            PathBuf::from("/bin/sh"),
            Some(ProfileRef {
                id: "Profile 1".into(),
                display_name: "Work".into(),
                path: None,
            }),
            LaunchCapability::full(),
            Some("com.google.Chrome"),
        );
        let cmd = build_command(&ctx, &[example_url()]).unwrap();
        // Must NOT use `open -b` — must use the direct executable path.
        assert_eq!(cmd.get_program().to_string_lossy(), "/bin/sh");
        let args = cmd
            .get_args()
            .map(|a| a.to_string_lossy().to_string())
            .collect::<Vec<_>>();
        assert!(
            args.contains(&"--profile-directory=Profile 1".to_string()),
            "args={args:?}"
        );
        assert!(args.contains(&example_url_str()));
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    #[test]
    fn macos_direct_exec_fallback() {
        let ctx = chrome_ctx(
            PathBuf::from("/bin/sh"),
            None,
            LaunchCapability::full(),
            None,
        );
        let cmd = build_command(&ctx, &[example_url()]).unwrap();
        assert_eq!(cmd.get_program().to_string_lossy(), "/bin/sh");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_direct_exec_fallback() {
        let ctx = chrome_ctx(
            PathBuf::from("/bin/sh"),
            None,
            LaunchCapability::full(),
            None,
        );
        let cmd = build_command(&ctx, &[example_url()]).unwrap();
        assert_eq!(cmd.get_program().to_string_lossy(), "/bin/sh");
    }

    #[cfg(unix)]
    #[test]
    fn other_browser_no_profile() {
        let browser = Browser::new("other", "Other", PathBuf::from("/bin/sh"))
            .with_variant(BrowserVariant::Single(BrowserFamily::Other));
        let ctx = BrowserContext::new(browser, None, LaunchCapability::full());
        let cmd = build_command(&ctx, &[example_url()]).unwrap();
        let args = cmd
            .get_args()
            .map(|a| a.to_string_lossy().to_string())
            .collect::<Vec<_>>();
        assert_eq!(args, vec![example_url_str()]);
    }

    #[test]
    fn not_launchable_returns_error() {
        let ctx = chrome_ctx(
            PathBuf::from("/bin/sh"),
            None,
            LaunchCapability {
                discoverable: true,
                launchable: false,
                profile_launchable: false,
                requires_user_config: false,
                limitations: vec!["nope".into()],
            },
            None,
        );
        let err = build_command(&ctx, &[example_url()]).unwrap_err();
        match err {
            LaunchError::NotLaunchable { limitations } => {
                assert_eq!(limitations, vec!["nope".to_string()]);
            }
            e => panic!("unexpected error: {e:?}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn empty_urls_returns_error() {
        let ctx = chrome_ctx(
            PathBuf::from("/bin/sh"),
            None,
            LaunchCapability::full(),
            None,
        );
        let err = build_command(&ctx, &[]).unwrap_err();
        assert!(matches!(err, LaunchError::EmptyUrls));
    }
}
