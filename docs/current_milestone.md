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

### Week 1: Foundation & Registry ✅

- [x] Create `src/registry.rs` with `BrowserMeta` struct and `KNOWN_BROWSERS`
- [x] Create `src/platform/mod.rs` with cfg routing
- [x] Create `src/lib.rs` with public API signatures
- [x] Add platform-specific dependencies to `Cargo.toml`
- [x] Add `tracing` instrumentation points

### Week 2: macOS Implementation ✅

- [x] Implement `LSCopyAllHandlersForURLScheme()` FFI bindings
- [x] Implement `extract_version_from_plist()`
- [x] Implement `detect_browsers()` with discovery-first approach
- [x] Implement `detect_default_browser()`
- [x] Filter nested apps (helper apps inside Contents/Support/)
- [x] Unit tests for macOS
- [x] Manual testing on macOS

### Week 3: Windows Implementation

- [ ] Implement registry enumeration
- [ ] Implement `parse_executable_from_command()`
- [ ] Implement version extraction from PE
- [ ] Implement `detect_browsers()`
- [ ] Implement `detect_default_browser()`
- [ ] Unit tests for Windows
- [ ] Manual testing on Windows (VM or CI)

### Week 4: Linux Implementation ✅

- [x] Implement XDG directory scanning
- [x] Implement `.desktop` file parsing
- [x] Implement `parse_exec_to_path()`
- [x] Handle Flatpak and Snap paths
- [x] Implement `detect_browsers()`
- [x] Implement `detect_default_browser()`
- [x] Unit tests for Linux

### Week 5: CLI Integration & Polish (partial) ✅

- [x] Update `browserware-cli` to use `browserware-detect`
- [x] Implement table/json/plain output formatting
- [x] Add default browser indicator (`*` prefix)
- [ ] CLI integration tests
- [ ] Cross-platform CI verification

### Week 6: Documentation & Release Prep

- [x] Complete rustdoc for all public items
- [ ] Add usage examples to crate docs
- [ ] Update README with examples
- [ ] Update CHANGELOG
- [ ] Performance testing (detection should be <100ms)
- [ ] Tag M1 completion

---

## Success Criteria

1. 🟡 `brw browsers` returns accurate list on macOS, Windows, Linux (macOS ✅, Linux ✅, Windows pending)
2. ✅ `brw browsers --format json` produces valid JSON
3. 🟡 Default browser is correctly identified (macOS ✅, Linux ✅, Windows pending)
4. ✅ All detected browsers have valid executable paths
5. ✅ Versions are extracted where possible
6. 🟡 CI passes on all three platforms (needs verification)
7. ✅ `cargo doc` builds without warnings
8. ✅ Minimal, well-documented unsafe FFI for Launch Services

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
- ✅ `crates/browserware-detect/src/registry.rs`
- ✅ `crates/browserware-detect/src/platform/mod.rs`
- ✅ `crates/browserware-detect/src/platform/macos.rs`
- ✅ `crates/browserware-detect/src/platform/windows.rs` (stub)
- ✅ `crates/browserware-detect/src/platform/linux.rs`
- [ ] `crates/browserware-detect/tests/integration.rs`

**Modified Files**:
- ✅ `crates/browserware-detect/Cargo.toml` (add dependencies)
- ✅ `crates/browserware-detect/src/lib.rs` (implement API)
- ✅ `crates/browserware-cli/Cargo.toml` (add browserware-detect)
- ✅ `crates/browserware-cli/src/main.rs` (implement browsers command)
- [ ] `crates/browserware-cli/tests/cli.rs` (add browsers tests)
- [ ] `Cargo.toml` (add platform dependencies to workspace)
- [ ] `CHANGELOG.md` (update for M1)