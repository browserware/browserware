//! Linux browser detection using XDG desktop files.
//!
//! Detection strategy:
//! 1. Scan XDG application directories for `.desktop` files
//! 2. Filter files with `MimeType=` containing `x-scheme-handler/http`
//! 3. Parse `Exec=` field for executable path
//! 4. Match against `KNOWN_BROWSERS` or derive metadata
//! 5. Use `xdg-settings get default-web-browser` for default

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use browserware_types::{Browser, BrowserFamily, BrowserVariant};

use crate::registry;

/// Detect all installed browsers on Linux.
///
/// Scans XDG application directories for desktop files that declare
/// HTTP URL handling capability, then enriches with metadata from
/// the known browser registry.
#[tracing::instrument(level = "debug")]
pub fn detect_browsers() -> Vec<Browser> {
    tracing::debug!("Starting Linux browser detection");

    let mut browsers = Vec::new();
    let mut seen_ids = HashSet::new();

    for dir in get_desktop_dirs() {
        tracing::trace!(?dir, "Scanning directory");

        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(e) => {
                tracing::trace!(?dir, error = %e, "Could not read directory");
                continue;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();

            // Skip non-.desktop files
            if path.extension().is_none_or(|ext| ext != "desktop") {
                continue;
            }

            // Parse desktop file
            let Some(desktop) = parse_desktop_file(&path) else {
                continue;
            };

            // Skip if not a browser (doesn't handle http)
            if !desktop.is_browser() {
                continue;
            }

            // Get desktop ID (filename without .desktop extension)
            let Some(desktop_id) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };

            // Build browser from metadata
            let browser = build_browser(desktop_id, &desktop);

            // Skip duplicates (same browser from different directories)
            if seen_ids.contains(&browser.id.0) {
                tracing::trace!(
                    browser_id = %browser.id,
                    ?path,
                    "Skipping duplicate browser"
                );
                continue;
            }

            tracing::debug!(
                browser_id = %browser.id,
                browser_name = %browser.name,
                ?path,
                "Detected browser"
            );

            seen_ids.insert(browser.id.0.clone());
            browsers.push(browser);
        }
    }

    tracing::debug!(count = browsers.len(), "Linux browser detection complete");
    browsers
}

/// Detect the default browser on Linux.
///
/// Uses `xdg-settings get default-web-browser` to query the default browser.
#[tracing::instrument(level = "debug")]
pub fn detect_default_browser() -> Option<Browser> {
    tracing::debug!("Querying Linux default browser via xdg-settings");

    let output = Command::new("xdg-settings")
        .args(["get", "default-web-browser"])
        .output()
        .ok()?;

    if !output.status.success() {
        tracing::warn!(
            status = ?output.status,
            "xdg-settings command failed"
        );
        return None;
    }

    let desktop_file = String::from_utf8_lossy(&output.stdout);
    let desktop_file = desktop_file.trim();

    if desktop_file.is_empty() {
        tracing::debug!("No default browser configured");
        return None;
    }

    tracing::debug!(desktop_file = %desktop_file, "Default browser desktop file");

    // Remove .desktop extension if present
    let desktop_id = desktop_file
        .strip_suffix(".desktop")
        .unwrap_or(desktop_file);

    // Find this browser in detected browsers
    detect_browsers().into_iter().find(|b| {
        // Check if any of the known browser's desktop IDs match
        registry::find_by_id(&b.id.0).map_or_else(
            // For unknown browsers, check if ID matches desktop_id
            || b.id.0 == desktop_id,
            |meta| meta.linux_desktop_ids.contains(&desktop_id),
        )
    })
}

/// Get all XDG application directories to search for desktop files.
fn get_desktop_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    // System-wide directories
    dirs.push(PathBuf::from("/usr/share/applications"));
    dirs.push(PathBuf::from("/usr/local/share/applications"));

    // User directory from XDG
    let xdg_dirs = xdg::BaseDirectories::new();
    if let Some(data_home) = xdg_dirs.get_data_home() {
        dirs.push(data_home.join("applications"));
    }

    // Flatpak user directory
    if let Some(home) = home::home_dir() {
        dirs.push(home.join(".local/share/flatpak/exports/share/applications"));
    }

    // Snap directory
    dirs.push(PathBuf::from("/var/lib/snapd/desktop/applications"));

    // Filter to only existing directories
    dirs.retain(|d| d.is_dir());

    tracing::trace!(dirs = ?dirs, "XDG application directories");
    dirs
}

/// Parsed .desktop file content.
#[derive(Debug, Default)]
struct DesktopEntry {
    name: Option<String>,
    exec: Option<String>,
    mime_types: Vec<String>,
    categories: Vec<String>,
}

impl DesktopEntry {
    /// Check if this desktop entry declares HTTP URL handling capability.
    fn is_browser(&self) -> bool {
        // Check for http scheme handler in mime types
        let handles_http = self.mime_types.iter().any(|m| {
            m == "x-scheme-handler/http" || m == "x-scheme-handler/https" || m == "text/html"
        });

        // Also check if it's in the WebBrowser category
        let is_web_browser = self.categories.iter().any(|c| c == "WebBrowser");

        handles_http || is_web_browser
    }
}

/// Parse a .desktop file into a `DesktopEntry`.
fn parse_desktop_file(path: &Path) -> Option<DesktopEntry> {
    let content = std::fs::read_to_string(path).ok()?;

    let mut entry = DesktopEntry::default();
    let mut in_desktop_entry = false;

    for line in content.lines() {
        let line = line.trim();

        // Handle section headers
        if line.starts_with('[') {
            in_desktop_entry = line == "[Desktop Entry]";
            continue;
        }

        // Only parse [Desktop Entry] section
        if !in_desktop_entry {
            continue;
        }

        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse key=value
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };

        match key {
            "Name" => entry.name = Some(value.to_string()),
            "Exec" => entry.exec = Some(value.to_string()),
            "MimeType" => {
                entry.mime_types = value
                    .split(';')
                    .filter(|s| !s.is_empty())
                    .map(String::from)
                    .collect();
            }
            "Categories" => {
                entry.categories = value
                    .split(';')
                    .filter(|s| !s.is_empty())
                    .map(String::from)
                    .collect();
            }
            _ => {}
        }
    }

    // Must have at least a name and exec
    if entry.name.is_some() && entry.exec.is_some() {
        Some(entry)
    } else {
        None
    }
}

/// Build a Browser from a desktop entry and its ID.
fn build_browser(desktop_id: &str, entry: &DesktopEntry) -> Browser {
    // Try to match against known browsers
    if let Some(meta) = registry::find_by_desktop_id(desktop_id) {
        return build_browser_from_meta(meta, entry);
    }

    // Unknown browser - derive metadata
    build_unknown_browser(desktop_id, entry)
}

/// Build a Browser from known registry metadata.
fn build_browser_from_meta(meta: &'static registry::BrowserMeta, entry: &DesktopEntry) -> Browser {
    let executable = entry
        .exec
        .as_ref()
        .and_then(|e| parse_exec_to_path(e))
        .unwrap_or_default();

    let version = extract_version(&executable);

    Browser::new(meta.id, meta.name, executable)
        .with_variant(meta.variant)
        .maybe_with_version(version)
}

/// Build a Browser for an unknown application.
fn build_unknown_browser(desktop_id: &str, entry: &DesktopEntry) -> Browser {
    let name = entry.name.clone().unwrap_or_else(|| desktop_id.to_string());

    let executable = entry
        .exec
        .as_ref()
        .and_then(|e| parse_exec_to_path(e))
        .unwrap_or_default();

    let version = extract_version(&executable);

    tracing::debug!(
        desktop_id = desktop_id,
        derived_name = %name,
        "Unknown browser - using desktop ID as identifier"
    );

    Browser::new(desktop_id, name, executable)
        .with_variant(BrowserVariant::Single(BrowserFamily::Other))
        .maybe_with_version(version)
}

/// Parse the Exec field to extract the executable path.
///
/// The Exec field can have various formats:
/// - `/usr/bin/firefox %u`
/// - `env VAR=value /usr/bin/browser %U`
/// - `flatpak run org.mozilla.firefox`
/// - `snap run firefox`
fn parse_exec_to_path(exec: &str) -> Option<PathBuf> {
    let parts: Vec<&str> = exec.split_whitespace().collect();

    for (i, part) in parts.iter().enumerate() {
        // Skip field codes like %u, %U, %f, %F
        if part.starts_with('%') {
            continue;
        }

        // Skip environment variable assignments
        if part.contains('=') {
            continue;
        }

        // Handle flatpak/snap wrappers
        if *part == "flatpak" && parts.get(i + 1) == Some(&"run") {
            // For flatpak, the app ID is next
            // Return the flatpak command itself as the "executable"
            if let Some(app_id) = parts.get(i + 2) {
                // Try to find the actual executable, fall back to flatpak command
                return Some(PathBuf::from(format!("/usr/bin/flatpak run {app_id}")));
            }
            return Some(PathBuf::from("/usr/bin/flatpak"));
        }

        if *part == "snap" && parts.get(i + 1) == Some(&"run") {
            if let Some(snap_name) = parts.get(i + 2) {
                return Some(PathBuf::from(format!("/snap/bin/{snap_name}")));
            }
            return Some(PathBuf::from("/usr/bin/snap"));
        }

        // Skip known wrapper commands
        if ["env", "sh", "-c", "bash"].contains(part) {
            continue;
        }

        // If it's an absolute path, use it directly
        if part.starts_with('/') {
            return Some(PathBuf::from(part));
        }

        // Try to resolve via which
        if let Ok(output) = Command::new("which").arg(part).output()
            && output.status.success()
        {
            let resolved = String::from_utf8_lossy(&output.stdout);
            let resolved = resolved.trim();
            if !resolved.is_empty() {
                return Some(PathBuf::from(resolved));
            }
        }

        // If we've skipped wrappers and found a non-wrapper command, use it
        // even if we can't resolve its full path
        if !part.is_empty() && !part.starts_with('-') {
            return Some(PathBuf::from(part));
        }
    }

    None
}

/// Try to extract version from browser executable.
///
/// Runs `executable --version` and parses the output.
fn extract_version(executable: &Path) -> Option<String> {
    // Skip for empty paths or non-existent executables
    if executable.as_os_str().is_empty() {
        return None;
    }

    // Handle special cases like flatpak/snap paths
    let exec_str = executable.to_string_lossy();
    if exec_str.contains("flatpak run") || exec_str.starts_with("/snap/bin/") {
        // For flatpak/snap, version extraction is more complex
        // Skip for now - version is optional
        return None;
    }

    // Try to run --version
    let output = Command::new(executable).arg("--version").output().ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_version_string(&stdout)
}

/// Parse version number from --version output.
///
/// Handles common patterns like:
/// - "Mozilla Firefox 120.0"
/// - "Google Chrome 120.0.6099.109"
/// - "Chromium 120.0.6099.109 built on Debian"
fn parse_version_string(output: &str) -> Option<String> {
    // Find a version-like pattern: numbers separated by dots
    // Common pattern: word followed by version
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Look for version pattern: X.Y.Z (optional more segments)
        for word in line.split_whitespace() {
            // Check if it looks like a version (starts with digit, contains dots)
            if word.chars().next().is_some_and(|c| c.is_ascii_digit()) && word.contains('.') {
                // Clean up trailing punctuation
                let version = word.trim_end_matches(|c: char| !c.is_ascii_digit() && c != '.');
                if !version.is_empty() {
                    return Some(version.to_string());
                }
            }
        }
    }

    None
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
    fn parse_exec_absolute_path() {
        assert_eq!(
            parse_exec_to_path("/usr/bin/firefox %u"),
            Some(PathBuf::from("/usr/bin/firefox"))
        );
    }

    #[test]
    fn parse_exec_with_env() {
        // When env is used, we skip it and find the actual command
        let result = parse_exec_to_path("env VAR=value /usr/bin/browser %U");
        assert_eq!(result, Some(PathBuf::from("/usr/bin/browser")));
    }

    #[test]
    fn parse_exec_flatpak() {
        let result = parse_exec_to_path("flatpak run org.mozilla.firefox %u");
        assert!(result.is_some());
        // Returns a flatpak command string
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("flatpak"));
    }

    #[test]
    fn parse_exec_snap() {
        let result = parse_exec_to_path("snap run firefox");
        assert_eq!(result, Some(PathBuf::from("/snap/bin/firefox")));
    }

    #[test]
    fn parse_version_firefox() {
        let output = "Mozilla Firefox 120.0";
        assert_eq!(parse_version_string(output), Some("120.0".to_string()));
    }

    #[test]
    fn parse_version_chrome() {
        let output = "Google Chrome 120.0.6099.109";
        assert_eq!(
            parse_version_string(output),
            Some("120.0.6099.109".to_string())
        );
    }

    #[test]
    fn parse_version_chromium_verbose() {
        let output = "Chromium 120.0.6099.109 built on Debian 12.4";
        assert_eq!(
            parse_version_string(output),
            Some("120.0.6099.109".to_string())
        );
    }

    #[test]
    fn desktop_entry_is_browser_with_http_handler() {
        let entry = DesktopEntry {
            name: Some("Firefox".to_string()),
            exec: Some("/usr/bin/firefox %u".to_string()),
            mime_types: vec!["x-scheme-handler/http".to_string()],
            categories: vec![],
        };
        assert!(entry.is_browser());
    }

    #[test]
    fn desktop_entry_is_browser_with_category() {
        let entry = DesktopEntry {
            name: Some("Firefox".to_string()),
            exec: Some("/usr/bin/firefox %u".to_string()),
            mime_types: vec![],
            categories: vec!["WebBrowser".to_string(), "Network".to_string()],
        };
        assert!(entry.is_browser());
    }

    #[test]
    fn desktop_entry_not_browser() {
        let entry = DesktopEntry {
            name: Some("Text Editor".to_string()),
            exec: Some("/usr/bin/gedit %u".to_string()),
            mime_types: vec!["text/plain".to_string()],
            categories: vec!["TextEditor".to_string()],
        };
        assert!(!entry.is_browser());
    }
}
