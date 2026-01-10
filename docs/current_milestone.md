# Milestone 1: Browser Detection Implementation Plan

**Duration**: 6 weeks (Week 3-8 of roadmap)  
**Goal**: Fully functional `browserware-detect` crate with cross-platform browser discovery  
**Success Metric**: `brw browsers` returns accurate browser list on macOS, Windows, and Linux

---

## Current State (Post-M0)

### What We Have

```rust
// browserware-types already provides:
pub struct Browser {
    pub id: BrowserId,
    pub name: String,
    pub variant: BrowserVariant,  // Replaces separate family + channel
    pub version: Option<String>,
    pub executable: PathBuf,
    pub bundle_id: Option<String>,  // macOS
}

pub enum BrowserFamily { Chromium, Firefox, WebKit, Other }

pub enum BrowserVariant {
    Chromium(ChromiumChannel),  // Stable, Beta, Dev, Canary
    Firefox(FirefoxChannel),    // Stable, Beta, Dev, Nightly, Esr
    WebKit(WebKitChannel),      // Stable, TechnologyPreview
    Single(BrowserFamily),      // Arc, GNOME Web, etc.
}
```

### What We're Building

```rust
// browserware-detect will provide:
pub fn detect_browsers() -> Vec<Browser>;
pub fn detect_browser(id: &str) -> Option<Browser>;
pub fn detect_default_browser() -> Option<Browser>;
pub fn detect_browsers_by_family(family: BrowserFamily) -> Vec<Browser>;
```

---

## Architecture

### Module Structure

```
crates/browserware-detect/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Public API, re-exports
│   ├── registry.rs         # Known browser registry (metadata)
│   ├── platform/
│   │   ├── mod.rs          # Platform detection, cfg routing
│   │   ├── macos.rs        # macOS: Launch Services, Spotlight
│   │   ├── windows.rs      # Windows: Registry
│   │   └── linux.rs        # Linux: XDG desktop files
│   └── version.rs          # Version extraction utilities
└── tests/
    ├── integration.rs      # Cross-platform integration tests
    └── fixtures/           # Test data (mock registries, etc.)
```


## Task Breakdown by Week

### Week 1: Foundation & Registry

- [ ] Create `src/registry.rs` with `BrowserMeta` struct and `KNOWN_BROWSERS`
- [ ] Create `src/platform/mod.rs` with cfg routing
- [ ] Create `src/lib.rs` with public API signatures
- [ ] Add platform-specific dependencies to `Cargo.toml`
- [ ] Add `tracing` instrumentation points

### Week 2: macOS Implementation

- [ ] Implement `detect_by_bundle_id()` using Core Foundation
- [ ] Implement `extract_version_from_plist()`
- [ ] Implement `detect_browsers()`
- [ ] Implement `detect_default_browser()`
- [ ] Unit tests for macOS
- [ ] Manual testing on macOS

### Week 3: Windows Implementation

- [ ] Implement registry enumeration
- [ ] Implement `parse_executable_from_command()`
- [ ] Implement version extraction from PE
- [ ] Implement `detect_browsers()`
- [ ] Implement `detect_default_browser()`
- [ ] Unit tests for Windows
- [ ] Manual testing on Windows (VM or CI)

### Week 4: Linux Implementation

- [ ] Implement XDG directory scanning
- [ ] Implement `.desktop` file parsing
- [ ] Implement `parse_exec_to_path()`
- [ ] Handle Flatpak and Snap paths
- [ ] Implement `detect_browsers()`
- [ ] Implement `detect_default_browser()`
- [ ] Unit tests for Linux

### Week 5: CLI Integration & Polish

- [ ] Update `browserware-cli` to use `browserware-detect`
- [ ] Implement table/json/plain output formatting
- [ ] Add default browser indicator
- [ ] CLI integration tests
- [ ] Cross-platform CI verification

### Week 6: Documentation & Release Prep

- [ ] Complete rustdoc for all public items
- [ ] Add usage examples to crate docs
- [ ] Update README with examples
- [ ] Update CHANGELOG
- [ ] Performance testing (detection should be <100ms)
- [ ] Tag M1 completion

---

## Success Criteria

1. ✅ `brw browsers` returns accurate list on macOS, Windows, Linux
2. ✅ `brw browsers --format json` produces valid JSON
3. ✅ Default browser is correctly identified
4. ✅ All detected browsers have valid executable paths
5. ✅ Versions are extracted where possible
6. ✅ CI passes on all three platforms
7. ✅ `cargo doc` builds without warnings
8. ✅ No `unsafe` code (or minimal, well-documented FFI)

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Core Foundation API changes | Use stable, documented APIs; test on multiple macOS versions |
| Windows registry structure varies | Test on Windows 10/11; handle missing keys gracefully |
| Flatpak/Snap paths differ | Scan all known locations; log warnings for unrecognized |
| Browser not in registry | Return unknown browser with generic family |
| Version extraction fails | Version is `Option<String>`, gracefully handle None |

---

## Files to Create/Modify

**New Files**:
- `crates/browserware-detect/src/registry.rs`
- `crates/browserware-detect/src/platform/mod.rs`
- `crates/browserware-detect/src/platform/macos.rs`
- `crates/browserware-detect/src/platform/windows.rs`
- `crates/browserware-detect/src/platform/linux.rs`
- `crates/browserware-detect/tests/integration.rs`

**Modified Files**:
- `crates/browserware-detect/Cargo.toml` (add dependencies)
- `crates/browserware-detect/src/lib.rs` (implement API)
- `crates/browserware-cli/Cargo.toml` (add browserware-detect)
- `crates/browserware-cli/src/main.rs` (implement browsers command)
- `crates/browserware-cli/tests/cli.rs` (add browsers tests)
- `Cargo.toml` (add platform dependencies to workspace)
- `CHANGELOG.md` (update for M1)