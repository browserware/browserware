
### Known Browser Registry

A central registry of browser metadata to match detected installations:

```rust
// src/registry.rs

pub struct BrowserMeta {
    /// Canonical ID used in configs (e.g., "chrome", "firefox-nightly")
    pub id: &'static str,
    /// Display name
    pub name: &'static str,
    /// Browser variant
    pub variant: BrowserVariant,
    /// macOS bundle identifiers
    pub macos_bundle_ids: &'static [&'static str],
    /// Windows registry keys (under StartMenuInternet)
    pub windows_registry_keys: &'static [&'static str],
    /// Linux desktop file names (without .desktop)
    pub linux_desktop_ids: &'static [&'static str],
}

pub static KNOWN_BROWSERS: &[BrowserMeta] = &[
    // Chromium family
    BrowserMeta {
        id: "chrome",
        name: "Google Chrome",
        variant: BrowserVariant::Chromium(ChromiumChannel::Stable),
        macos_bundle_ids: &["com.google.Chrome"],
        windows_registry_keys: &["Google Chrome"],
        linux_desktop_ids: &["google-chrome", "google-chrome-stable"],
    },
    BrowserMeta {
        id: "chrome-beta",
        name: "Google Chrome Beta",
        variant: BrowserVariant::Chromium(ChromiumChannel::Beta),
        macos_bundle_ids: &["com.google.Chrome.beta"],
        windows_registry_keys: &["Google Chrome Beta"],
        linux_desktop_ids: &["google-chrome-beta"],
    },
    BrowserMeta {
        id: "chrome-dev",
        name: "Google Chrome Dev",
        variant: BrowserVariant::Chromium(ChromiumChannel::Dev),
        macos_bundle_ids: &["com.google.Chrome.dev"],
        windows_registry_keys: &["Google Chrome Dev"],
        linux_desktop_ids: &["google-chrome-unstable"],
    },
    BrowserMeta {
        id: "chrome-canary",
        name: "Google Chrome Canary",
        variant: BrowserVariant::Chromium(ChromiumChannel::Canary),
        macos_bundle_ids: &["com.google.Chrome.canary"],
        windows_registry_keys: &["Google Chrome Canary"],
        linux_desktop_ids: &[],  // No Canary on Linux
    },
    BrowserMeta {
        id: "edge",
        name: "Microsoft Edge",
        variant: BrowserVariant::Chromium(ChromiumChannel::Stable),
        macos_bundle_ids: &["com.microsoft.edgemac"],
        windows_registry_keys: &["Microsoft Edge"],
        linux_desktop_ids: &["microsoft-edge", "microsoft-edge-stable"],
    },
    BrowserMeta {
        id: "brave",
        name: "Brave Browser",
        variant: BrowserVariant::Chromium(ChromiumChannel::Stable),
        macos_bundle_ids: &["com.brave.Browser"],
        windows_registry_keys: &["BraveSoftware Brave-Browser"],
        linux_desktop_ids: &["brave-browser", "brave"],
    },
    BrowserMeta {
        id: "arc",
        name: "Arc",
        variant: BrowserVariant::Single(BrowserFamily::Chromium),
        macos_bundle_ids: &["company.thebrowser.Browser"],
        windows_registry_keys: &["Arc"],
        linux_desktop_ids: &[],  // No Arc on Linux
    },
    BrowserMeta {
        id: "vivaldi",
        name: "Vivaldi",
        variant: BrowserVariant::Chromium(ChromiumChannel::Stable),
        macos_bundle_ids: &["com.vivaldi.Vivaldi"],
        windows_registry_keys: &["Vivaldi"],
        linux_desktop_ids: &["vivaldi", "vivaldi-stable"],
    },
    BrowserMeta {
        id: "opera",
        name: "Opera",
        variant: BrowserVariant::Chromium(ChromiumChannel::Stable),
        macos_bundle_ids: &["com.operasoftware.Opera"],
        windows_registry_keys: &["Opera Stable"],
        linux_desktop_ids: &["opera"],
    },
    
    // Firefox family
    BrowserMeta {
        id: "firefox",
        name: "Firefox",
        variant: BrowserVariant::Firefox(FirefoxChannel::Stable),
        macos_bundle_ids: &["org.mozilla.firefox"],
        windows_registry_keys: &["Firefox"],
        linux_desktop_ids: &["firefox", "firefox-esr"],
    },
    BrowserMeta {
        id: "firefox-beta",
        name: "Firefox Beta",
        variant: BrowserVariant::Firefox(FirefoxChannel::Beta),
        macos_bundle_ids: &["org.mozilla.firefoxbeta"],
        windows_registry_keys: &["Firefox Beta"],
        linux_desktop_ids: &["firefox-beta"],
    },
    BrowserMeta {
        id: "firefox-dev",
        name: "Firefox Developer Edition",
        variant: BrowserVariant::Firefox(FirefoxChannel::Dev),
        macos_bundle_ids: &["org.mozilla.firefoxdeveloperedition"],
        windows_registry_keys: &["Firefox Developer Edition"],
        linux_desktop_ids: &["firefox-developer-edition"],
    },
    BrowserMeta {
        id: "firefox-nightly",
        name: "Firefox Nightly",
        variant: BrowserVariant::Firefox(FirefoxChannel::Nightly),
        macos_bundle_ids: &["org.mozilla.nightly"],
        windows_registry_keys: &["Firefox Nightly"],
        linux_desktop_ids: &["firefox-nightly"],
    },
    BrowserMeta {
        id: "librewolf",
        name: "LibreWolf",
        variant: BrowserVariant::Single(BrowserFamily::Firefox),
        macos_bundle_ids: &["io.gitlab.LibreWolf"],
        windows_registry_keys: &["LibreWolf"],
        linux_desktop_ids: &["librewolf"],
    },
    BrowserMeta {
        id: "waterfox",
        name: "Waterfox",
        variant: BrowserVariant::Single(BrowserFamily::Firefox),
        macos_bundle_ids: &["net.waterfox.waterfox"],
        windows_registry_keys: &["Waterfox"],
        linux_desktop_ids: &["waterfox"],
    },
    
    // WebKit family
    BrowserMeta {
        id: "safari",
        name: "Safari",
        variant: BrowserVariant::WebKit(WebKitChannel::Stable),
        macos_bundle_ids: &["com.apple.Safari"],
        windows_registry_keys: &[],  // No Safari on Windows
        linux_desktop_ids: &[],      // No Safari on Linux
    },
    BrowserMeta {
        id: "safari-preview",
        name: "Safari Technology Preview",
        variant: BrowserVariant::WebKit(WebKitChannel::TechnologyPreview),
        macos_bundle_ids: &["com.apple.SafariTechnologyPreview"],
        windows_registry_keys: &[],
        linux_desktop_ids: &[],
    },
    BrowserMeta {
        id: "gnome-web",
        name: "GNOME Web",
        variant: BrowserVariant::Single(BrowserFamily::WebKit),
        macos_bundle_ids: &[],
        windows_registry_keys: &[],
        linux_desktop_ids: &["org.gnome.Epiphany", "epiphany"],
    },
];
```

---

## Platform Implementation

### Week 1-2: macOS Implementation

**File: `src/platform/macos.rs`**

**Approach**:
1. Use `LSCopyApplicationURLsForBundleIdentifier` to find installed apps by bundle ID
2. Parse `Info.plist` to extract version
3. Get executable path from app bundle structure
4. Use `LSCopyDefaultHandlerForURLScheme` for default browser

**Dependencies**:
```toml
[target.'cfg(target_os = "macos")'.dependencies]
core-foundation = "0.10"
core-services = "0.2"  # or use objc2-core-services when stable
```

**Key Functions**:

```rust
// src/platform/macos.rs

use core_foundation::bundle::CFBundle;
use core_foundation::string::CFString;
use core_foundation::url::CFURL;

/// Detect all installed browsers on macOS.
pub fn detect_browsers() -> Vec<Browser> {
    let mut browsers = Vec::new();
    
    for meta in KNOWN_BROWSERS {
        if meta.macos_bundle_ids.is_empty() {
            continue;
        }
        
        for bundle_id in meta.macos_bundle_ids {
            if let Some(browser) = detect_by_bundle_id(bundle_id, meta) {
                browsers.push(browser);
                break;  // Found this browser, move to next
            }
        }
    }
    
    browsers
}

fn detect_by_bundle_id(bundle_id: &str, meta: &BrowserMeta) -> Option<Browser> {
    // Use LSCopyApplicationURLsForBundleIdentifier
    let cf_bundle_id = CFString::new(bundle_id);
    let urls = unsafe {
        core_services::LSCopyApplicationURLsForBundleIdentifier(
            cf_bundle_id.as_concrete_TypeRef(),
            std::ptr::null_mut(),
        )
    };
    
    if urls.is_null() {
        return None;
    }
    
    // Get first (usually only) URL
    let app_url: CFURL = /* extract from CFArray */;
    let app_path = app_url.to_path()?;
    
    // Parse Info.plist for version
    let plist_path = app_path.join("Contents/Info.plist");
    let version = extract_version_from_plist(&plist_path);
    
    // Get executable path
    let executable = app_path
        .join("Contents/MacOS")
        .join(meta.name.split_whitespace().next().unwrap_or("browser"));
    
    Some(Browser {
        id: BrowserId::new(meta.id),
        name: meta.name.to_string(),
        variant: meta.variant,
        version,
        executable,
        bundle_id: Some(bundle_id.to_string()),
    })
}

pub fn detect_default_browser() -> Option<Browser> {
    // LSCopyDefaultHandlerForURLScheme for "http"
    let http_scheme = CFString::new("http");
    let default_bundle_id = unsafe {
        core_services::LSCopyDefaultHandlerForURLScheme(
            http_scheme.as_concrete_TypeRef()
        )
    };
    
    if default_bundle_id.is_null() {
        return None;
    }
    
    let bundle_id_str: String = /* convert CFString */;
    
    // Find matching browser in our registry
    for meta in KNOWN_BROWSERS {
        if meta.macos_bundle_ids.contains(&bundle_id_str.as_str()) {
            return detect_by_bundle_id(&bundle_id_str, meta);
        }
    }
    
    None
}
```

**Version Extraction**:

```rust
fn extract_version_from_plist(plist_path: &Path) -> Option<String> {
    let plist = std::fs::read(plist_path).ok()?;
    
    // Parse plist (use plist crate or manual XML parsing)
    // Look for CFBundleShortVersionString or CFBundleVersion
    
    // Simple approach: use `plutil -convert json` or parse XML
    let content = std::str::from_utf8(&plist).ok()?;
    
    // Extract version using regex or XML parsing
    extract_plist_key(content, "CFBundleShortVersionString")
        .or_else(|| extract_plist_key(content, "CFBundleVersion"))
}
```

### Week 3-4: Windows Implementation

**File: `src/platform/windows.rs`**

**Approach**:
1. Enumerate `HKLM\SOFTWARE\Clients\StartMenuInternet` for registered browsers
2. Read `DefaultIcon` and `shell\open\command` subkeys for executable path
3. Parse executable to extract version (PE header or file version info)
4. Check `HKCU\Software\Microsoft\Windows\Shell\Associations\UrlAssociations\http\UserChoice` for default

**Dependencies**:
```toml
[target.'cfg(target_os = "windows")'.dependencies]
windows = { version = "0.58", features = [
    "Win32_System_Registry",
    "Win32_Storage_FileSystem",
] }
```

**Key Functions**:

```rust
// src/platform/windows.rs

use windows::Win32::System::Registry::*;

const BROWSERS_KEY: &str = r"SOFTWARE\Clients\StartMenuInternet";

pub fn detect_browsers() -> Vec<Browser> {
    let mut browsers = Vec::new();
    
    // Open registry key
    let hkey = open_registry_key(HKEY_LOCAL_MACHINE, BROWSERS_KEY)?;
    
    // Enumerate subkeys (browser names)
    for subkey_name in enumerate_subkeys(&hkey) {
        if let Some(browser) = detect_browser_from_registry(&hkey, &subkey_name) {
            browsers.push(browser);
        }
    }
    
    browsers
}

fn detect_browser_from_registry(parent: &HKEY, name: &str) -> Option<Browser> {
    // Find matching metadata
    let meta = KNOWN_BROWSERS.iter()
        .find(|m| m.windows_registry_keys.contains(&name))?;
    
    // Read shell\open\command for executable path
    let command_key = format!(r"{}\{}\shell\open\command", BROWSERS_KEY, name);
    let command = read_registry_string(HKEY_LOCAL_MACHINE, &command_key, "")?;
    
    // Parse command to extract executable path
    let executable = parse_executable_from_command(&command)?;
    
    // Extract version from executable
    let version = extract_version_from_executable(&executable);
    
    Some(Browser {
        id: BrowserId::new(meta.id),
        name: meta.name.to_string(),
        variant: meta.variant,
        version,
        executable,
        bundle_id: None,
    })
}

fn parse_executable_from_command(command: &str) -> Option<PathBuf> {
    // Command format: "C:\Path\To\Browser.exe" --args
    // or: C:\Path\To\Browser.exe --args
    
    if command.starts_with('"') {
        // Quoted path
        let end = command[1..].find('"')?;
        Some(PathBuf::from(&command[1..=end]))
    } else {
        // Unquoted path (ends at first space or end of string)
        let end = command.find(' ').unwrap_or(command.len());
        Some(PathBuf::from(&command[..end]))
    }
}

pub fn detect_default_browser() -> Option<Browser> {
    // Read UserChoice for http
    let user_choice_key = r"Software\Microsoft\Windows\Shell\Associations\UrlAssociations\http\UserChoice";
    let prog_id = read_registry_string(HKEY_CURRENT_USER, user_choice_key, "ProgId")?;
    
    // Map ProgId back to browser
    // e.g., "ChromeHTML" -> chrome, "FirefoxURL" -> firefox
    match_prog_id_to_browser(&prog_id)
}
```

### Week 4-5: Linux Implementation

**File: `src/platform/linux.rs`**

**Approach**:
1. Parse `.desktop` files from XDG directories
2. Look for `x-scheme-handler/http` in mimetypes
3. Extract `Exec` command and `Name` from desktop files
4. Use `xdg-settings get default-web-browser` for default

**Dependencies**:
```toml
[target.'cfg(target_os = "linux")'.dependencies]
xdg = "2.5"
```

**XDG Desktop File Locations**:
- `/usr/share/applications/`
- `/usr/local/share/applications/`
- `~/.local/share/applications/`
- Flatpak: `~/.local/share/flatpak/exports/share/applications/`
- Snap: `/var/lib/snapd/desktop/applications/`

**Key Functions**:

```rust
// src/platform/linux.rs

use std::collections::HashSet;
use xdg::BaseDirectories;

pub fn detect_browsers() -> Vec<Browser> {
    let mut browsers = Vec::new();
    let mut seen_ids = HashSet::new();
    
    let xdg = BaseDirectories::new().ok();
    
    for dir in get_desktop_dirs(&xdg) {
        for entry in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "desktop").unwrap_or(false)
                && let Some(browser) = parse_desktop_file(&path)
                && !seen_ids.contains(&browser.id)
            {
                seen_ids.insert(browser.id.clone());
                browsers.push(browser);
            }
        }
    }
    
    browsers
}

fn get_desktop_dirs(xdg: &Option<BaseDirectories>) -> Vec<PathBuf> {
    let mut dirs = vec![
        PathBuf::from("/usr/share/applications"),
        PathBuf::from("/usr/local/share/applications"),
    ];
    
    if let Some(xdg) = xdg {
        if let Ok(data_home) = xdg.get_data_home() {
            dirs.push(data_home.join("applications"));
        }
    }
    
    // Flatpak
    if let Some(home_dir) = home::home_dir() {
        dirs.push(home_dir.join(".local/share/flatpak/exports/share/applications"));
    }
    
    // Snap
    dirs.push(PathBuf::from("/var/lib/snapd/desktop/applications"));
    
    dirs
}

fn parse_desktop_file(path: &Path) -> Option<Browser> {
    let content = std::fs::read_to_string(path).ok()?;
    
    // Parse .desktop file (INI-like format)
    let mut name = None;
    let mut exec = None;
    let mut categories = String::new();
    
    let mut in_desktop_entry = false;
    for line in content.lines() {
        let line = line.trim();
        
        if line == "[Desktop Entry]" {
            in_desktop_entry = true;
            continue;
        }
        if line.starts_with('[') {
            in_desktop_entry = false;
            continue;
        }
        
        if in_desktop_entry
            && let Some((key, value)) = line.split_once('=')
        {
            match key {
                "Name" => name = Some(value.to_string()),
                "Exec" => exec = Some(value.to_string()),
                "Categories" => categories = value.to_string(),
                _ => {}
            }
        }
    }
    
    // Must have WebBrowser category or be a known browser
    let desktop_id = path.file_stem()?.to_str()?;
    
    // Try to match against known browsers
    let meta = KNOWN_BROWSERS.iter()
        .find(|m| m.linux_desktop_ids.contains(&desktop_id))?;
    
    let exec_cmd = exec?;
    let executable = parse_exec_to_path(&exec_cmd)?;
    
    // Extract version (try --version flag or parse binary)
    let version = extract_version_from_executable(&executable);
    
    Some(Browser {
        id: BrowserId::new(meta.id),
        name: meta.name.to_string(),
        variant: meta.variant,
        version,
        executable,
        bundle_id: None,
    })
}

fn parse_exec_to_path(exec: &str) -> Option<PathBuf> {
    // Exec format: /usr/bin/firefox %u
    // or: env VAR=value /usr/bin/browser %U
    
    let parts: Vec<&str> = exec.split_whitespace().collect();
    
    for part in parts {
        if part.starts_with('/') || part.starts_with("./") {
            return Some(PathBuf::from(part));
        }
        // Skip env, flatpak, snap wrappers
        if !part.contains('=') && !["env", "flatpak", "snap"].contains(&part) {
            // Try to resolve via which
            if let Ok(output) = std::process::Command::new("which")
                .arg(part)
                .output()
            {
                if output.status.success() {
                    let path = String::from_utf8_lossy(&output.stdout);
                    return Some(PathBuf::from(path.trim()));
                }
            }
        }
    }
    
    None
}

pub fn detect_default_browser() -> Option<Browser> {
    // Use xdg-settings
    let output = std::process::Command::new("xdg-settings")
        .args(["get", "default-web-browser"])
        .output()
        .ok()?;
    
    if !output.status.success() {
        return None;
    }
    
    let desktop_file = String::from_utf8_lossy(&output.stdout);
    let desktop_file = desktop_file.trim().trim_end_matches(".desktop");
    
    // Find in detected browsers
    detect_browsers()
        .into_iter()
        .find(|b| {
            KNOWN_BROWSERS.iter()
                .find(|m| m.id == b.id.0)
                .map(|m| m.linux_desktop_ids.contains(&desktop_file))
                .unwrap_or(false)
        })
}
```

### Week 5-6: Public API & CLI Integration

**File: `src/lib.rs`**

```rust
//! Browser detection for the browserware ecosystem.
//!
//! This crate provides cross-platform browser discovery, detecting installed
//! browsers, their versions, and the system's default browser.
//!
//! # Example
//!
//! ```no_run
//! use browserware_detect::{detect_browsers, detect_default_browser};
//!
//! // List all installed browsers
//! for browser in detect_browsers() {
//!     println!("{}: {} ({:?})", browser.id, browser.name, browser.variant);
//! }
//!
//! // Get the default browser
//! if let Some(default) = detect_default_browser() {
//!     println!("Default browser: {}", default.name);
//! }
//! ```

#![forbid(unsafe_code)]  // Note: May need to allow for FFI on some platforms
#![warn(missing_docs)]

mod registry;

#[cfg(target_os = "macos")]
mod platform {
    mod macos;
    pub use macos::*;
}

#[cfg(target_os = "windows")]
mod platform {
    mod windows;
    pub use windows::*;
}

#[cfg(target_os = "linux")]
mod platform {
    mod linux;
    pub use linux::*;
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
mod platform {
    use browserware_types::{Browser, BrowserFamily};
    
    pub fn detect_browsers() -> Vec<Browser> {
        Vec::new()
    }
    
    pub fn detect_default_browser() -> Option<Browser> {
        None
    }
}

pub use browserware_types::{Browser, BrowserFamily, BrowserId, BrowserVariant};
use registry::KNOWN_BROWSERS;

/// Detect all installed browsers on the system.
///
/// Returns a list of detected browsers with their metadata.
/// The list is not guaranteed to be in any particular order.
pub fn detect_browsers() -> Vec<Browser> {
    platform::detect_browsers()
}

/// Detect a specific browser by ID.
///
/// # Arguments
///
/// * `id` - The browser ID (e.g., "chrome", "firefox-nightly")
///
/// # Returns
///
/// The browser if found and installed, `None` otherwise.
pub fn detect_browser(id: &str) -> Option<Browser> {
    detect_browsers()
        .into_iter()
        .find(|b| b.id.0 == id)
}

/// Detect the system's default browser.
///
/// Returns the browser configured as the default HTTP/HTTPS handler.
pub fn detect_default_browser() -> Option<Browser> {
    platform::detect_default_browser()
}

/// Detect all browsers of a specific family.
///
/// # Arguments
///
/// * `family` - The browser engine family to filter by
pub fn detect_browsers_by_family(family: BrowserFamily) -> Vec<Browser> {
    detect_browsers()
        .into_iter()
        .filter(|b| b.variant.family() == family)
        .collect()
}
```

**CLI Integration** (`browserware-cli/src/main.rs`):

```rust
Commands::Browsers => {
    let browsers = browserware_detect::detect_browsers();
    let default = browserware_detect::detect_default_browser();
    
    match cli.format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&browsers)?);
        }
        OutputFormat::Table => {
            println!("{:<15} {:<25} {:<12} {:<10} {}", 
                "ID", "NAME", "FAMILY", "CHANNEL", "DEFAULT");
            println!("{}", "-".repeat(70));
            
            for browser in &browsers {
                let is_default = default.as_ref()
                    .map(|d| d.id == browser.id)
                    .unwrap_or(false);
                
                println!("{:<15} {:<25} {:<12} {:<10} {}", 
                    browser.id,
                    browser.name,
                    browser.variant.family(),
                    browser.variant,
                    if is_default { "*" } else { "" }
                );
            }
        }
        OutputFormat::Plain => {
            for browser in &browsers {
                println!("{}", browser.id);
            }
        }
    }
}
```

---

## Testing Strategy

### Unit Tests

Each platform module should have comprehensive tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn parse_executable_from_command_quoted() {
        let cmd = r#""C:\Program Files\Browser\browser.exe" --flag"#;
        let exe = parse_executable_from_command(cmd).unwrap();
        assert_eq!(exe, PathBuf::from(r"C:\Program Files\Browser\browser.exe"));
    }
    
    #[test]
    fn parse_executable_from_command_unquoted() {
        let cmd = r"C:\Browser\browser.exe --flag";
        let exe = parse_executable_from_command(cmd).unwrap();
        assert_eq!(exe, PathBuf::from(r"C:\Browser\browser.exe"));
    }
}
```

### Integration Tests

```rust
// tests/integration.rs

#[test]
fn detect_browsers_returns_vec() {
    let browsers = browserware_detect::detect_browsers();
    // Should at least not panic
    // On CI, may be empty if no browsers installed
    assert!(browsers.len() >= 0);
}

#[test]
fn detected_browsers_have_valid_ids() {
    for browser in browserware_detect::detect_browsers() {
        assert!(!browser.id.0.is_empty());
        assert!(!browser.name.is_empty());
        assert!(browser.executable.exists() || cfg!(test));
    }
}

#[test]
fn detect_browser_by_id_consistency() {
    let browsers = browserware_detect::detect_browsers();
    
    for browser in &browsers {
        let found = browserware_detect::detect_browser(&browser.id.0);
        assert!(found.is_some(), "Browser {} should be findable by ID", browser.id);
    }
}
```

### CLI Integration Tests

```rust
// crates/browserware-cli/tests/cli.rs

#[test]
fn browsers_command_json_output() {
    brw()
        .args(["browsers", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("["));
}

#[test]
fn browsers_command_table_output() {
    brw()
        .args(["browsers", "--format", "table"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ID"));
}
```

---

## Dependencies Update

```toml
# crates/browserware-detect/Cargo.toml
[package]
name = "browserware-detect"
description = "Cross-platform browser detection"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true

[dependencies]
browserware-types = { workspace = true }
tracing = { workspace = true }

[target.'cfg(target_os = "macos")'.dependencies]
core-foundation = "0.10"
# For plist parsing
plist = "1.7"

[target.'cfg(target_os = "windows")'.dependencies]
windows = { version = "0.58", features = [
    "Win32_System_Registry",
    "Win32_Storage_FileSystem",
] }

[target.'cfg(target_os = "linux")'.dependencies]
xdg = "3"
home = "0.5"

[dev-dependencies]
tempfile = { workspace = true }

[lints]
workspace = true
```

---
