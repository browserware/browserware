//! Platform-specific browser detection implementations.
//!
//! This module routes to the appropriate platform implementation based on
//! the target operating system at compile time.

// Allow pub items from private submodules: they are public within this module
// and re-exported at the crate level from here.
#![allow(unreachable_pub)]

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

// Re-export the current platform's implementation
#[cfg(target_os = "macos")]
pub use macos::{detect_browsers, detect_default_browser};

#[cfg(target_os = "windows")]
pub use windows::{detect_browsers, detect_default_browser};

#[cfg(target_os = "linux")]
pub use linux::{detect_browsers, detect_default_browser};

// Fallback for unsupported platforms
#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
pub fn detect_browsers() -> Vec<browserware_types::Browser> {
    tracing::warn!("Browser detection not implemented for this platform");
    Vec::new()
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
pub fn detect_default_browser() -> Option<browserware_types::Browser> {
    tracing::warn!("Default browser detection not implemented for this platform");
    None
}
