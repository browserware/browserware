//! Shared types for the browserware ecosystem.
//!
//! This crate provides common types, traits, and error definitions
//! used across all browserware crates.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod browser;
mod error;
mod variant;

pub use browser::{Browser, BrowserFamily, BrowserId};
pub use error::{Error, Result};
pub use variant::{BrowserVariant, ChromiumChannel, FirefoxChannel, WebKitChannel};

// Re-export url for convenience
pub use url::Url;
