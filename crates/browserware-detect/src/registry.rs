//! Known browser registry for metadata enrichment.
//!
//! This module provides static metadata about known browsers, used to enrich
//! browsers discovered through platform APIs with canonical IDs, display names,
//! and variant information.
//!
//! # Design
//!
//! The registry is **not** the source of browsers to detect. Platform detection
//! enumerates all registered URL handlers, then matches against this registry
//! for metadata enrichment. Unknown browsers still get detected with derived
//! metadata.

use browserware_types::{
    BrowserFamily, BrowserVariant, ChromiumChannel, FirefoxChannel, WebKitChannel,
};

/// Static metadata for a known browser.
///
/// This struct contains compile-time information about browsers we know
/// how to identify. Each entry describes platform-specific identifiers used
/// to match detected browsers to their canonical metadata.
#[derive(Debug, Clone, Copy)]
pub struct BrowserMeta {
    /// Canonical identifier used in configurations (e.g., "chrome", "firefox-nightly").
    ///
    /// This ID is stable across platforms and used in rule files.
    pub id: &'static str,

    /// Human-readable display name (e.g., "Google Chrome", "Firefox Nightly").
    pub name: &'static str,

    /// Browser variant encoding engine family and release channel.
    pub variant: BrowserVariant,

    /// macOS bundle identifiers (`CFBundleIdentifier` from Info.plist).
    ///
    /// Multiple entries handle renamed bundles or alternative distributions.
    /// Empty slice indicates the browser is not available on macOS.
    pub macos_bundle_ids: &'static [&'static str],

    /// Windows registry key names under `HKLM\SOFTWARE\Clients\StartMenuInternet`.
    ///
    /// Empty slice indicates the browser is not available on Windows.
    pub windows_registry_keys: &'static [&'static str],

    /// Linux desktop file basenames (without `.desktop` extension).
    ///
    /// Includes both native package names and Flatpak/Snap identifiers.
    /// Empty slice indicates the browser is not available on Linux.
    pub linux_desktop_ids: &'static [&'static str],
}

impl BrowserMeta {
    /// Returns true if this browser is available on macOS.
    #[must_use]
    pub const fn available_on_macos(&self) -> bool {
        !self.macos_bundle_ids.is_empty()
    }

    /// Returns true if this browser is available on Windows.
    #[must_use]
    pub const fn available_on_windows(&self) -> bool {
        !self.windows_registry_keys.is_empty()
    }

    /// Returns true if this browser is available on Linux.
    #[must_use]
    pub const fn available_on_linux(&self) -> bool {
        !self.linux_desktop_ids.is_empty()
    }

    /// Returns the browser engine family.
    #[must_use]
    pub const fn family(&self) -> BrowserFamily {
        self.variant.family()
    }
}

/// Registry of known browsers with their platform-specific identifiers.
///
/// This static array contains metadata for all browsers that browserware
/// knows how to identify. Platform implementations use this registry to
/// match detected installations to known browser metadata.
pub static KNOWN_BROWSERS: &[BrowserMeta] = &[
    // =========================================================================
    // CHROMIUM FAMILY - Google Chrome
    // =========================================================================
    BrowserMeta {
        id: "chrome",
        name: "Google Chrome",
        variant: BrowserVariant::Chromium(ChromiumChannel::Stable),
        macos_bundle_ids: &["com.google.Chrome"],
        windows_registry_keys: &["Google Chrome"],
        linux_desktop_ids: &["google-chrome", "google-chrome-stable"],
    },
    BrowserMeta {
        id: "chrome-beta",
        name: "Google Chrome Beta",
        variant: BrowserVariant::Chromium(ChromiumChannel::Beta),
        macos_bundle_ids: &["com.google.Chrome.beta"],
        windows_registry_keys: &["Google Chrome Beta"],
        linux_desktop_ids: &["google-chrome-beta"],
    },
    BrowserMeta {
        id: "chrome-dev",
        name: "Google Chrome Dev",
        variant: BrowserVariant::Chromium(ChromiumChannel::Dev),
        macos_bundle_ids: &["com.google.Chrome.dev"],
        windows_registry_keys: &["Google Chrome Dev"],
        linux_desktop_ids: &["google-chrome-unstable"],
    },
    BrowserMeta {
        id: "chrome-canary",
        name: "Google Chrome Canary",
        variant: BrowserVariant::Chromium(ChromiumChannel::Canary),
        macos_bundle_ids: &["com.google.Chrome.canary"],
        windows_registry_keys: &["Google Chrome Canary"],
        linux_desktop_ids: &[], // Canary not available on Linux
    },
    // =========================================================================
    // CHROMIUM FAMILY - Microsoft Edge
    // =========================================================================
    BrowserMeta {
        id: "edge",
        name: "Microsoft Edge",
        variant: BrowserVariant::Chromium(ChromiumChannel::Stable),
        macos_bundle_ids: &["com.microsoft.edgemac"],
        windows_registry_keys: &["Microsoft Edge"],
        linux_desktop_ids: &["microsoft-edge", "microsoft-edge-stable"],
    },
    BrowserMeta {
        id: "edge-beta",
        name: "Microsoft Edge Beta",
        variant: BrowserVariant::Chromium(ChromiumChannel::Beta),
        macos_bundle_ids: &["com.microsoft.edgemac.Beta"],
        windows_registry_keys: &["Microsoft Edge Beta"],
        linux_desktop_ids: &["microsoft-edge-beta"],
    },
    BrowserMeta {
        id: "edge-dev",
        name: "Microsoft Edge Dev",
        variant: BrowserVariant::Chromium(ChromiumChannel::Dev),
        macos_bundle_ids: &["com.microsoft.edgemac.Dev"],
        windows_registry_keys: &["Microsoft Edge Dev"],
        linux_desktop_ids: &["microsoft-edge-dev"],
    },
    BrowserMeta {
        id: "edge-canary",
        name: "Microsoft Edge Canary",
        variant: BrowserVariant::Chromium(ChromiumChannel::Canary),
        macos_bundle_ids: &["com.microsoft.edgemac.Canary"],
        windows_registry_keys: &["Microsoft Edge Canary"],
        linux_desktop_ids: &[], // Canary not available on Linux
    },
    // =========================================================================
    // CHROMIUM FAMILY - Brave
    // =========================================================================
    BrowserMeta {
        id: "brave",
        name: "Brave Browser",
        variant: BrowserVariant::Chromium(ChromiumChannel::Stable),
        macos_bundle_ids: &["com.brave.Browser"],
        windows_registry_keys: &["BraveSoftware Brave-Browser"],
        linux_desktop_ids: &["brave-browser", "brave"],
    },
    BrowserMeta {
        id: "brave-beta",
        name: "Brave Browser Beta",
        variant: BrowserVariant::Chromium(ChromiumChannel::Beta),
        macos_bundle_ids: &["com.brave.Browser.beta"],
        windows_registry_keys: &["BraveSoftware Brave-Browser-Beta"],
        linux_desktop_ids: &["brave-browser-beta"],
    },
    BrowserMeta {
        id: "brave-nightly",
        name: "Brave Browser Nightly",
        variant: BrowserVariant::Chromium(ChromiumChannel::Canary),
        macos_bundle_ids: &["com.brave.Browser.nightly"],
        windows_registry_keys: &["BraveSoftware Brave-Browser-Nightly"],
        linux_desktop_ids: &["brave-browser-nightly"],
    },
    // =========================================================================
    // CHROMIUM FAMILY - Arc (Single channel)
    // =========================================================================
    BrowserMeta {
        id: "arc",
        name: "Arc",
        variant: BrowserVariant::Single(BrowserFamily::Chromium),
        macos_bundle_ids: &["company.thebrowser.Browser"],
        windows_registry_keys: &["Arc"],
        linux_desktop_ids: &[], // Not available on Linux
    },
    // =========================================================================
    // CHROMIUM FAMILY - Vivaldi
    // =========================================================================
    BrowserMeta {
        id: "vivaldi",
        name: "Vivaldi",
        variant: BrowserVariant::Chromium(ChromiumChannel::Stable),
        macos_bundle_ids: &["com.vivaldi.Vivaldi"],
        windows_registry_keys: &["Vivaldi"],
        linux_desktop_ids: &["vivaldi", "vivaldi-stable"],
    },
    BrowserMeta {
        id: "vivaldi-snapshot",
        name: "Vivaldi Snapshot",
        variant: BrowserVariant::Chromium(ChromiumChannel::Dev),
        macos_bundle_ids: &["com.vivaldi.Vivaldi.snapshot"],
        windows_registry_keys: &["Vivaldi Snapshot"],
        linux_desktop_ids: &["vivaldi-snapshot"],
    },
    // =========================================================================
    // CHROMIUM FAMILY - Opera
    // =========================================================================
    BrowserMeta {
        id: "opera",
        name: "Opera",
        variant: BrowserVariant::Chromium(ChromiumChannel::Stable),
        macos_bundle_ids: &["com.operasoftware.Opera"],
        windows_registry_keys: &["Opera Stable"],
        linux_desktop_ids: &["opera"],
    },
    BrowserMeta {
        id: "opera-beta",
        name: "Opera Beta",
        variant: BrowserVariant::Chromium(ChromiumChannel::Beta),
        macos_bundle_ids: &["com.operasoftware.OperaNext"],
        windows_registry_keys: &["Opera Beta"],
        linux_desktop_ids: &["opera-beta"],
    },
    BrowserMeta {
        id: "opera-developer",
        name: "Opera Developer",
        variant: BrowserVariant::Chromium(ChromiumChannel::Dev),
        macos_bundle_ids: &["com.operasoftware.OperaDeveloper"],
        windows_registry_keys: &["Opera Developer"],
        linux_desktop_ids: &["opera-developer"],
    },
    BrowserMeta {
        id: "opera-gx",
        name: "Opera GX",
        variant: BrowserVariant::Single(BrowserFamily::Chromium),
        macos_bundle_ids: &["com.operasoftware.OperaGX"],
        windows_registry_keys: &["Opera GX Stable"],
        linux_desktop_ids: &[], // Not available on Linux
    },
    // =========================================================================
    // CHROMIUM FAMILY - Chromium (open source)
    // =========================================================================
    BrowserMeta {
        id: "chromium",
        name: "Chromium",
        variant: BrowserVariant::Chromium(ChromiumChannel::Stable),
        macos_bundle_ids: &["org.chromium.Chromium"],
        windows_registry_keys: &["Chromium"],
        linux_desktop_ids: &["chromium", "chromium-browser"],
    },
    // =========================================================================
    // FIREFOX FAMILY - Mozilla Firefox
    // =========================================================================
    BrowserMeta {
        id: "firefox",
        name: "Firefox",
        variant: BrowserVariant::Firefox(FirefoxChannel::Stable),
        macos_bundle_ids: &["org.mozilla.firefox"],
        windows_registry_keys: &["Firefox"],
        linux_desktop_ids: &["firefox"],
    },
    BrowserMeta {
        id: "firefox-beta",
        name: "Firefox Beta",
        variant: BrowserVariant::Firefox(FirefoxChannel::Beta),
        macos_bundle_ids: &["org.mozilla.firefoxbeta"],
        windows_registry_keys: &["Firefox Beta"],
        linux_desktop_ids: &["firefox-beta"],
    },
    BrowserMeta {
        id: "firefox-dev",
        name: "Firefox Developer Edition",
        variant: BrowserVariant::Firefox(FirefoxChannel::Dev),
        macos_bundle_ids: &["org.mozilla.firefoxdeveloperedition"],
        windows_registry_keys: &["Firefox Developer Edition"],
        linux_desktop_ids: &["firefox-developer-edition", "firefoxdeveloperedition"],
    },
    BrowserMeta {
        id: "firefox-nightly",
        name: "Firefox Nightly",
        variant: BrowserVariant::Firefox(FirefoxChannel::Nightly),
        macos_bundle_ids: &["org.mozilla.nightly"],
        windows_registry_keys: &["Firefox Nightly"],
        linux_desktop_ids: &["firefox-nightly"],
    },
    BrowserMeta {
        id: "firefox-esr",
        name: "Firefox ESR",
        variant: BrowserVariant::Firefox(FirefoxChannel::Esr),
        macos_bundle_ids: &["org.mozilla.firefoxesr"],
        windows_registry_keys: &["Firefox ESR"],
        linux_desktop_ids: &["firefox-esr"],
    },
    // =========================================================================
    // FIREFOX FAMILY - LibreWolf (privacy-focused fork)
    // =========================================================================
    BrowserMeta {
        id: "librewolf",
        name: "LibreWolf",
        variant: BrowserVariant::Single(BrowserFamily::Firefox),
        macos_bundle_ids: &["io.gitlab.LibreWolf"],
        windows_registry_keys: &["LibreWolf"],
        linux_desktop_ids: &["librewolf", "io.gitlab.librewolf"],
    },
    // =========================================================================
    // FIREFOX FAMILY - Waterfox
    // =========================================================================
    BrowserMeta {
        id: "waterfox",
        name: "Waterfox",
        variant: BrowserVariant::Single(BrowserFamily::Firefox),
        macos_bundle_ids: &["net.waterfox.waterfox"],
        windows_registry_keys: &["Waterfox"],
        linux_desktop_ids: &["waterfox", "waterfox-current"],
    },
    // =========================================================================
    // FIREFOX FAMILY - Floorp
    // =========================================================================
    BrowserMeta {
        id: "floorp",
        name: "Floorp",
        variant: BrowserVariant::Single(BrowserFamily::Firefox),
        macos_bundle_ids: &["one.ablaze.floorp"],
        windows_registry_keys: &["Floorp"],
        linux_desktop_ids: &["floorp", "one.ablaze.floorp"],
    },
    // =========================================================================
    // WEBKIT FAMILY - Safari (macOS only)
    // =========================================================================
    BrowserMeta {
        id: "safari",
        name: "Safari",
        variant: BrowserVariant::WebKit(WebKitChannel::Stable),
        macos_bundle_ids: &["com.apple.Safari"],
        windows_registry_keys: &[], // Discontinued on Windows
        linux_desktop_ids: &[],     // Never available on Linux
    },
    BrowserMeta {
        id: "safari-preview",
        name: "Safari Technology Preview",
        variant: BrowserVariant::WebKit(WebKitChannel::TechnologyPreview),
        macos_bundle_ids: &["com.apple.SafariTechnologyPreview"],
        windows_registry_keys: &[],
        linux_desktop_ids: &[],
    },
    // =========================================================================
    // WEBKIT FAMILY - GNOME Web (Linux only)
    // =========================================================================
    BrowserMeta {
        id: "gnome-web",
        name: "GNOME Web",
        variant: BrowserVariant::Single(BrowserFamily::WebKit),
        macos_bundle_ids: &[],
        windows_registry_keys: &[],
        linux_desktop_ids: &["org.gnome.Epiphany", "epiphany", "epiphany-browser"],
    },
];

/// Find browser metadata by canonical ID.
///
/// # Arguments
///
/// * `id` - The browser's canonical identifier (e.g., "chrome", "firefox-nightly")
///
/// # Returns
///
/// The browser metadata if found, `None` otherwise.
///
/// # Example
///
/// ```
/// use browserware_detect::registry::find_by_id;
///
/// if let Some(chrome) = find_by_id("chrome") {
///     assert_eq!(chrome.name, "Google Chrome");
/// }
/// ```
#[must_use]
pub fn find_by_id(id: &str) -> Option<&'static BrowserMeta> {
    KNOWN_BROWSERS.iter().find(|meta| meta.id == id)
}

/// Find browser metadata by macOS bundle identifier.
///
/// # Arguments
///
/// * `bundle_id` - The macOS bundle identifier (e.g., "com.google.Chrome")
///
/// # Returns
///
/// The browser metadata if found, `None` otherwise.
///
/// # Example
///
/// ```
/// use browserware_detect::registry::find_by_bundle_id;
///
/// if let Some(chrome) = find_by_bundle_id("com.google.Chrome") {
///     assert_eq!(chrome.id, "chrome");
/// }
/// ```
#[must_use]
pub fn find_by_bundle_id(bundle_id: &str) -> Option<&'static BrowserMeta> {
    KNOWN_BROWSERS
        .iter()
        .find(|meta| meta.macos_bundle_ids.contains(&bundle_id))
}

/// Find browser metadata by Windows registry key.
///
/// # Arguments
///
/// * `key` - The Windows registry key name (e.g., "Google Chrome")
///
/// # Returns
///
/// The browser metadata if found, `None` otherwise.
///
/// # Example
///
/// ```
/// use browserware_detect::registry::find_by_registry_key;
///
/// if let Some(chrome) = find_by_registry_key("Google Chrome") {
///     assert_eq!(chrome.id, "chrome");
/// }
/// ```
#[must_use]
pub fn find_by_registry_key(key: &str) -> Option<&'static BrowserMeta> {
    KNOWN_BROWSERS
        .iter()
        .find(|meta| meta.windows_registry_keys.contains(&key))
}

/// Find browser metadata by Linux desktop ID.
///
/// # Arguments
///
/// * `desktop_id` - The desktop file basename without extension (e.g., "firefox")
///
/// # Returns
///
/// The browser metadata if found, `None` otherwise.
///
/// # Example
///
/// ```
/// use browserware_detect::registry::find_by_desktop_id;
///
/// if let Some(firefox) = find_by_desktop_id("firefox") {
///     assert_eq!(firefox.id, "firefox");
/// }
/// ```
#[must_use]
pub fn find_by_desktop_id(desktop_id: &str) -> Option<&'static BrowserMeta> {
    KNOWN_BROWSERS
        .iter()
        .find(|meta| meta.linux_desktop_ids.contains(&desktop_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_browsers_not_empty() {
        assert!(!KNOWN_BROWSERS.is_empty());
    }

    #[test]
    fn all_browsers_have_unique_ids() {
        let mut ids: Vec<&str> = KNOWN_BROWSERS.iter().map(|m| m.id).collect();
        ids.sort_unstable();
        let original_len = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), original_len, "Duplicate browser IDs found");
    }

    #[test]
    fn all_browsers_have_at_least_one_platform() {
        for meta in KNOWN_BROWSERS {
            assert!(
                meta.available_on_macos()
                    || meta.available_on_windows()
                    || meta.available_on_linux(),
                "Browser '{}' has no platform identifiers",
                meta.id
            );
        }
    }

    #[test]
    fn find_chrome_by_id() {
        let chrome = find_by_id("chrome");
        assert!(chrome.is_some());
        assert_eq!(chrome.unwrap().name, "Google Chrome");
    }

    #[test]
    fn find_chrome_by_bundle_id() {
        let chrome = find_by_bundle_id("com.google.Chrome");
        assert!(chrome.is_some());
        assert_eq!(chrome.unwrap().id, "chrome");
    }

    #[test]
    fn find_chrome_by_registry_key() {
        let chrome = find_by_registry_key("Google Chrome");
        assert!(chrome.is_some());
        assert_eq!(chrome.unwrap().id, "chrome");
    }

    #[test]
    fn find_firefox_by_desktop_id() {
        let firefox = find_by_desktop_id("firefox");
        assert!(firefox.is_some());
        assert_eq!(firefox.unwrap().id, "firefox");
    }

    #[test]
    fn find_nonexistent_returns_none() {
        assert!(find_by_id("nonexistent-browser").is_none());
        assert!(find_by_bundle_id("com.nonexistent.browser").is_none());
        assert!(find_by_registry_key("Nonexistent Browser").is_none());
        assert!(find_by_desktop_id("nonexistent-browser").is_none());
    }

    #[test]
    fn browser_families_correct() {
        let chrome = find_by_id("chrome").unwrap();
        assert_eq!(chrome.family(), BrowserFamily::Chromium);

        let firefox = find_by_id("firefox").unwrap();
        assert_eq!(firefox.family(), BrowserFamily::Firefox);

        let safari = find_by_id("safari").unwrap();
        assert_eq!(safari.family(), BrowserFamily::WebKit);

        let arc = find_by_id("arc").unwrap();
        assert_eq!(arc.family(), BrowserFamily::Chromium);
    }

    #[test]
    fn safari_only_on_macos() {
        let safari = find_by_id("safari").unwrap();
        assert!(safari.available_on_macos());
        assert!(!safari.available_on_windows());
        assert!(!safari.available_on_linux());
    }

    #[test]
    fn gnome_web_only_on_linux() {
        let gnome_web = find_by_id("gnome-web").unwrap();
        assert!(!gnome_web.available_on_macos());
        assert!(!gnome_web.available_on_windows());
        assert!(gnome_web.available_on_linux());
    }

    #[test]
    fn chrome_on_all_platforms() {
        let chrome = find_by_id("chrome").unwrap();
        assert!(chrome.available_on_macos());
        assert!(chrome.available_on_windows());
        assert!(chrome.available_on_linux());
    }

    #[test]
    fn alternative_desktop_ids_work() {
        // Chrome has multiple desktop IDs
        let by_google_chrome = find_by_desktop_id("google-chrome");
        let by_google_chrome_stable = find_by_desktop_id("google-chrome-stable");
        assert!(by_google_chrome.is_some());
        assert!(by_google_chrome_stable.is_some());
        assert_eq!(
            by_google_chrome.unwrap().id,
            by_google_chrome_stable.unwrap().id
        );
    }
}
