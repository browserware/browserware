//! Browser context types for profile and launch capability tracking.
//!
//! This module provides types for representing a discovered browser context,
//! including profile information and capability flags for launching browsers.

use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::Browser;

/// A discovered browser profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileRef {
    /// Directory name (Chrome "Profile 1") or name (Firefox "work").
    pub id: String,
    /// Human-readable profile name (Chrome "Work", Firefox same as id).
    pub display_name: String,
}

/// Capability flags for a browser context.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaunchCapability {
    /// Whether the browser can be discovered automatically.
    pub discoverable: bool,
    /// Whether the browser can be launched.
    pub launchable: bool,
    /// Whether a specific profile can be targeted at launch.
    pub profile_launchable: bool,
    /// Whether the user must supply additional configuration to use this browser.
    pub requires_user_config: bool,
    /// Human-readable limitations for this browser context.
    pub limitations: Vec<String>,
}

impl LaunchCapability {
    /// Create a fully-capable context: all flags true, no limitations.
    #[must_use]
    pub const fn full() -> Self {
        Self {
            discoverable: true,
            launchable: true,
            profile_launchable: true,
            requires_user_config: false,
            limitations: Vec::new(),
        }
    }

    /// Create a launch-only context: discoverable and launchable, but profile
    /// launching is not supported. Attaches one limitation message.
    #[must_use]
    pub fn launch_only(limitation: impl Into<String>) -> Self {
        Self {
            discoverable: true,
            launchable: true,
            profile_launchable: false,
            requires_user_config: false,
            limitations: vec![limitation.into()],
        }
    }
}

/// Combines browser, optional profile, selector string, and capability flags.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserContext {
    /// The underlying browser.
    pub browser: Browser,
    /// The targeted profile, if any.
    pub profile: Option<ProfileRef>,
    /// Selector string, e.g. `"family=chromium,browser=chrome,profile=Profile 1"`.
    selector: String,
    /// Capability flags for this context.
    pub capability: LaunchCapability,
}

impl BrowserContext {
    /// Returns the canonical selector for this context.
    #[must_use]
    pub fn selector(&self) -> &str {
        &self.selector
    }

    /// Create a new `BrowserContext`, computing the selector automatically.
    ///
    /// The selector is `family=<family>,browser=<id>` optionally followed by
    /// `,profile=<profile.id>` when a profile is provided.
    #[must_use]
    pub fn new(
        browser: Browser,
        profile: Option<ProfileRef>,
        capability: LaunchCapability,
    ) -> Self {
        let selector = build_selector(&browser, profile.as_ref());
        Self {
            browser,
            profile,
            selector,
            capability,
        }
    }
}

/// Build the selector string from browser and optional profile.
fn build_selector(browser: &Browser, profile: Option<&ProfileRef>) -> String {
    let base = format!("family={},browser={}", browser.family(), browser.id);
    match profile {
        Some(p) => format!("{base},profile={}", encode_selector_value(&p.id)),
        None => base,
    }
}

#[allow(clippy::redundant_pub_crate)]
pub(crate) fn encode_selector_value(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());

    for ch in value.chars() {
        match ch {
            '%' | ',' | '=' => {
                let _ = write!(encoded, "%{:02X}", ch as u32);
            }
            _ => encoded.push(ch),
        }
    }

    encoded
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::variant::{BrowserVariant, ChromiumChannel, WebKitChannel};

    fn chrome_browser() -> Browser {
        Browser::new("chrome", "Google Chrome", PathBuf::from("/usr/bin/chrome"))
            .with_variant(BrowserVariant::Chromium(ChromiumChannel::Stable))
    }

    fn safari_browser() -> Browser {
        Browser::new(
            "safari",
            "Safari",
            PathBuf::from("/Applications/Safari.app"),
        )
        .with_variant(BrowserVariant::WebKit(WebKitChannel::Stable))
    }

    #[test]
    fn profile_ref_round_trip() {
        let profile = ProfileRef {
            id: "Profile 1".to_string(),
            display_name: "Work".to_string(),
        };
        let json = serde_json::to_string(&profile).unwrap();
        let parsed: ProfileRef = serde_json::from_str(&json).unwrap();
        assert_eq!(profile, parsed);
    }

    #[test]
    fn launch_capability_full() {
        let cap = LaunchCapability::full();
        assert!(cap.discoverable);
        assert!(cap.launchable);
        assert!(cap.profile_launchable);
        assert!(!cap.requires_user_config);
        assert!(cap.limitations.is_empty());
    }

    #[test]
    fn launch_capability_launch_only() {
        let cap = LaunchCapability::launch_only("Profile switching not supported");
        assert!(cap.discoverable);
        assert!(cap.launchable);
        assert!(!cap.profile_launchable);
        assert!(!cap.requires_user_config);
        assert_eq!(cap.limitations.len(), 1);
        assert_eq!(cap.limitations[0], "Profile switching not supported");
    }

    #[test]
    fn launch_capability_round_trip() {
        let cap = LaunchCapability {
            discoverable: true,
            launchable: true,
            profile_launchable: false,
            requires_user_config: true,
            limitations: vec!["needs config".to_string(), "no profiles".to_string()],
        };
        let json = serde_json::to_string(&cap).unwrap();
        let parsed: LaunchCapability = serde_json::from_str(&json).unwrap();
        assert_eq!(cap, parsed);
    }

    #[test]
    fn browser_context_selector_with_profile() {
        let profile = ProfileRef {
            id: "Profile 1".to_string(),
            display_name: "Work".to_string(),
        };
        let ctx = BrowserContext::new(chrome_browser(), Some(profile), LaunchCapability::full());
        assert_eq!(
            ctx.selector(),
            "family=chromium,browser=chrome,profile=Profile 1"
        );
    }

    #[test]
    fn browser_context_selector_no_profile() {
        let ctx = BrowserContext::new(chrome_browser(), None, LaunchCapability::full());
        assert_eq!(ctx.selector(), "family=chromium,browser=chrome");
    }

    #[test]
    fn browser_context_selector_escapes_reserved_profile_chars() {
        let profile = ProfileRef {
            id: "work,alpha=1%".to_string(),
            display_name: "Work".to_string(),
        };
        let ctx = BrowserContext::new(chrome_browser(), Some(profile), LaunchCapability::full());
        assert_eq!(
            ctx.selector(),
            "family=chromium,browser=chrome,profile=work%2Calpha%3D1%25"
        );
    }

    #[test]
    fn browser_context_round_trip() {
        let profile = ProfileRef {
            id: "Default".to_string(),
            display_name: "Default".to_string(),
        };
        let ctx = BrowserContext::new(chrome_browser(), Some(profile), LaunchCapability::full());
        let json = serde_json::to_string(&ctx).unwrap();
        let parsed: BrowserContext = serde_json::from_str(&json).unwrap();
        assert_eq!(ctx, parsed);
    }

    #[test]
    fn browser_context_with_limitations_round_trip() {
        let cap = LaunchCapability::launch_only("Safari does not support profile flags");
        let ctx = BrowserContext::new(safari_browser(), None, cap);
        let json = serde_json::to_string(&ctx).unwrap();
        let parsed: BrowserContext = serde_json::from_str(&json).unwrap();
        assert_eq!(ctx, parsed);
    }
}
