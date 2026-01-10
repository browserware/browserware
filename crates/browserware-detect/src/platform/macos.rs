//! macOS browser detection using Launch Services.
//!
//! Detection strategy:
//! 1. `LSCopyAllHandlersForURLScheme("https")` → get all bundle IDs
//! 2. For each bundle ID:
//!    a. `LSCopyApplicationURLsForBundleIdentifier` → get app path
//!    b. Parse `Info.plist` for version and display name
//!    c. Match against `KNOWN_BROWSERS` or derive metadata
//! 3. `LSCopyDefaultHandlerForURLScheme("https")` → identify default browser

use std::path::{Path, PathBuf};

use core_foundation::array::CFArray;
use core_foundation::base::TCFType;
use core_foundation::string::{CFString, CFStringRef};
use core_foundation::url::CFURL;

use browserware_types::{Browser, BrowserFamily, BrowserVariant};

use crate::registry;

// FFI bindings for Launch Services functions not exposed by core-foundation crate
#[link(name = "CoreServices", kind = "framework")]
unsafe extern "C" {
    fn LSCopyAllHandlersForURLScheme(
        scheme: CFStringRef,
    ) -> *const core_foundation::array::__CFArray;
    fn LSCopyDefaultHandlerForURLScheme(scheme: CFStringRef) -> CFStringRef;
    fn LSCopyApplicationURLsForBundleIdentifier(
        bundle_id: CFStringRef,
        out_error: *mut core_foundation::error::CFErrorRef,
    ) -> *const core_foundation::array::__CFArray;
}

/// Detect all installed browsers on macOS.
///
/// Enumerates all applications registered as HTTPS URL handlers using
/// Launch Services, then enriches with metadata from the known browser registry.
#[tracing::instrument(level = "debug")]
pub fn detect_browsers() -> Vec<Browser> {
    tracing::debug!("Starting macOS browser detection");

    let mut browsers = Vec::new();

    // Get all applications that can handle HTTPS URLs
    let Some(bundle_ids) = get_all_url_handlers("https") else {
        tracing::warn!("Failed to get URL handlers");
        return browsers;
    };

    tracing::debug!(count = bundle_ids.len(), "Found URL handlers");

    for bundle_id in &bundle_ids {
        let bundle_id_str = bundle_id.to_string();
        tracing::trace!(bundle_id = %bundle_id_str, "Processing handler");

        // Get application path
        let Some(app_url) = get_application_url(&bundle_id_str) else {
            tracing::trace!(bundle_id = %bundle_id_str, "Could not get application URL");
            continue;
        };

        let Some(app_path) = app_url.to_path() else {
            tracing::trace!(bundle_id = %bundle_id_str, "Could not convert URL to path");
            continue;
        };

        // Skip nested apps (e.g., helper apps inside Contents/Support/)
        if is_nested_app(&app_path) {
            tracing::trace!(bundle_id = %bundle_id_str, ?app_path, "Skipping nested app");
            continue;
        }

        // Build browser from metadata
        let browser = build_browser(&bundle_id_str, &app_path);
        tracing::debug!(
            browser_id = %browser.id,
            browser_name = %browser.name,
            "Detected browser"
        );
        browsers.push(browser);
    }

    tracing::debug!(count = browsers.len(), "macOS browser detection complete");
    browsers
}

/// Detect the default browser on macOS.
///
/// Queries Launch Services for the default HTTPS URL handler.
#[tracing::instrument(level = "debug")]
pub fn detect_default_browser() -> Option<Browser> {
    tracing::debug!("Querying macOS default browser");

    let bundle_id = get_default_url_handler("https")?;
    let bundle_id_str = bundle_id.to_string();

    tracing::debug!(bundle_id = %bundle_id_str, "Default handler found");

    let app_url = get_application_url(&bundle_id_str)?;
    let app_path = app_url.to_path()?;

    // Skip nested apps (same filter as detect_browsers)
    if is_nested_app(&app_path) {
        tracing::debug!(bundle_id = %bundle_id_str, ?app_path, "Default browser is a nested app, skipping");
        return None;
    }

    let browser = build_browser(&bundle_id_str, &app_path);
    tracing::debug!(
        browser_id = %browser.id,
        browser_name = %browser.name,
        "Default browser detected"
    );

    Some(browser)
}

/// Get all applications registered to handle a URL scheme.
fn get_all_url_handlers(scheme: &str) -> Option<Vec<CFString>> {
    let scheme_cf = CFString::new(scheme);

    // SAFETY: LSCopyAllHandlersForURLScheme is a safe C function that returns
    // a CFArray of CFStrings or NULL. We own the returned array.
    let array_ptr = unsafe { LSCopyAllHandlersForURLScheme(scheme_cf.as_concrete_TypeRef()) };

    if array_ptr.is_null() {
        return None;
    }

    // SAFETY: We verified the pointer is not null, and we own it (Copy in name means we own it)
    let array: CFArray<CFString> = unsafe { CFArray::wrap_under_create_rule(array_ptr.cast_mut()) };

    Some(array.iter().map(|s| s.clone()).collect())
}

/// Get the default application for a URL scheme.
fn get_default_url_handler(scheme: &str) -> Option<CFString> {
    let scheme_cf = CFString::new(scheme);

    // SAFETY: LSCopyDefaultHandlerForURLScheme returns a CFString or NULL
    let string_ref = unsafe { LSCopyDefaultHandlerForURLScheme(scheme_cf.as_concrete_TypeRef()) };

    if string_ref.is_null() {
        return None;
    }

    // SAFETY: We verified the pointer is not null, and we own it
    Some(unsafe { CFString::wrap_under_create_rule(string_ref) })
}

/// Get the application URL for a bundle identifier.
fn get_application_url(bundle_id: &str) -> Option<CFURL> {
    let bundle_id_cf = CFString::new(bundle_id);

    // SAFETY: LSCopyApplicationURLsForBundleIdentifier returns an array of URLs or NULL
    let array_ptr = unsafe {
        LSCopyApplicationURLsForBundleIdentifier(
            bundle_id_cf.as_concrete_TypeRef(),
            std::ptr::null_mut(),
        )
    };

    if array_ptr.is_null() {
        return None;
    }

    // SAFETY: We verified the pointer is not null, and we own it
    let array: CFArray<CFURL> = unsafe { CFArray::wrap_under_create_rule(array_ptr.cast_mut()) };

    // Return the first URL (primary installation)
    array.iter().next().map(|url| url.clone())
}

/// Check if an app is nested inside another app bundle.
///
/// Nested apps (like helper apps in Contents/Support/) should be filtered out
/// to match the behavior of macOS System Settings.
fn is_nested_app(app_path: &Path) -> bool {
    let path_str = app_path.to_string_lossy();

    // Check if there's a ".app/" before the final ".app"
    // e.g., "/Applications/Foo.app/Contents/Support/Bar.app" is nested
    let Some(last_app_pos) = path_str.rfind(".app") else {
        return false;
    };

    let before_last = &path_str[..last_app_pos];
    before_last.contains(".app/")
}

/// Build a Browser struct from bundle ID and application path.
fn build_browser(bundle_id: &str, app_path: &Path) -> Browser {
    // Try to match against known browsers
    if let Some(meta) = registry::find_by_bundle_id(bundle_id) {
        return build_browser_from_meta(meta, bundle_id, app_path);
    }

    // Unknown browser - derive metadata
    build_unknown_browser(bundle_id, app_path)
}

/// Build a Browser from known registry metadata.
fn build_browser_from_meta(
    meta: &'static registry::BrowserMeta,
    bundle_id: &str,
    app_path: &Path,
) -> Browser {
    let version = extract_version_from_plist(app_path);
    let executable = find_executable(app_path);

    Browser::new(meta.id, meta.name, executable)
        .with_variant(meta.variant)
        .with_bundle_id(bundle_id)
        .maybe_with_version(version)
}

/// Build a Browser for an unknown application.
fn build_unknown_browser(bundle_id: &str, app_path: &Path) -> Browser {
    let name = extract_name_from_plist(app_path)
        .or_else(|| derive_name_from_bundle_id(bundle_id))
        .unwrap_or_else(|| bundle_id.to_string());

    let version = extract_version_from_plist(app_path);
    let executable = find_executable(app_path);

    tracing::debug!(
        bundle_id = bundle_id,
        derived_name = %name,
        "Unknown browser - using bundle ID as identifier"
    );

    Browser::new(bundle_id, name, executable)
        .with_variant(BrowserVariant::Single(BrowserFamily::Other))
        .with_bundle_id(bundle_id)
        .maybe_with_version(version)
}

/// Extract version from Info.plist.
fn extract_version_from_plist(app_path: &Path) -> Option<String> {
    let plist_path = app_path.join("Contents/Info.plist");

    let plist = plist::Value::from_file(&plist_path).ok()?;
    let dict = plist.as_dictionary()?;

    // Try CFBundleShortVersionString first, then CFBundleVersion
    dict.get("CFBundleShortVersionString")
        .or_else(|| dict.get("CFBundleVersion"))
        .and_then(|v| v.as_string())
        .map(String::from)
}

/// Extract display name from Info.plist.
fn extract_name_from_plist(app_path: &Path) -> Option<String> {
    let plist_path = app_path.join("Contents/Info.plist");

    let plist = plist::Value::from_file(&plist_path).ok()?;
    let dict = plist.as_dictionary()?;

    // Try CFBundleDisplayName first, then CFBundleName
    dict.get("CFBundleDisplayName")
        .or_else(|| dict.get("CFBundleName"))
        .and_then(|v| v.as_string())
        .map(String::from)
}

/// Derive a display name from bundle ID.
fn derive_name_from_bundle_id(bundle_id: &str) -> Option<String> {
    // com.example.MyBrowser -> MyBrowser
    bundle_id.split('.').next_back().map(String::from)
}

/// Find the main executable inside the app bundle.
fn find_executable(app_path: &Path) -> PathBuf {
    // Try to get executable name from Info.plist
    if let Some(exec_name) = get_executable_name_from_plist(app_path) {
        let exec_path = app_path.join("Contents/MacOS").join(&exec_name);
        if exec_path.exists() {
            return exec_path;
        }
        tracing::trace!(?exec_path, "Executable from plist doesn't exist");
    }

    // Fall back to app name
    let app_name = app_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("executable");

    let fallback_path = app_path.join("Contents/MacOS").join(app_name);

    if !fallback_path.exists() {
        tracing::warn!(?fallback_path, "Fallback executable path doesn't exist");
    }

    fallback_path
}

/// Get the executable name from Info.plist.
fn get_executable_name_from_plist(app_path: &Path) -> Option<String> {
    let plist_path = app_path.join("Contents/Info.plist");

    let plist = plist::Value::from_file(&plist_path).ok()?;
    let dict = plist.as_dictionary()?;

    dict.get("CFBundleExecutable")
        .and_then(|v| v.as_string())
        .map(String::from)
}

/// Extension trait to add `maybe_with_version` to Browser.
trait BrowserExt {
    fn maybe_with_version(self, version: Option<String>) -> Self;
}

impl BrowserExt for Browser {
    fn maybe_with_version(self, version: Option<String>) -> Self {
        if let Some(v) = version {
            self.with_version(v)
        } else {
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_name_from_bundle_id_works() {
        assert_eq!(
            derive_name_from_bundle_id("com.google.Chrome"),
            Some("Chrome".to_string())
        );
        assert_eq!(
            derive_name_from_bundle_id("org.mozilla.firefox"),
            Some("firefox".to_string())
        );
    }

    #[test]
    fn is_nested_app_detects_nested_apps() {
        // Nested app inside Contents/Support/
        assert!(is_nested_app(Path::new(
            "/Applications/ChatGPT Atlas.app/Contents/Support/ChatGPT Atlas.app"
        )));

        // Nested app inside Contents/Frameworks/
        assert!(is_nested_app(Path::new(
            "/Applications/Foo.app/Contents/Frameworks/Helper.app"
        )));

        // Top-level app - not nested
        assert!(!is_nested_app(Path::new("/Applications/Safari.app")));
        assert!(!is_nested_app(Path::new("/Applications/Google Chrome.app")));
        assert!(!is_nested_app(Path::new(
            "/System/Volumes/Preboot/Cryptexes/App/System/Applications/Safari.app"
        )));
    }
}
