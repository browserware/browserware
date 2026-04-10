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

#[test]
fn firefox_multi_profiles_integration() {
    let d = browserware_profiles::discover_firefox_profiles_from(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/firefox_multi/profiles.ini"),
    );
    assert!(d.capability.profile_launchable);
    assert_eq!(d.profiles.len(), 2);
    let ids: Vec<&str> = d.profiles.iter().map(|p| p.id.as_str()).collect();
    assert!(ids.contains(&"default-release"));
    assert!(ids.contains(&"work"));
}

#[test]
fn firefox_default_only_integration() {
    let d = browserware_profiles::discover_firefox_profiles_from(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/firefox_default/profiles.ini"),
    );
    assert!(d.capability.profile_launchable);
    assert_eq!(d.profiles.len(), 1);
    assert_eq!(d.profiles[0].id, "default");
}

#[test]
fn firefox_missing_ini_returns_limitation() {
    let d = browserware_profiles::discover_firefox_profiles_from(
        &PathBuf::from("/nonexistent/firefox/profiles.ini"),
    );
    assert!(!d.capability.profile_launchable);
    assert!(d.profiles.is_empty());
}

#[test]
fn discover_profiles_safari_no_profile_launch() {
    use browserware_types::{Browser, BrowserVariant, WebKitChannel};
    let browser = Browser::new("safari", "Safari", PathBuf::from("/Applications/Safari.app"))
        .with_variant(BrowserVariant::WebKit(WebKitChannel::Stable));
    let d = browserware_profiles::discover_profiles(&browser);
    assert!(!d.capability.profile_launchable);
    assert!(d.profiles.is_empty());
    assert!(!d.capability.limitations.is_empty());
}
