//! Windows browser detection using the Registry.
//!
//! Detection strategy:
//! 1. Enumerate `HKLM\SOFTWARE\Clients\StartMenuInternet` subkeys
//! 2. For each subkey:
//!    a. Read `shell\open\command` for executable path
//!    b. Match against `KNOWN_BROWSERS` or derive metadata
//! 3. Check `HKCU\...\UrlAssociations\http\UserChoice\ProgId` for default

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use windows_registry::{CURRENT_USER, Key, LOCAL_MACHINE};

use browserware_types::{Browser, BrowserFamily, BrowserVariant};

use crate::registry;

/// Detect all installed browsers on Windows.
///
/// Enumerates all applications registered in the StartMenuInternet registry key,
/// then enriches with metadata from the known browser registry.
#[tracing::instrument(level = "debug")]
pub fn detect_browsers() -> Vec<Browser> {
    tracing::debug!("Starting Windows browser detection");

    let mut browsers = Vec::new();
    let subkey_names = collect_start_menu_internet_subkeys();

    tracing::debug!(count = subkey_names.len(), "Found browser registry entries");

    for subkey_name in subkey_names {
        tracing::trace!(registry_key = %subkey_name, "Processing browser entry");

        // Get executable path from shell\open\command
        let Some(executable) = get_browser_executable(&subkey_name) else {
            tracing::trace!(registry_key = %subkey_name, "Could not get executable path");
            continue;
        };

        // Build browser from metadata
        let browser = build_browser(&subkey_name, &executable);
        tracing::debug!(
            browser_id = %browser.id,
            browser_name = %browser.name,
            "Detected browser"
        );
        browsers.push(browser);
    }

    tracing::debug!(count = browsers.len(), "Windows browser detection complete");
    browsers
}

/// Detect the default browser on Windows.
///
/// Queries the Windows registry for the default HTTP handler.
#[tracing::instrument(level = "debug")]
pub fn detect_default_browser() -> Option<Browser> {
    tracing::debug!("Querying Windows default browser");

    // First try to get the ProgId from UserChoice
    let prog_id = get_default_browser_prog_id()?;

    tracing::debug!(prog_id = %prog_id, "Default browser ProgId found");

    // Map ProgId to registry key name
    let registry_key = map_prog_id_to_registry_key(&prog_id)?;

    tracing::debug!(registry_key = %registry_key, "Mapped to registry key");

    // Get executable path
    let executable = get_browser_executable(&registry_key)?;

    // Build browser
    let browser = build_browser(&registry_key, &executable);
    tracing::debug!(
        browser_id = %browser.id,
        browser_name = %browser.name,
        "Default browser detected"
    );

    Some(browser)
}

fn collect_start_menu_internet_subkeys() -> Vec<String> {
    let mut seen = HashSet::new();
    let mut merged = Vec::new();

    for root in [CURRENT_USER, LOCAL_MACHINE] {
        let Ok(key) = root.open(r"SOFTWARE\Clients\StartMenuInternet") else {
            continue;
        };

        let Ok(subkeys) = key.keys() else {
            continue;
        };

        for name in subkeys {
            if seen.insert(name.clone()) {
                merged.push(name);
            }
        }
    }

    merged
}

/// Get the ProgId for the default HTTP handler.
fn get_default_browser_prog_id() -> Option<String> {
    // Try HKCU\Software\Microsoft\Windows\Shell\Associations\UrlAssociations\http\UserChoice
    let user_choice_key = CURRENT_USER
        .open(r"Software\Microsoft\Windows\Shell\Associations\UrlAssociations\http\UserChoice")
        .ok()?;

    user_choice_key.get_string("ProgId").ok()
}

/// Map a ProgId to a registry key name.
///
/// Some ProgIds directly match registry key names, others need translation.
fn map_prog_id_to_registry_key(prog_id: &str) -> Option<String> {
    // Common ProgId patterns:
    // - ChromeHTML -> Google Chrome
    // - FirefoxURL-... -> Firefox
    // - MSEdgeHTM -> Microsoft Edge
    // - BraveHTML -> BraveSoftware Brave-Browser
    // - OperaStable -> Opera Stable

    match prog_id {
        "ChromeHTML" => Some("Google Chrome".to_string()),
        "MSEdgeHTM" => Some("Microsoft Edge".to_string()),
        "OperaStable" => Some("Opera Stable".to_string()),
        "BraveHTML" => Some("BraveSoftware Brave-Browser".to_string()),
        s if s.starts_with("FirefoxURL-") => Some("Firefox".to_string()),
        s if s.starts_with("ChromiumHTM") => Some("Chromium".to_string()),
        // For others, try to extract the browser name directly
        _ => {
            tracing::debug!(prog_id = %prog_id, "Unknown ProgId pattern, using as-is");
            Some(prog_id.to_string())
        }
    }
}

/// Get the executable path for a browser from its registry subkey.
fn get_browser_executable(subkey_name: &str) -> Option<PathBuf> {
    for root in [CURRENT_USER, LOCAL_MACHINE] {
        let Ok(parent_key) = root.open(r"SOFTWARE\Clients\StartMenuInternet") else {
            continue;
        };

        let Some(executable) = get_browser_executable_from_parent(&parent_key, subkey_name) else {
            continue;
        };

        return Some(executable);
    }

    None
}

fn get_browser_executable_from_parent(parent_key: &Key, subkey_name: &str) -> Option<PathBuf> {
    let browser_key = parent_key.open(subkey_name).ok()?;
    let command_key = browser_key.open(r"shell\open\command").ok()?;
    let command_string = command_key.get_string("").ok()?;
    parse_command_to_executable(&command_string)
}

/// Parse a command string to extract the executable path.
///
/// Windows command strings often look like:
/// - `"C:\Program Files\Google\Chrome\Application\chrome.exe"`
/// - `"C:\Program Files\Google\Chrome\Application\chrome.exe" -- "%1"`
/// - `C:\Program Files\Mozilla Firefox\firefox.exe -osint -url "%1"`
fn parse_command_to_executable(command: &str) -> Option<PathBuf> {
    let command = command.trim();

    if command.is_empty() {
        return None;
    }

    // If the command starts with a quote, extract the quoted path
    if command.starts_with('"') {
        if let Some(end_quote) = command[1..].find('"') {
            let path = &command[1..end_quote + 1];
            return Some(PathBuf::from(path));
        }
        // Malformed quoted string - no closing quote found
        return None;
    }

    // For unquoted paths, find the .exe boundary syntactically without filesystem access.
    // This handles paths like "C:\Program Files\Mozilla Firefox\firefox.exe -osint -url %1"
    let parts: Vec<&str> = command.split_whitespace().collect();

    // Try progressively longer paths until we find one that ends with .exe
    for i in 1..=parts.len() {
        let potential_path = parts[..i].join(" ");
        if potential_path.to_ascii_lowercase().ends_with(".exe") {
            return Some(PathBuf::from(potential_path));
        }
    }

    // Fallback: take everything up to the first space
    let path = parts.first().unwrap_or(&command);
    Some(PathBuf::from(path))
}

/// Build a Browser struct from registry key name and executable path.
fn build_browser(registry_key: &str, executable: &Path) -> Browser {
    // Try to match against known browsers
    if let Some(meta) = registry::find_by_registry_key(registry_key) {
        return build_browser_from_meta(meta, registry_key, executable);
    }

    // Unknown browser - derive metadata
    build_unknown_browser(registry_key, executable)
}

/// Build a Browser from known registry metadata.
fn build_browser_from_meta(
    meta: &'static registry::BrowserMeta,
    _registry_key: &str,
    executable: &Path,
) -> Browser {
    let version = extract_version_from_executable(executable);

    Browser::new(meta.id, meta.name, executable.to_path_buf())
        .with_variant(meta.variant)
        .maybe_with_version(version)
}

/// Build a Browser for an unknown application.
fn build_unknown_browser(registry_key: &str, executable: &Path) -> Browser {
    let name = registry_key.to_string();
    let version = extract_version_from_executable(executable);

    tracing::debug!(
        registry_key = registry_key,
        derived_name = %name,
        "Unknown browser - using registry key as identifier"
    );

    Browser::new(registry_key, name, executable.to_path_buf())
        .with_variant(BrowserVariant::Single(BrowserFamily::Other))
        .maybe_with_version(version)
}

/// Extract version information from a Windows executable.
///
/// This requires reading the PE file's version resource, which is complex.
/// For now, return None and log that version extraction is not implemented.
fn extract_version_from_executable(_executable: &Path) -> Option<String> {
    // TODO: Implement PE version info extraction
    // This requires reading the VERSION_INFO resource from the PE file
    // Libraries like pelite or goblin could be used for this
    tracing::trace!("Windows PE version extraction not yet implemented");
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
    fn parse_quoted_command() {
        let command = r#""C:\Program Files\Google\Chrome\Application\chrome.exe" -- "%1""#;
        let executable = parse_command_to_executable(command);
        assert_eq!(
            executable,
            Some(PathBuf::from(
                r"C:\Program Files\Google\Chrome\Application\chrome.exe"
            ))
        );
    }

    #[test]
    fn parse_unquoted_command() {
        let command = r"C:\Program Files\Mozilla Firefox\firefox.exe -osint -url %1";
        let executable = parse_command_to_executable(command);
        assert_eq!(
            executable,
            Some(PathBuf::from(
                r"C:\Program Files\Mozilla Firefox\firefox.exe"
            ))
        );
    }

    #[test]
    fn parse_simple_path() {
        let command = r"C:\Users\test\browser.exe";
        let executable = parse_command_to_executable(command);
        assert_eq!(
            executable,
            Some(PathBuf::from(r"C:\Users\test\browser.exe"))
        );
    }

    #[test]
    fn parse_empty_command() {
        let command = "";
        let executable = parse_command_to_executable(command);
        assert_eq!(executable, None);
    }

    #[test]
    fn map_common_prog_ids() {
        assert_eq!(
            map_prog_id_to_registry_key("ChromeHTML"),
            Some("Google Chrome".to_string())
        );
        assert_eq!(
            map_prog_id_to_registry_key("MSEdgeHTM"),
            Some("Microsoft Edge".to_string())
        );
        assert_eq!(
            map_prog_id_to_registry_key("FirefoxURL-308046B0AF4A39CB"),
            Some("Firefox".to_string())
        );
        assert_eq!(
            map_prog_id_to_registry_key("BraveHTML"),
            Some("BraveSoftware Brave-Browser".to_string())
        );
    }
}
