//! Linux browser detection using XDG desktop files.
//!
//! Detection strategy:
//! 1. Scan XDG application directories for `.desktop` files
//! 2. Filter files with `MimeType=` containing `x-scheme-handler/http`
//! 3. Parse `Exec=` field for executable path
//! 4. Match against `KNOWN_BROWSERS` or derive metadata
//! 5. Use `xdg-settings get default-web-browser` for default

use browserware_types::Browser;

/// Detect all installed browsers on Linux.
#[tracing::instrument(level = "debug")]
pub fn detect_browsers() -> Vec<Browser> {
    tracing::debug!("Linux browser detection not yet implemented");
    // TODO: Implement in Week 4
    Vec::new()
}

/// Detect the default browser on Linux.
#[tracing::instrument(level = "debug")]
pub fn detect_default_browser() -> Option<Browser> {
    tracing::debug!("Linux default browser detection not yet implemented");
    // TODO: Implement in Week 4
    None
}
