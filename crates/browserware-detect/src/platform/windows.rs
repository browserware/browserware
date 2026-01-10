//! Windows browser detection using the Registry.
//!
//! Detection strategy:
//! 1. Enumerate `HKLM\SOFTWARE\Clients\StartMenuInternet` subkeys
//! 2. For each subkey:
//!    a. Read `shell\open\command` for executable path
//!    b. Match against `KNOWN_BROWSERS` or derive metadata
//! 3. Check `HKCU\...\UrlAssociations\http\UserChoice\ProgId` for default

use browserware_types::Browser;

/// Detect all installed browsers on Windows.
#[tracing::instrument(level = "debug")]
pub fn detect_browsers() -> Vec<Browser> {
    tracing::debug!("Windows browser detection not yet implemented");
    // TODO: Implement in Week 3
    Vec::new()
}

/// Detect the default browser on Windows.
#[tracing::instrument(level = "debug")]
pub fn detect_default_browser() -> Option<Browser> {
    tracing::debug!("Windows default browser detection not yet implemented");
    // TODO: Implement in Week 3
    None
}
