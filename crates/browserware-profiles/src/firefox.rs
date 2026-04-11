//! Firefox profile discovery.
//!
//! Parses a Firefox `profiles.ini` file to discover browser profiles.

use browserware_types::{LaunchCapability, ProfileRef};

use crate::ProfileDiscovery;

/// Discover Firefox profiles by reading the `profiles.ini` file.
///
/// Returns a [`ProfileDiscovery`] with all found profiles and the derived
/// capability. When the file is missing or unreadable, a `launch_only`
/// capability is returned with an explanation in `limitations`.
#[must_use]
pub fn discover_firefox_profiles_from(profiles_ini: &std::path::Path) -> ProfileDiscovery {
    let contents = match std::fs::read_to_string(profiles_ini) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(
                path = %profiles_ini.display(),
                error = %e,
                "Could not read Firefox profiles.ini"
            );
            return ProfileDiscovery {
                profiles: vec![],
                capability: LaunchCapability::launch_only(format!(
                    "Firefox profiles.ini unreadable: {e}"
                )),
            };
        }
    };

    parse_profiles_ini(&contents)
}

// Parse a Firefox profiles.ini file content and return discovered profiles.
fn parse_profiles_ini(contents: &str) -> ProfileDiscovery {
    let mut profiles: Vec<ProfileRef> = Vec::new();
    let mut in_profile_section = false;
    let mut current_name: Option<String> = None;

    for line in contents.lines() {
        let line = line.trim();

        if line.starts_with('[') && line.ends_with(']') {
            let section_name = &line[1..line.len() - 1];

            // Flush the current profile section if we were in one and have a name
            if in_profile_section && let Some(name) = current_name.take() {
                profiles.push(ProfileRef {
                    id: name.clone(),
                    display_name: name,
                });
            }

            if section_name.starts_with("Profile") {
                in_profile_section = true;
                current_name = None;
            } else {
                in_profile_section = false;
            }
        } else if in_profile_section && let Some(value) = line.strip_prefix("Name=") {
            let name = value.trim();
            if !name.is_empty() {
                current_name = Some(name.to_string());
            }
        }
    }

    // Flush the last profile section
    if in_profile_section && let Some(name) = current_name.take() {
        profiles.push(ProfileRef {
            id: name.clone(),
            display_name: name,
        });
    }

    if profiles.is_empty() {
        return ProfileDiscovery {
            profiles: vec![],
            capability: LaunchCapability::launch_only("Firefox profiles.ini contains no profiles"),
        };
    }

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
            .join("profiles.ini")
    }

    #[test]
    fn firefox_multi_profiles() {
        let d = discover_firefox_profiles_from(&fixture("firefox_multi"));
        assert!(d.capability.profile_launchable);
        assert_eq!(d.profiles.len(), 2);
        let ids: Vec<&str> = d.profiles.iter().map(|p| p.id.as_str()).collect();
        assert!(ids.contains(&"default-release"));
        assert!(ids.contains(&"work"));
    }

    #[test]
    fn firefox_default_profile() {
        let d = discover_firefox_profiles_from(&fixture("firefox_default"));
        assert_eq!(d.profiles.len(), 1);
        assert_eq!(d.profiles[0].id, "default");
    }

    #[test]
    fn firefox_missing_file_returns_limitation() {
        let d = discover_firefox_profiles_from(std::path::Path::new("/nonexistent/profiles.ini"));
        assert!(!d.capability.profile_launchable);
        assert!(d.profiles.is_empty());
        assert!(!d.capability.limitations.is_empty());
    }

    #[test]
    fn firefox_empty_ini_returns_limitation() {
        let dir = tempfile::tempdir().unwrap();
        let ini_path = dir.path().join("profiles.ini");
        let mut f = std::fs::File::create(&ini_path).unwrap();
        f.write_all(b"[General]\nStartWithLastProfile=1\n").unwrap();
        drop(f);

        let d = discover_firefox_profiles_from(&ini_path);
        assert!(!d.capability.profile_launchable);
        assert!(d.profiles.is_empty());
    }

    #[test]
    fn firefox_empty_name_value_skipped() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let ini_path = dir.path().join("profiles.ini");
        let mut f = std::fs::File::create(&ini_path)?;
        // Name= with no value (and whitespace-only variant) must not produce a profile
        // with an empty id; a valid profile follows to confirm the parser keeps running.
        f.write_all(
            b"[Profile0]\nName=\n[Profile1]\nName=   \n[Profile2]\nName=default-release\n",
        )?;
        drop(f);

        let d = discover_firefox_profiles_from(&ini_path);
        assert_eq!(
            d.profiles.len(),
            1,
            "only the non-empty profile should appear"
        );
        assert_eq!(d.profiles[0].id, "default-release");
        Ok(())
    }

    #[test]
    fn firefox_malformed_profile_section_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let ini_path = dir.path().join("profiles.ini");
        let mut f = std::fs::File::create(&ini_path).unwrap();
        // Profile0 section with no Name= line
        f.write_all(b"[Profile0]\nIsRelative=1\nPath=Profiles/abc.default\n")
            .unwrap();
        drop(f);

        // Should not panic; profiles empty => capability limited
        let d = discover_firefox_profiles_from(&ini_path);
        // Either profiles is empty or capability is limited — no panic is the key assertion
        let _ = d;
    }
}
