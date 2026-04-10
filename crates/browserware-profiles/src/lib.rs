//! Browser profile management for the browserware ecosystem.
//!
//! This crate provides profile discovery for Chrome-family browsers.
//! Additional browser families will be added in future milestones.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod chrome;

pub use chrome::discover_chrome_profiles_from;

/// The result of profile discovery for a single browser.
#[derive(Debug, Clone)]
pub struct ProfileDiscovery {
    /// Discovered profiles. Empty when the browser has no profile support or
    /// when metadata is inaccessible.
    pub profiles: Vec<browserware_types::ProfileRef>,
    /// Capability flags derived from the discovery result.
    pub capability: browserware_types::LaunchCapability,
}
