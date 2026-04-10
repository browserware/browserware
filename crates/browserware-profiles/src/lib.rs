//! Browser profile management for the browserware ecosystem.
//!
//! This crate provides profile discovery for Chrome-family and Firefox browsers.
//! Additional browser families will be added in future milestones.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod chrome;
mod firefox;

use std::path::PathBuf;

use browserware_types::{Browser, BrowserFamily, BrowserVariant, LaunchCapability};

pub use chrome::discover_chrome_profiles_from;
pub use firefox::discover_firefox_profiles_from;

/// The result of profile discovery for a single browser.
#[derive(Debug, Clone)]
pub struct ProfileDiscovery {
    /// Discovered profiles. Empty when the browser has no profile support or
    /// when metadata is inaccessible.
    pub profiles: Vec<browserware_types::ProfileRef>,
    /// Capability flags derived from the discovery result.
    pub capability: browserware_types::LaunchCapability,
}

/// Discover profiles for the given browser.
///
/// Dispatches to the correct discovery backend based on the browser's variant,
/// using platform-specific data directory paths. Returns a [`ProfileDiscovery`]
/// with all found profiles and capability flags.
#[must_use]
pub fn discover_profiles(browser: &Browser) -> ProfileDiscovery {
    match browser.variant {
        BrowserVariant::Chromium(_) | BrowserVariant::Single(BrowserFamily::Chromium) => {
            chrome_user_data_dir(browser).map_or_else(
                || ProfileDiscovery {
                    profiles: vec![],
                    capability: LaunchCapability::launch_only(
                        "Chrome user data directory could not be located",
                    ),
                },
                |dir| chrome::discover_chrome_profiles_from(&dir),
            )
        }
        BrowserVariant::Firefox(_) | BrowserVariant::Single(BrowserFamily::Firefox) => {
            firefox_profiles_ini(browser).map_or_else(
                || ProfileDiscovery {
                    profiles: vec![],
                    capability: LaunchCapability::launch_only(
                        "Firefox profiles.ini could not be located",
                    ),
                },
                |ini| firefox::discover_firefox_profiles_from(&ini),
            )
        }
        BrowserVariant::WebKit(_) | BrowserVariant::Single(BrowserFamily::WebKit) => {
            ProfileDiscovery {
                profiles: vec![],
                capability: LaunchCapability::launch_only(
                    "WebKit/Safari does not support profile-specific launch",
                ),
            }
        }
        BrowserVariant::Single(BrowserFamily::Other) => ProfileDiscovery {
            profiles: vec![],
            capability: LaunchCapability::launch_only(
                "Profile launch not supported for this browser",
            ),
        },
    }
}

/// Returns the Chrome-family user data directory path for the given browser,
/// or `None` if the browser ID is unrecognised on this platform.
fn chrome_user_data_dir(browser: &Browser) -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let base = home_dir()?.join("Library/Application Support");
        let dir_name = match browser.id.0.as_str() {
            "chrome" => "Google/Chrome",
            "chrome-beta" => "Google/Chrome Beta",
            "chrome-dev" => "Google/Chrome Dev",
            "chrome-canary" => "Google/Chrome Canary",
            "edge" => "Microsoft Edge",
            "edge-beta" => "Microsoft Edge Beta",
            "edge-dev" => "Microsoft Edge Dev",
            "edge-canary" => "Microsoft Edge Canary",
            "brave" => "BraveSoftware/Brave-Browser",
            "brave-beta" => "BraveSoftware/Brave-Browser-Beta",
            "chromium" => "Chromium",
            "vivaldi" => "Vivaldi",
            "opera" => "com.operasoftware.Opera",
            _ => return None,
        };
        Some(base.join(dir_name))
    }

    #[cfg(target_os = "linux")]
    {
        let base = home_dir()?.join(".config");
        let dir_name = match browser.id.0.as_str() {
            "chrome" | "chrome-beta" | "chrome-dev" => "google-chrome",
            "chrome-canary" => "google-chrome-unstable",
            "edge" => "microsoft-edge",
            "edge-beta" => "microsoft-edge-beta",
            "edge-dev" => "microsoft-edge-dev",
            "brave" => "BraveSoftware/Brave-Browser",
            "chromium" => "chromium",
            "vivaldi" => "vivaldi",
            _ => return None,
        };
        Some(base.join(dir_name))
    }

    #[cfg(target_os = "windows")]
    {
        let base = home_dir()?.join("AppData/Local");
        let dir_name = match browser.id.0.as_str() {
            "chrome" => "Google/Chrome/User Data",
            "chrome-beta" => "Google/Chrome Beta/User Data",
            "chrome-dev" => "Google/Chrome Dev/User Data",
            "chrome-canary" => "Google/Chrome SxS/User Data",
            "edge" => "Microsoft/Edge/User Data",
            "edge-beta" => "Microsoft/Edge Beta/User Data",
            "edge-dev" => "Microsoft/Edge Dev/User Data",
            "edge-canary" => "Microsoft/Edge SxS/User Data",
            "brave" => "BraveSoftware/Brave-Browser/User Data",
            "chromium" => "Chromium/User Data",
            _ => return None,
        };
        Some(base.join(dir_name))
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        let _ = browser;
        None
    }
}

/// Returns the Firefox-family `profiles.ini` path for the given browser,
/// or `None` if the browser ID is unrecognised on this platform.
fn firefox_profiles_ini(browser: &Browser) -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let base = home_dir()?.join("Library/Application Support");
        let file = match browser.id.0.as_str() {
            "firefox" | "firefox-beta" | "firefox-dev" | "firefox-nightly" | "firefox-esr" => {
                "Firefox/profiles.ini"
            }
            "librewolf" => "librewolf/profiles.ini",
            "waterfox" => "Waterfox/profiles.ini",
            "floorp" => "Floorp/profiles.ini",
            _ => return None,
        };
        Some(base.join(file))
    }

    #[cfg(target_os = "linux")]
    {
        let base = home_dir()?.join(".mozilla");
        let file = match browser.id.0.as_str() {
            "firefox" | "firefox-beta" | "firefox-dev" | "firefox-nightly" | "firefox-esr" => {
                "firefox/profiles.ini"
            }
            "librewolf" => "librewolf/profiles.ini",
            "waterfox" => "waterfox/profiles.ini",
            "floorp" => "floorp/profiles.ini",
            _ => return None,
        };
        Some(base.join(file))
    }

    #[cfg(target_os = "windows")]
    {
        let base = home_dir()?.join("AppData/Roaming");
        let file = match browser.id.0.as_str() {
            "firefox" | "firefox-beta" | "firefox-dev" | "firefox-nightly" | "firefox-esr" => {
                "Mozilla/Firefox/profiles.ini"
            }
            "librewolf" => "librewolf/librewolf/profiles.ini",
            "waterfox" => "Waterfox/profiles.ini",
            _ => return None,
        };
        Some(base.join(file))
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        let _ = browser;
        None
    }
}

/// Returns the user's home directory path.
///
/// Reads `HOME` on Unix-like systems and falls back to `USERPROFILE` on Windows.
fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from).or({
        #[cfg(target_os = "windows")]
        {
            std::env::var("USERPROFILE").ok().map(PathBuf::from)
        }
        #[cfg(not(target_os = "windows"))]
        {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use browserware_types::{Browser, BrowserFamily, BrowserVariant, WebKitChannel};

    use super::discover_profiles;

    #[test]
    fn discover_profiles_webkit_returns_limitation() {
        let browser =
            Browser::new("safari", "Safari", PathBuf::from("/Applications/Safari.app"))
                .with_variant(BrowserVariant::WebKit(WebKitChannel::Stable));
        let d = discover_profiles(&browser);
        assert!(!d.capability.profile_launchable);
        assert!(d.profiles.is_empty());
        assert!(!d.capability.limitations.is_empty());
    }

    #[test]
    fn discover_profiles_other_returns_limitation() {
        let browser =
            Browser::new("unknown", "Unknown Browser", PathBuf::from("/usr/bin/unknown"))
                .with_variant(BrowserVariant::Single(BrowserFamily::Other));
        let d = discover_profiles(&browser);
        assert!(!d.capability.profile_launchable);
        assert!(d.profiles.is_empty());
    }
}
