//! Linux browser detection using XDG desktop files.
//!
//! Detection strategy:
//! 1. Scan XDG application directories for `.desktop` files
//! 2. Filter files with `MimeType=` containing `x-scheme-handler/http`
//! 3. Parse `Exec=` field for executable path
//! 4. Match against `KNOWN_BROWSERS` or derive metadata
//! 5. Use `xdg-settings get default-web-browser` for default

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use browserware_types::{Browser, BrowserFamily, BrowserVariant};

use crate::registry;

/// Detect all installed browsers on Linux.
#[tracing::instrument(level = "debug")]
pub fn detect_browsers() -> Vec<Browser> {
    tracing::debug!("Starting Linux browser detection");

    let mut browsers = Vec::new();
    let mut seen_desktop_ids = HashSet::new();

    // Get all XDG application directories
    let app_dirs = get_application_directories();
    tracing::debug!(count = app_dirs.len(), "Found application directories");

    for app_dir in &app_dirs {
        if !app_dir.exists() {
            continue;
        }

        // Scan for .desktop files
        let Ok(entries) = fs::read_dir(app_dir) else {
            tracing::trace!(?app_dir, "Could not read directory");
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();

            // Only process .desktop files
            if path.extension().and_then(|s| s.to_str()) != Some("desktop") {
                continue;
            }

            let Some(desktop_id) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };

            // Skip if we've already processed this desktop ID
            if seen_desktop_ids.contains(desktop_id) {
                continue;
            }

            // Parse the desktop file
            let Some(desktop_entry) = parse_desktop_file(&path) else {
                continue;
            };

            // Filter for HTTP handlers
            if !desktop_entry.is_http_handler {
                continue;
            }

            seen_desktop_ids.insert(desktop_id.to_string());

            // Build browser from desktop entry
            let browser = build_browser(desktop_id, &desktop_entry);
            tracing::debug!(
                browser_id = %browser.id,
                browser_name = %browser.name,
                "Detected browser"
            );
            browsers.push(browser);
        }
    }

    tracing::debug!(count = browsers.len(), "Linux browser detection complete");
    browsers
}

/// Detect the default browser on Linux.
#[tracing::instrument(level = "debug")]
pub fn detect_default_browser() -> Option<Browser> {
    tracing::debug!("Querying Linux default browser");

    // Try xdg-settings first
    let desktop_id = get_default_browser_desktop_id()?;
    tracing::debug!(desktop_id = %desktop_id, "Default browser found");

    // Find the desktop file
    let desktop_file = find_desktop_file(&desktop_id)?;
    let desktop_entry = parse_desktop_file(&desktop_file)?;

    let browser = build_browser(&desktop_id, &desktop_entry);
    tracing::debug!(
        browser_id = %browser.id,
        browser_name = %browser.name,
        "Default browser detected"
    );

    Some(browser)
}

/// Get the default browser desktop ID using xdg-settings.
fn get_default_browser_desktop_id() -> Option<String> {
    let output = Command::new("xdg-settings")
        .args(["get", "default-web-browser"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let desktop_id = String::from_utf8(output.stdout).ok()?;
    let desktop_id = desktop_id.trim();

    // Remove .desktop extension if present
    let desktop_id = desktop_id.strip_suffix(".desktop").unwrap_or(desktop_id);

    Some(desktop_id.to_string())
}

/// Get all XDG application directories.
fn get_application_directories() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    // User directory via XDG comes first so per-user desktop files override system entries.
    let xdg_dirs = xdg::BaseDirectories::new();
    if let Some(data_home) = xdg_dirs.get_data_home() {
        dirs.push(data_home.join("applications"));
    }

    // Fallback user directories
    if let Some(home) = home::home_dir() {
        dirs.push(home.join(".local/share/applications"));
        dirs.push(home.join(".local/share/flatpak/exports/share/applications"));
    }

    // System directories
    dirs.push(PathBuf::from("/usr/share/applications"));
    dirs.push(PathBuf::from("/usr/local/share/applications"));

    // Flatpak
    dirs.push(PathBuf::from("/var/lib/flatpak/exports/share/applications"));

    // Snap
    dirs.push(PathBuf::from("/var/lib/snapd/desktop/applications"));

    dirs
}

/// Find a desktop file by its ID.
fn find_desktop_file(desktop_id: &str) -> Option<PathBuf> {
    let app_dirs = get_application_directories();

    for app_dir in app_dirs {
        let desktop_file = app_dir.join(format!("{desktop_id}.desktop"));
        if desktop_file.exists() {
            return Some(desktop_file);
        }
    }

    None
}

/// A parsed desktop entry.
#[derive(Debug)]
struct DesktopEntry {
    name: String,
    exec: String,
    is_http_handler: bool,
}

/// Parse a .desktop file.
fn parse_desktop_file(path: &Path) -> Option<DesktopEntry> {
    let content = fs::read_to_string(path).ok()?;

    let mut name: Option<String> = None;
    let mut exec: Option<String> = None;
    let mut mime_types: Vec<String> = Vec::new();
    let mut in_desktop_entry = false;

    for line in content.lines() {
        let line = line.trim();

        // Track [Desktop Entry] section
        if line == "[Desktop Entry]" {
            in_desktop_entry = true;
            continue;
        } else if line.starts_with('[') {
            in_desktop_entry = false;
            continue;
        }

        // Only parse lines in [Desktop Entry] section
        if !in_desktop_entry {
            continue;
        }

        // Parse key=value pairs
        if let Some((key, value)) = line.split_once('=') {
            match key {
                "Name" => name = Some(value.to_string()),
                "Exec" => exec = Some(value.to_string()),
                "MimeType" => {
                    mime_types = value.split(';').map(String::from).collect();
                }
                _ => {}
            }
        }
    }

    let name = name?;
    let exec = exec?;

    // Check if this is an HTTP handler
    let is_http_handler = mime_types
        .iter()
        .any(|mime| mime == "x-scheme-handler/http" || mime == "x-scheme-handler/https");

    Some(DesktopEntry {
        name,
        exec,
        is_http_handler,
    })
}

/// Build a Browser from a desktop entry.
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
    let executable = parse_exec_to_path(&entry.exec);

    Browser::new(meta.id, meta.name, executable).with_variant(meta.variant)
}

/// Build a Browser for an unknown application.
fn build_unknown_browser(desktop_id: &str, entry: &DesktopEntry) -> Browser {
    let executable = parse_exec_to_path(&entry.exec);

    tracing::debug!(
        desktop_id = desktop_id,
        derived_name = %entry.name,
        "Unknown browser - using desktop ID as identifier"
    );

    Browser::new(desktop_id, &entry.name, executable)
        .with_variant(BrowserVariant::Single(BrowserFamily::Other))
}

/// Parse the Exec= field to extract the executable path.
///
/// Desktop file Exec= fields can contain arguments and field codes like %U, %F, etc.
/// This function extracts just the executable path.
fn parse_quoted_exec(stripped: &str, quote: char) -> (&str, &str) {
    stripped.find(quote).map_or_else(
        || (stripped, ""),
        |end_quote| {
            let path = &stripped[..end_quote];
            let remaining = &stripped[end_quote + 1..];
            (path, remaining)
        },
    )
}

fn parse_exec_to_path(exec: &str) -> PathBuf {
    let exec = exec.trim();

    // Handle quoted paths first (before splitting on whitespace)
    let (executable, remaining) = exec.strip_prefix('"').map_or_else(
        || {
            exec.strip_prefix('\'').map_or_else(
                || {
                    // Not quoted, scan tokens and skip wrapper commands / environment assignments.
                    let parts: Vec<&str> = exec.split_whitespace().collect();
                    let executable = parts
                        .iter()
                        .copied()
                        .find(|token| !should_skip_exec_token(token))
                        .unwrap_or(exec);
                    (executable, "")
                },
                |stripped| parse_quoted_exec(stripped, '\''),
            )
        },
        |stripped| parse_quoted_exec(stripped, '"'),
    );

    // Parse remaining for flatpak/snap detection
    let parts: Vec<&str> = if remaining.is_empty() {
        vec![executable]
    } else {
        let mut v = vec![executable];
        v.extend(remaining.split_whitespace());
        v
    };

    // Handle Flatpak wrapper
    if executable.ends_with("/flatpak-spawn") || executable == "flatpak" {
        // Try to extract the actual app from the command
        // Flatpak commands look like: flatpak run org.mozilla.firefox
        if let Some(app_pos) = parts.iter().position(|&p| p == "run")
            && let Some(app_id) = parts.get(app_pos + 1)
        {
            // For Flatpak apps, try to find the actual executable
            // Check common locations for the app's files
            let app_id = app_id.trim_matches('"').trim_matches('\'');
            let base_paths = [
                format!("/var/lib/flatpak/app/{app_id}/current/active/files/bin"),
                format!("/var/lib/flatpak/app/{app_id}/current/active/files/lib"),
                home::home_dir()
                    .map(|h| {
                        format!(
                            "{}/.local/share/flatpak/app/{app_id}/current/active/files/bin",
                            h.display()
                        )
                    })
                    .unwrap_or_default(),
            ];

            // Try to find the executable in standard locations
            for base_path in &base_paths {
                if base_path.is_empty() {
                    continue;
                }
                let base = PathBuf::from(base_path);
                if let Ok(entries) = std::fs::read_dir(&base) {
                    // Look for the first executable file (simple heuristic)
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() {
                            // Check if file is executable
                            #[cfg(unix)]
                            {
                                use std::os::unix::fs::PermissionsExt;
                                if let Ok(metadata) = std::fs::metadata(&path)
                                    && metadata.permissions().mode() & 0o111 != 0
                                {
                                    return path;
                                }
                            }
                        }
                    }
                }
            }

            // Fallback: return flatpak command format that can be used for identification
            return PathBuf::from(format!("flatpak run {app_id}"));
        }
    }

    // Handle Snap wrapper
    if executable.contains("/snap/") {
        // Snap paths like /snap/bin/firefox
        return PathBuf::from(executable);
    }

    // Absolute path
    if executable.starts_with('/') {
        return PathBuf::from(executable);
    }

    // Relative path - try to resolve via PATH
    which::which(executable).unwrap_or_else(|_| PathBuf::from(executable))
}

fn should_skip_exec_token(token: &str) -> bool {
    if token.is_empty() {
        return true;
    }

    if token.contains('=') && !token.contains('/') {
        return true;
    }

    matches!(
        token,
        "env"
            | "nohup"
            | "setsid"
            | "sudo"
            | "dbus-launch"
            | "flatpak-spawn"
            | "snap"
            | "sh"
            | "bash"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_exec_to_path_handles_simple_path() {
        assert_eq!(
            parse_exec_to_path("/usr/bin/firefox"),
            PathBuf::from("/usr/bin/firefox")
        );
    }

    #[test]
    fn parse_exec_to_path_handles_arguments() {
        assert_eq!(
            parse_exec_to_path("/usr/bin/firefox %u"),
            PathBuf::from("/usr/bin/firefox")
        );
    }

    #[test]
    fn parse_exec_to_path_handles_quoted_paths() {
        assert_eq!(
            parse_exec_to_path("\"/usr/bin/Google Chrome\" %U"),
            PathBuf::from("/usr/bin/Google Chrome")
        );
    }

    #[test]
    fn parse_exec_to_path_handles_snap() {
        assert_eq!(
            parse_exec_to_path("/snap/bin/firefox %u"),
            PathBuf::from("/snap/bin/firefox")
        );
    }

    #[test]
    fn parse_exec_to_path_skips_env_wrapper() {
        assert_eq!(
            parse_exec_to_path("env FOO=bar /usr/bin/firefox %u"),
            PathBuf::from("/usr/bin/firefox")
        );
    }

    #[test]
    fn parse_desktop_file_extracts_name_and_exec() {
        let content = r"[Desktop Entry]
Name=Firefox
Exec=/usr/bin/firefox %u
MimeType=x-scheme-handler/http;x-scheme-handler/https;
";

        let temp_file = std::env::temp_dir().join("test.desktop");
        std::fs::write(&temp_file, content).unwrap();

        let entry = parse_desktop_file(&temp_file).unwrap();
        assert_eq!(entry.name, "Firefox");
        assert_eq!(entry.exec, "/usr/bin/firefox %u");
        assert!(entry.is_http_handler);

        std::fs::remove_file(temp_file).ok();
    }

    #[test]
    fn parse_desktop_file_filters_non_http_handlers() {
        let content = r"[Desktop Entry]
Name=TextEditor
Exec=/usr/bin/editor %f
MimeType=text/plain;
";

        let temp_file = std::env::temp_dir().join("test-editor.desktop");
        std::fs::write(&temp_file, content).unwrap();

        let entry = parse_desktop_file(&temp_file).unwrap();
        assert!(!entry.is_http_handler);

        std::fs::remove_file(temp_file).ok();
    }

    #[test]
    fn get_application_directories_returns_system_dirs() {
        let dirs = get_application_directories();
        assert!(dirs.contains(&PathBuf::from("/usr/share/applications")));
    }
}
