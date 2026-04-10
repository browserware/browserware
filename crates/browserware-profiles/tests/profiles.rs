//! Integration tests for browserware-profiles.

use std::path::PathBuf;

#[test]
fn chrome_multi_profiles_integration() {
    let d = browserware_profiles::discover_chrome_profiles_from(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/chrome_multi"),
    );
    assert!(d.capability.profile_launchable);
    assert_eq!(d.profiles.len(), 2);
    assert_eq!(d.profiles[0].id, "Default");
    assert_eq!(d.profiles[1].id, "Profile 1");
}

#[test]
fn chrome_inaccessible_returns_limitation() {
    let d = browserware_profiles::discover_chrome_profiles_from(
        &PathBuf::from("/nonexistent/chrome/dir"),
    );
    assert!(!d.capability.profile_launchable);
    assert!(d.profiles.is_empty());
    assert!(!d.capability.limitations.is_empty());
}
