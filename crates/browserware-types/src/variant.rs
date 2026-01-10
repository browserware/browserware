//! Browser variant and channel definitions.
//!
//! Each browser family has its own set of release channels. This module provides
//! type-safe channel enums per browser family, plus a unified `BrowserVariant`
//! type that combines engine family with release channel.

use serde::{Deserialize, Serialize};

use crate::BrowserFamily;

/// Release channels for Chromium-based browsers (Chrome, Edge, Brave, Arc, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ChromiumChannel {
    /// Stable release channel
    #[default]
    Stable,
    /// Beta release channel
    Beta,
    /// Developer release channel
    Dev,
    /// Canary (bleeding edge) release channel
    Canary,
}

impl ChromiumChannel {
    /// Returns the canonical string name for this channel.
    #[must_use]
    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Beta => "beta",
            Self::Dev => "dev",
            Self::Canary => "canary",
        }
    }
}

impl std::fmt::Display for ChromiumChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.canonical_name())
    }
}

/// Release channels for Firefox-based browsers (Firefox, `LibreWolf`, Waterfox, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum FirefoxChannel {
    /// Stable release channel
    #[default]
    Stable,
    /// Beta release channel
    Beta,
    /// Developer Edition release channel
    Dev,
    /// Nightly (bleeding edge) release channel
    Nightly,
    /// Extended Support Release channel
    Esr,
}

impl FirefoxChannel {
    /// Returns the canonical string name for this channel.
    #[must_use]
    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Beta => "beta",
            Self::Dev => "dev",
            Self::Nightly => "nightly",
            Self::Esr => "esr",
        }
    }
}

impl std::fmt::Display for FirefoxChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.canonical_name())
    }
}

/// Release channels for WebKit-based browsers (Safari, GNOME Web, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum WebKitChannel {
    /// Stable release channel
    #[default]
    Stable,
    /// Safari Technology Preview
    TechnologyPreview,
}

impl WebKitChannel {
    /// Returns the canonical string name for this channel.
    #[must_use]
    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::TechnologyPreview => "technology-preview",
        }
    }
}

impl std::fmt::Display for WebKitChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.canonical_name())
    }
}

/// Browser variant combining engine family and release channel.
///
/// This enum provides a type-safe way to represent browser variants,
/// encoding both the engine family and the release channel in a single type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum BrowserVariant {
    /// Chromium-based browser with a specific release channel
    Chromium(ChromiumChannel),
    /// Firefox-based browser with a specific release channel
    Firefox(FirefoxChannel),
    /// WebKit-based browser with a specific release channel
    WebKit(WebKitChannel),
    /// Browser with a single release channel (e.g., Arc, GNOME Web).
    /// Carries the engine family for profile compatibility.
    Single(BrowserFamily),
}

impl BrowserVariant {
    /// Returns the browser engine family for this variant.
    #[must_use]
    pub const fn family(self) -> BrowserFamily {
        match self {
            Self::Chromium(_) => BrowserFamily::Chromium,
            Self::Firefox(_) => BrowserFamily::Firefox,
            Self::WebKit(_) => BrowserFamily::WebKit,
            Self::Single(family) => family,
        }
    }

    /// Returns the canonical string name for this channel.
    #[must_use]
    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::Chromium(c) => c.canonical_name(),
            Self::Firefox(c) => c.canonical_name(),
            Self::WebKit(c) => c.canonical_name(),
            Self::Single(_) => "stable",
        }
    }

    /// Creates a default Chromium channel (stable).
    #[must_use]
    pub const fn chromium_stable() -> Self {
        Self::Chromium(ChromiumChannel::Stable)
    }

    /// Creates a default Firefox channel (stable).
    #[must_use]
    pub const fn firefox_stable() -> Self {
        Self::Firefox(FirefoxChannel::Stable)
    }

    /// Creates a default `WebKit` channel (stable).
    #[must_use]
    pub const fn webkit_stable() -> Self {
        Self::WebKit(WebKitChannel::Stable)
    }

    /// Creates a single-channel browser with the given family.
    #[must_use]
    pub const fn single(family: BrowserFamily) -> Self {
        Self::Single(family)
    }
}

impl Default for BrowserVariant {
    fn default() -> Self {
        Self::Single(BrowserFamily::Other)
    }
}

impl std::fmt::Display for BrowserVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.canonical_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chromium_channel_canonical_names() {
        assert_eq!(ChromiumChannel::Stable.canonical_name(), "stable");
        assert_eq!(ChromiumChannel::Beta.canonical_name(), "beta");
        assert_eq!(ChromiumChannel::Dev.canonical_name(), "dev");
        assert_eq!(ChromiumChannel::Canary.canonical_name(), "canary");
    }

    #[test]
    fn firefox_channel_canonical_names() {
        assert_eq!(FirefoxChannel::Stable.canonical_name(), "stable");
        assert_eq!(FirefoxChannel::Beta.canonical_name(), "beta");
        assert_eq!(FirefoxChannel::Dev.canonical_name(), "dev");
        assert_eq!(FirefoxChannel::Nightly.canonical_name(), "nightly");
        assert_eq!(FirefoxChannel::Esr.canonical_name(), "esr");
    }

    #[test]
    fn webkit_channel_canonical_names() {
        assert_eq!(WebKitChannel::Stable.canonical_name(), "stable");
        assert_eq!(
            WebKitChannel::TechnologyPreview.canonical_name(),
            "technology-preview"
        );
    }

    #[test]
    fn browser_variant_canonical_names() {
        use BrowserVariant::*;

        assert_eq!(Chromium(ChromiumChannel::Canary).canonical_name(), "canary");
        assert_eq!(Firefox(FirefoxChannel::Nightly).canonical_name(), "nightly");
        assert_eq!(
            WebKit(WebKitChannel::TechnologyPreview).canonical_name(),
            "technology-preview"
        );
        assert_eq!(Single(BrowserFamily::Chromium).canonical_name(), "stable");
    }

    #[test]
    fn browser_variant_display() {
        assert_eq!(BrowserVariant::chromium_stable().to_string(), "stable");
        assert_eq!(
            BrowserVariant::Firefox(FirefoxChannel::Esr).to_string(),
            "esr"
        );
    }

    #[test]
    fn channel_serialization() {
        let channel = BrowserVariant::Chromium(ChromiumChannel::Beta);
        let json = serde_json::to_string(&channel).unwrap();
        let parsed: BrowserVariant = serde_json::from_str(&json).unwrap();
        assert_eq!(channel, parsed);
    }

    #[test]
    fn browser_variant_family() {
        use crate::BrowserFamily;
        use BrowserVariant::*;

        assert_eq!(
            Chromium(ChromiumChannel::Stable).family(),
            BrowserFamily::Chromium
        );
        assert_eq!(
            Firefox(FirefoxChannel::Nightly).family(),
            BrowserFamily::Firefox
        );
        assert_eq!(
            WebKit(WebKitChannel::TechnologyPreview).family(),
            BrowserFamily::WebKit
        );

        // Single-channel browsers preserve their family
        assert_eq!(
            Single(BrowserFamily::Chromium).family(),
            BrowserFamily::Chromium
        );
        assert_eq!(Single(BrowserFamily::Other).family(), BrowserFamily::Other);
    }
}
