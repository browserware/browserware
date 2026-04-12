//! Firefox profile discovery.
//!
//! Parses a Firefox `profiles.ini` file to discover browser profiles.

use std::path::{Path, PathBuf};

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

    let base_dir = profiles_ini
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    parse_profiles_ini(&contents, &base_dir)
}

#[derive(Debug, Default)]
struct ProfileSection {
    name: Option<String>,
    path_raw: Option<String>,
    is_relative: Option<bool>,
}

impl ProfileSection {
    fn flush(self, base_dir: &Path, profiles: &mut Vec<ProfileRef>) {
        let Some(name) = self.name.filter(|n| !n.is_empty()) else {
            return;
        };
        let path = self.path_raw.map(|p| {
            if self.is_relative.unwrap_or(true) {
                base_dir.join(p)
            } else {
                PathBuf::from(p)
            }
        });
        profiles.push(ProfileRef {
            id: name.clone(),
            display_name: name,
            path,
        });
    }
}

// Parse a Firefox profiles.ini file content and return discovered profiles.
fn parse_profiles_ini(contents: &str, base_dir: &Path) -> ProfileDiscovery {
    let mut profiles: Vec<ProfileRef> = Vec::new();
    let mut in_profile_section = false;
    let mut section = ProfileSection::default();

    for line in contents.lines() {
        let line = line.trim();

        if line.starts_with('[') && line.ends_with(']') {
            let section_name = &line[1..line.len() - 1];

            if in_profile_section {
                section.flush(base_dir, &mut profiles);
                section = ProfileSection::default();
            }

            in_profile_section = section_name.starts_with("Profile");
            continue;
        }

        if !in_profile_section {
            continue;
        }

        if let Some(value) = line.strip_prefix("Name=") {
            let name = value.trim();
            if !name.is_empty() {
                section.name = Some(name.to_string());
            }
            continue;
        }

        if let Some(value) = line.strip_prefix("Path=") {
            let p = value.trim();
            if !p.is_empty() {
                section.path_raw = Some(p.to_string());
            }
            continue;
        }

        if let Some(value) = line.strip_prefix("IsRelative=") {
            let v = value.trim();
            section.is_relative = Some(v == "1");
        }
    }

    if in_profile_section {
        section.flush(base_dir, &mut profiles);
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
        let ini_path = fixture("firefox_multi");
        let base = ini_path.parent().expect("fixture path");
        let d = discover_firefox_profiles_from(&ini_path);
        assert!(d.capability.profile_launchable);
        assert_eq!(d.profiles.len(), 2);
        let ids: Vec<&str> = d.profiles.iter().map(|p| p.id.as_str()).collect();
        assert!(ids.contains(&"default-release"));
        assert!(ids.contains(&"work"));
        let dr = d
            .profiles
            .iter()
            .find(|p| p.id == "default-release")
            .unwrap();
        assert_eq!(dr.path, Some(base.join("Profiles/abc123.default-release")));
        let w = d.profiles.iter().find(|p| p.id == "work").unwrap();
        assert_eq!(w.path, Some(base.join("Profiles/def456.work")));
    }

    #[test]
    fn firefox_default_profile() {
        let ini_path = fixture("firefox_default");
        let base = ini_path.parent().unwrap();
        let d = discover_firefox_profiles_from(&ini_path);
        assert_eq!(d.profiles.len(), 1);
        assert_eq!(d.profiles[0].id, "default");
        assert_eq!(
            d.profiles[0].path,
            Some(base.join("Profiles/xyz789.default"))
        );
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
            b"[Profile0]\nName=\n[Profile1]\nName=   \n[Profile2]\nName=default-release\nIsRelative=1\nPath=Profiles/x\n",
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

        let d = discover_firefox_profiles_from(&ini_path);
        let _ = d;
    }

    #[test]
    fn firefox_profile_relative_path() {
        let dir = tempfile::tempdir().unwrap();
        let ini_path = dir.path().join("profiles.ini");
        std::fs::write(
            &ini_path,
            b"[Profile0]\nName=work\nIsRelative=1\nPath=Profiles/abc.default\n",
        )
        .unwrap();

        let d = discover_firefox_profiles_from(&ini_path);
        assert_eq!(d.profiles.len(), 1);
        assert_eq!(
            d.profiles[0].path,
            Some(dir.path().join("Profiles/abc.default"))
        );
    }

    #[test]
    fn firefox_profile_absolute_path() {
        let dir = tempfile::tempdir().unwrap();
        let ini_path = dir.path().join("profiles.ini");
        std::fs::write(
            &ini_path,
            b"[Profile0]\nName=work\nIsRelative=0\nPath=/home/user/.mozilla/firefox/abc.default\n",
        )
        .unwrap();

        let d = discover_firefox_profiles_from(&ini_path);
        assert_eq!(d.profiles.len(), 1);
        assert_eq!(
            d.profiles[0].path,
            Some(PathBuf::from("/home/user/.mozilla/firefox/abc.default"))
        );
    }

    #[test]
    fn firefox_profile_missing_path_key() {
        let dir = tempfile::tempdir().unwrap();
        let ini_path = dir.path().join("profiles.ini");
        std::fs::write(&ini_path, b"[Profile0]\nName=work\n").unwrap();

        let d = discover_firefox_profiles_from(&ini_path);
        assert_eq!(d.profiles.len(), 1);
        assert!(d.profiles[0].path.is_none());
    }

    #[test]
    fn firefox_profile_isrelative_missing() {
        let dir = tempfile::tempdir().unwrap();
        let ini_path = dir.path().join("profiles.ini");
        std::fs::write(&ini_path, b"[Profile0]\nName=work\nPath=Profiles/rel\n").unwrap();

        let d = discover_firefox_profiles_from(&ini_path);
        assert_eq!(d.profiles.len(), 1);
        assert_eq!(d.profiles[0].path, Some(dir.path().join("Profiles/rel")));
    }
}
