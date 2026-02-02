# Milestone 1: Browser Detection

**Goal**: Fully functional `browserware-detect` crate with cross-platform browser discovery
**Success Metric**: `brw browsers` returns accurate browser list on macOS, Windows, and Linux

---

## Current State

### Completed

- **browserware-types**: Core types (`Browser`, `BrowserVariant`, `BrowserFamily`, `BrowserId`) with serde support, builder pattern, and comprehensive tests
- **browserware-detect registry**: 30 known browser entries covering Chromium (19), Firefox (8), and WebKit (3) families with all platform identifiers
- **macOS detection**: Full implementation using Launch Services discovery-first approach (see [ADR: Discovery-First Detection](../decisions.md#discovery-first-detection-strategy))
- **Detection public API**: `detect_browsers()`, `detect_browser()`, `detect_default_browser()`, `detect_browsers_by_family()` with tracing instrumentation
- **CLI browsers command**: `brw browsers` with `--format table|json|plain`, `--family` filter, default browser `*` indicator
- **Rustdoc**: All public items documented with examples

### In Progress / Remaining

- Windows detection (registry enumeration)
- Linux detection (XDG desktop file scanning)
- Integration tests for browserware-detect
- CLI integration tests
- Usage examples in crate-level docs
- CHANGELOG update for M1
- Performance testing (detection target: <100ms)
- Cross-platform CI verification

---

## Architecture

### Module Structure

```
crates/browserware-detect/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Public API, re-exports
│   ├── registry.rs         # Known browser registry (30 entries)
│   ├── platform/
│   │   ├── mod.rs          # Platform detection, cfg routing
│   │   ├── macos.rs        # macOS: Launch Services (implemented)
│   │   ├── windows.rs      # Windows: Registry (stub)
│   │   └── linux.rs        # Linux: XDG desktop files (stub)
└── tests/
    └── (integration tests TBD)
```

### Detection Strategy: Discovery-First

The detection process enumerates all registered URL handlers from the OS, then enriches with metadata from the known browser registry. This ensures unknown browsers are never missed.

```rust
// 1. Enumerate ALL browsers from OS (e.g., LSCopyAllHandlersForURLScheme on macOS)
let all_handlers = platform::enumerate_url_handlers("https");

// 2. For each, enrich with known metadata or derive
for bundle_id in all_handlers {
    match registry::find_by_bundle_id(&bundle_id) {
        Some(meta) => Browser::from_meta(meta, ...),  // Known browser
        None => Browser::unknown(bundle_id, ...),     // Unknown browser
    }
}
```

Unknown browsers receive:
- ID: derived from platform identifier (e.g., bundle ID)
- Name: from app metadata (`CFBundleName`, desktop `Name=`)
- Variant: `BrowserVariant::Single(BrowserFamily::Other)`

### Platform APIs

| Platform | Enumerate Browsers | Default Browser | App Metadata |
|----------|-------------------|-----------------|--------------|
| macOS | `LSCopyAllHandlersForURLScheme("https")` | `LSCopyDefaultHandlerForURLScheme("https")` | Info.plist parsing |
| Windows | `HKLM\SOFTWARE\Clients\StartMenuInternet` subkeys | `HKCU\...\UrlAssociations\http\UserChoice\ProgId` | PE version info |
| Linux | XDG `.desktop` files with `MimeType=x-scheme-handler/http` | `xdg-settings get default-web-browser` | Desktop file fields |

### Platform Dependencies

| Platform | Crate | Version | Purpose |
|----------|-------|---------|---------|
| macOS | `core-foundation` | 0.10.x | Core Foundation types and bindings |
| macOS | `plist` | 1.x | Parse Info.plist files |
| Windows | `windows-registry` | 0.6.x | Registry access (part of windows-rs) |
| Linux | `xdg` | 3.x | XDG Base Directory spec |
| Linux | `home` | 0.5.x | Home directory detection |

---

## Task Breakdown

### Task 1: Foundation & Registry [done]

- [x] `BrowserMeta` struct and `KNOWN_BROWSERS` array (30 entries)
- [x] Lookup functions: `find_by_id()`, `find_by_bundle_id()`, `find_by_registry_key()`, `find_by_desktop_id()`
- [x] Platform cfg routing in `platform/mod.rs`
- [x] Public API signatures in `lib.rs`
- [x] Platform-specific dependencies in `Cargo.toml`
- [x] Tracing instrumentation

### Task 2: macOS Implementation [done]

- [x] `LSCopyAllHandlersForURLScheme()` - enumerate all HTTPS handlers
- [x] `LSCopyApplicationURLsForBundleIdentifier()` - resolve app path from bundle ID
- [x] Info.plist parsing for version (`CFBundleShortVersionString`) and name
- [x] `detect_browsers()` with discovery-first approach
- [x] `detect_default_browser()` via `LSCopyDefaultHandlerForURLScheme()`
- [x] Nested app filtering (prevents duplicate helper apps)
- [x] Unknown browser derivation from bundle ID
- [x] Unit tests for macOS

### Task 3: Windows Implementation [done]

- [x] Enumerate `HKLM\SOFTWARE\Clients\StartMenuInternet` subkeys
- [x] Read `shell\open\command` for executable paths
- [x] Parse executable path from command strings
- [x] Version extraction from PE file info
- [x] `detect_browsers()` implementation
- [x] `detect_default_browser()` via `HKCU\...\UrlAssociations\http\UserChoice\ProgId`
- [x] Unit tests for Windows
- [ ] Manual testing on Windows (VM or CI)

### Task 4: Linux Implementation [done]

- [x] Scan XDG directories for `.desktop` files
- [x] Parse `.desktop` file format (Name, Exec, MimeType)
- [x] Filter for `x-scheme-handler/http` MimeType
- [x] Handle Flatpak and Snap application paths
- [x] `parse_exec_to_path()` - extract executable from Exec= field
- [x] `detect_browsers()` implementation
- [x] `detect_default_browser()` via `xdg-settings get default-web-browser`
- [x] Unit tests for Linux

### Task 5: CLI Integration [partial]

- [x] `brw browsers` command with `browserware-detect` integration
- [x] Table/JSON/plain output formatting
- [x] Default browser indicator (`*` prefix)
- [x] `--family` filter flag
- [ ] CLI integration tests
- [ ] Cross-platform CI verification

### Task 6: Documentation & Release Prep [partial]

- [x] Rustdoc for all public items
- [ ] Usage examples in crate docs
- [ ] Update CHANGELOG for M1
- [ ] Performance testing (detection target: <100ms)

---

## Success Criteria

| # | Criterion | Status |
|---|-----------|--------|
| 1 | `brw browsers` returns accurate list on macOS, Windows, Linux | macOS done, Windows/Linux pending |
| 2 | `brw browsers --format json` produces valid JSON | Done |
| 3 | Default browser correctly identified per platform | macOS done, Windows/Linux pending |
| 4 | All detected browsers have valid executable paths | Done |
| 5 | Versions extracted where possible | Done |
| 6 | CI passes on all three platforms | Needs verification |
| 7 | `cargo doc` builds without warnings | Done |
| 8 | Minimal, well-documented unsafe FFI for Launch Services | Done |

---

## Risk Mitigation

| Risk | Mitigation | Status |
|------|------------|--------|
| Core Foundation API changes | Use stable, documented APIs; test on multiple macOS versions | Mitigated (using stable LS APIs) |
| Windows registry structure varies | Test on Windows 10/11; handle missing keys gracefully | Pending |
| Flatpak/Snap paths differ | Scan all known locations; log warnings for unrecognized | Pending |
| Browser not in registry | Discovery-first: return unknown browser with derived metadata | Implemented |
| Version extraction fails | Version is `Option<String>`, gracefully handle None | Implemented |

---

## Files

### Created (this milestone)

- `crates/browserware-detect/src/registry.rs` - done
- `crates/browserware-detect/src/platform/mod.rs` - done
- `crates/browserware-detect/src/platform/macos.rs` - done
- `crates/browserware-detect/src/platform/windows.rs` - stub
- `crates/browserware-detect/src/platform/linux.rs` - stub

### Modified (this milestone)

- `crates/browserware-detect/Cargo.toml` - done
- `crates/browserware-detect/src/lib.rs` - done
- `crates/browserware-cli/Cargo.toml` - done
- `crates/browserware-cli/src/main.rs` - done

### Remaining

- `crates/browserware-detect/tests/integration.rs` - not started
- `crates/browserware-cli/tests/cli.rs` - needs browser command tests
- `CHANGELOG.md` - needs M1 entries
