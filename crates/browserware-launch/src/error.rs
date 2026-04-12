//! Launch error types.

use std::path::PathBuf;
use std::process::ExitStatus;

use thiserror::Error;

/// Errors from [`crate::launch`] and [`crate::build_command`].
#[derive(Debug, Error)]
pub enum LaunchError {
    /// Context cannot be launched (see `limitations` from discovery).
    #[error("context is not launchable: {limitations:?}")]
    NotLaunchable {
        /// Human-readable reasons from [`browserware_types::LaunchCapability`].
        limitations: Vec<String>,
    },
    /// No URLs were passed.
    #[error("no URLs to open")]
    EmptyUrls,
    /// Executable path does not exist (direct launch path).
    #[error("browser executable not found: {0}")]
    ExecutableNotFound(PathBuf),
    /// Failed to spawn the browser or run `open`.
    #[error("failed to launch browser: {0}")]
    Io(#[from] std::io::Error),
    /// Browser exited with a non-zero status.
    #[error("browser process exited with status {0}")]
    ProcessFailed(ExitStatus),
}
