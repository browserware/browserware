//! Chrome-family profile discovery.
//!
//! Parses a Chrome user data directory to discover browser profiles
//! from the `Local State` metadata file.

use browserware_types::{LaunchCapability, ProfileRef};

use crate::ProfileDiscovery;

/// Discover Chrome profiles by reading the `Local State` file in `data_dir`.
///
/// Returns a [`ProfileDiscovery`] with all found profiles and the derived
/// capability. When the file is missing or unreadable, a `launch_only`
/// capability is returned with an explanation in `limitations`.
#[must_use]
pub fn discover_chrome_profiles_from(data_dir: &std::path::Path) -> ProfileDiscovery {
    let path = data_dir.join("Local State");

    let contents = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(
                path = %path.display(),
                error = %e,
                "Could not read Chrome Local State"
            );
            return ProfileDiscovery {
                profiles: vec![],
                capability: LaunchCapability::launch_only(format!(
                    "Chrome profile metadata unreadable: {e}"
                )),
            };
        }
    };

    let value: serde_json::Value = match serde_json::from_str(&contents) {
        Ok(v) => v,
        Err(_) => {
            return ProfileDiscovery {
                profiles: vec![],
                capability: LaunchCapability::launch_only("Chrome Local State is not valid JSON"),
            };
        }
    };

    let Some(info_cache) = value["profile"]["info_cache"].as_object() else {
        return ProfileDiscovery {
            profiles: vec![],
            capability: LaunchCapability::launch_only(
                "Chrome Local State missing profile.info_cache",
            ),
        };
    };

    let mut profiles: Vec<ProfileRef> = info_cache
        .iter()
        .map(|(dir_name, meta)| ProfileRef {
            id: dir_name.clone(),
            display_name: meta["name"].as_str().unwrap_or(dir_name).to_string(),
        })
        .collect();

    // Sort: "Default" first, then alphabetically by id.
    profiles.sort_by(|a, b| match (a.id.as_str(), b.id.as_str()) {
        ("Default", _) => std::cmp::Ordering::Less,
        (_, "Default") => std::cmp::Ordering::Greater,
        _ => a.id.cmp(&b.id),
    });

    ProfileDiscovery {
        profiles,
        capability: LaunchCapability::full(),
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write as _;

    use super::*;

    fn fixture(name: &str) -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name)
    }

    #[test]
    fn chrome_multi_profiles() {
        let d = discover_chrome_profiles_from(&fixture("chrome_multi"));
        assert!(d.capability.profile_launchable);
        assert_eq!(d.profiles.len(), 2);
        assert_eq!(d.profiles[0].id, "Default");
        assert_eq!(d.profiles[0].display_name, "Personal");
        assert_eq!(d.profiles[1].id, "Profile 1");
        assert_eq!(d.profiles[1].display_name, "Work");
    }

    #[test]
    fn chrome_single_profile() {
        let d = discover_chrome_profiles_from(&fixture("chrome_single"));
        assert_eq!(d.profiles.len(), 1);
        assert_eq!(d.profiles[0].id, "Default");
        assert_eq!(d.profiles[0].display_name, "Personal");
    }

    #[test]
    fn chrome_missing_data_dir_returns_limitation() {
        let d = discover_chrome_profiles_from(std::path::Path::new("/nonexistent/chrome/dir"));
        assert!(!d.capability.profile_launchable);
        assert!(d.profiles.is_empty());
        assert!(!d.capability.limitations.is_empty());
    }

    #[test]
    fn chrome_malformed_json_returns_limitation() {
        let dir = tempfile::tempdir().unwrap();
        let local_state = dir.path().join("Local State");
        let mut f = std::fs::File::create(&local_state).unwrap();
        f.write_all(b"not json").unwrap();
        drop(f);

        let d = discover_chrome_profiles_from(dir.path());
        assert!(!d.capability.profile_launchable);
        assert!(d.profiles.is_empty());
        assert!(!d.capability.limitations.is_empty());
    }
}
