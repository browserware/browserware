//! Browser detection for the browserware ecosystem.
//!
//! This crate provides cross-platform browser discovery, detecting installed
//! browsers, their versions, and the system's default browser.
//!
//! # Overview
//!
//! The detection process uses platform-specific APIs:
//! - **macOS**: Launch Services and Info.plist parsing
//! - **Windows**: Registry enumeration under `StartMenuInternet`
//! - **Linux**: XDG desktop file scanning
//!
//! All detected browsers are matched against a known browser registry to
//! provide consistent metadata (IDs, display names, engine families).
//! Unknown browsers are still detected with derived metadata.
//!
//! # Example
//!
//! ```no_run
//! use browserware_detect::{detect_browsers, detect_default_browser, BrowserFamily};
//!
//! // List all installed browsers
//! for browser in detect_browsers() {
//!     println!("{}: {} ({})", browser.id, browser.name, browser.family());
//!     if let Some(version) = &browser.version {
//!         println!("  Version: {version}");
//!     }
//! }
//!
//! // Get the default browser
//! if let Some(default) = detect_default_browser() {
//!     println!("Default browser: {}", default.name);
//! }
//!
//! // Filter by browser family
//! let chromium_browsers = browserware_detect::detect_browsers_by_family(BrowserFamily::Chromium);
//! println!("Found {} Chromium-based browsers", chromium_browsers.len());
//! ```
//!
//! # Platform Support
//!
//! | Platform | Status | Notes |
//! |----------|--------|-------|
//! | macOS    | Active | Uses Launch Services API |
//! | Windows  | Stub   | Uses Registry API (not yet implemented) |
//! | Linux    | Stub   | XDG desktop files (not yet implemented) |
//! | Other    | Stub   | Returns empty results |

// Allow unsafe code for platform FFI bindings
#![allow(unsafe_code)]
#![warn(missing_docs)]

mod platform;
pub mod registry;

// Re-export types from browserware-types for convenience
pub use browserware_types::{Browser, BrowserFamily, BrowserId, BrowserVariant};

/// Detect all installed browsers on the system.
///
/// Scans the system for installed browsers using platform-specific APIs
/// and returns a list of all detected browsers with their metadata.
///
/// # Returns
///
/// A `Vec<Browser>` containing all detected browsers. The list is not
/// guaranteed to be in any particular order. An empty vector is returned
/// if no browsers are found or on unsupported platforms.
///
/// # Example
///
/// ```no_run
/// let browsers = browserware_detect::detect_browsers();
/// for browser in browsers {
///     println!("{}: {}", browser.id, browser.name);
/// }
/// ```
#[tracing::instrument(level = "info", skip_all)]
#[must_use]
pub fn detect_browsers() -> Vec<Browser> {
    tracing::info!("Detecting installed browsers");
    let browsers = platform::detect_browsers();
    tracing::info!(count = browsers.len(), "Browser detection complete");
    browsers
}

/// Detect a specific browser by its canonical ID.
///
/// Searches for a browser installation matching the given ID. This is
/// more efficient than calling `detect_browsers()` and filtering when
/// you only need one specific browser.
///
/// # Arguments
///
/// * `id` - The browser's canonical identifier (e.g., "chrome", "firefox-nightly")
///
/// # Returns
///
/// `Some(Browser)` if the browser is installed, `None` otherwise.
///
/// # Example
///
/// ```no_run
/// if let Some(chrome) = browserware_detect::detect_browser("chrome") {
///     println!("Chrome is installed at: {}", chrome.executable.display());
/// } else {
///     println!("Chrome is not installed");
/// }
/// ```
#[tracing::instrument(level = "info")]
#[must_use]
pub fn detect_browser(id: &str) -> Option<Browser> {
    tracing::debug!(browser_id = id, "Looking for specific browser");

    // TODO: Optimize to detect only the requested browser
    detect_browsers().into_iter().find(|b| b.id.0 == id)
}

/// Detect the system's default browser.
///
/// Returns the browser configured as the default handler for HTTP/HTTPS URLs.
///
/// # Returns
///
/// `Some(Browser)` if a default browser is configured and detected,
/// `None` otherwise.
///
/// # Platform Behavior
///
/// - **macOS**: Queries Launch Services for the `https` URL scheme handler
/// - **Windows**: Reads the `UserChoice` registry key for HTTP associations
/// - **Linux**: Uses `xdg-settings get default-web-browser`
///
/// # Example
///
/// ```no_run
/// match browserware_detect::detect_default_browser() {
///     Some(browser) => println!("Default browser: {}", browser.name),
///     None => println!("No default browser detected"),
/// }
/// ```
#[tracing::instrument(level = "info", skip_all)]
#[must_use]
pub fn detect_default_browser() -> Option<Browser> {
    tracing::info!("Detecting default browser");
    let default = platform::detect_default_browser();

    if let Some(ref browser) = default {
        tracing::info!(browser_id = %browser.id, browser_name = %browser.name, "Default browser detected");
    } else {
        tracing::warn!("No default browser detected");
    }

    default
}

/// Detect all browsers of a specific engine family.
///
/// Filters the detected browsers to return only those belonging to
/// the specified browser engine family (Chromium, Firefox, `WebKit`, etc.).
///
/// # Arguments
///
/// * `family` - The browser engine family to filter by
///
/// # Returns
///
/// A `Vec<Browser>` containing all detected browsers of the specified family.
///
/// # Example
///
/// ```no_run
/// use browserware_detect::BrowserFamily;
///
/// // Find all Chromium-based browsers
/// let chromium = browserware_detect::detect_browsers_by_family(BrowserFamily::Chromium);
/// println!("Found {} Chromium browsers:", chromium.len());
/// for browser in chromium {
///     println!("  - {}", browser.name);
/// }
///
/// // Find all Firefox-based browsers
/// let firefox = browserware_detect::detect_browsers_by_family(BrowserFamily::Firefox);
/// println!("Found {} Firefox browsers", firefox.len());
/// ```
#[tracing::instrument(level = "info")]
#[must_use]
pub fn detect_browsers_by_family(family: BrowserFamily) -> Vec<Browser> {
    tracing::debug!(?family, "Filtering browsers by family");

    detect_browsers()
        .into_iter()
        .filter(|b| b.family() == family)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_browsers_returns_vec() {
        // Should not panic
        let _browsers = detect_browsers();
        // On macOS this will return actual browsers, on other platforms may be empty
    }

    #[test]
    fn detect_browser_unknown_id_returns_none() {
        // Unknown browsers should not be found even if we have browsers installed
        // Actually, with discovery-first approach, we might find unknown browsers
        // So this test checks for a truly nonexistent browser
        let result = detect_browser("nonexistent-browser-xyz-12345");
        // If found, it would be an unknown browser with derived ID
        // Most likely won't match this specific ID
        assert!(result.is_none());
    }

    #[test]
    fn detect_browsers_by_family_filters_correctly() {
        let chromium = detect_browsers_by_family(BrowserFamily::Chromium);
        // All returned browsers should be Chromium family
        for browser in chromium {
            assert_eq!(browser.family(), BrowserFamily::Chromium);
        }
    }
}
