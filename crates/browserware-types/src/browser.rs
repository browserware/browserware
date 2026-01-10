//! Browser type definitions.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::variant::BrowserVariant;

/// Unique identifier for a browser installation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BrowserId(pub String);

impl BrowserId {
    /// Create a new browser ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for BrowserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Browser engine family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum BrowserFamily {
    /// Chromium-based browsers (Chrome, Edge, Brave, Arc, etc.)
    Chromium,
    /// Firefox-based browsers (Firefox, `LibreWolf`, Waterfox, etc.)
    Firefox,
    /// WebKit-based browsers (Safari, GNOME Web, etc.)
    WebKit,
    /// Unknown or other browser engines
    #[default]
    Other,
}

impl std::fmt::Display for BrowserFamily {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Chromium => "chromium",
            Self::Firefox => "firefox",
            Self::WebKit => "webkit",
            Self::Other => "other",
        };
        f.write_str(name)
    }
}

/// Information about an installed browser.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Browser {
    /// Unique identifier for this browser
    pub id: BrowserId,
    /// Display name
    pub name: String,
    /// Browser variant (engine family + release channel)
    pub variant: BrowserVariant,
    /// Version string (if available)
    pub version: Option<String>,
    /// Path to the browser executable
    pub executable: PathBuf,
    /// Bundle identifier (macOS)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundle_id: Option<String>,
}

impl Browser {
    /// Create a new browser with minimal information.
    pub fn new(id: impl Into<String>, name: impl Into<String>, executable: PathBuf) -> Self {
        Self {
            id: BrowserId::new(id),
            name: name.into(),
            variant: BrowserVariant::default(),
            version: None,
            executable,
            bundle_id: None,
        }
    }

    /// Returns the browser engine family, derived from the variant.
    #[must_use]
    pub const fn family(&self) -> BrowserFamily {
        self.variant.family()
    }

    /// Set the browser variant.
    #[must_use]
    pub const fn with_variant(mut self, variant: BrowserVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Set the browser version.
    #[must_use]
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set the bundle ID (macOS).
    #[must_use]
    pub fn with_bundle_id(mut self, bundle_id: impl Into<String>) -> Self {
        self.bundle_id = Some(bundle_id.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::variant::{ChromiumChannel, FirefoxChannel};

    #[test]
    fn browser_id_display() {
        let id = BrowserId::new("chrome");
        assert_eq!(id.to_string(), "chrome");
    }

    #[test]
    fn browser_family_display() {
        assert_eq!(BrowserFamily::Chromium.to_string(), "chromium");
        assert_eq!(BrowserFamily::Firefox.to_string(), "firefox");
    }

    #[test]
    fn browser_builder_pattern() {
        let browser = Browser::new("chrome", "Google Chrome", "/usr/bin/chrome".into())
            .with_variant(BrowserVariant::Chromium(ChromiumChannel::Stable))
            .with_version("120.0.0");

        assert_eq!(browser.name, "Google Chrome");
        assert_eq!(browser.family(), BrowserFamily::Chromium);
        assert_eq!(browser.version, Some("120.0.0".to_string()));
    }

    #[test]
    fn browser_family_derived_from_variant() {
        let chrome = Browser::new("chrome", "Chrome", "/usr/bin/chrome".into())
            .with_variant(BrowserVariant::Chromium(ChromiumChannel::Canary));
        assert_eq!(chrome.family(), BrowserFamily::Chromium);

        let firefox = Browser::new("firefox", "Firefox", "/usr/bin/firefox".into())
            .with_variant(BrowserVariant::Firefox(FirefoxChannel::Nightly));
        assert_eq!(firefox.family(), BrowserFamily::Firefox);

        // Arc is a single-channel Chromium-based browser
        let arc = Browser::new("arc", "Arc", "/Applications/Arc.app".into())
            .with_variant(BrowserVariant::Single(BrowserFamily::Chromium));
        assert_eq!(arc.family(), BrowserFamily::Chromium);

        let other = Browser::new("other", "Other", "/usr/bin/other".into())
            .with_variant(BrowserVariant::Single(BrowserFamily::Other));
        assert_eq!(other.family(), BrowserFamily::Other);
    }

    #[test]
    fn browser_serialization() {
        let browser = Browser::new("firefox", "Firefox", "/usr/bin/firefox".into())
            .with_variant(BrowserVariant::Firefox(FirefoxChannel::Stable));

        let json = serde_json::to_string(&browser).unwrap();
        let parsed: Browser = serde_json::from_str(&json).unwrap();

        assert_eq!(browser, parsed);
    }
}
