# Technical Decisions

Lightweight Architecture Decision Records (ADRs) for tooling and configuration choices.

## CI/CD

### MSRV Testing

**Decision**: Test MSRV via CI matrix, not `cargo-msrv` tool.

**Reason**: `taiki-e/install-action@cargo-msrv` doesn't exist. The cargo-msrv tool requires separate installation and is slower than just running the build with the MSRV toolchain.

**Implementation**:
```yaml
matrix:
  rust: [stable, "1.88"]  # 1.88 is MSRV
```

### GitHub Actions Versions

As of January 2026:
- `actions/checkout@v6`
- `actions/cache@v5`
- `dtolnay/rust-toolchain@master` (or `@stable`)
- `taiki-e/install-action@<tool>` for cargo-deny, cargo-audit, cross

## Rust Tooling

### assert_cmd

**Decision**: Use `cargo_bin_cmd!` macro, not `Command::cargo_bin()`.

**Reason**: `Command::cargo_bin()` is deprecated as of assert_cmd 2.x due to incompatibility with custom cargo build directories.

**Implementation**:
```rust
use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin_cmd;

fn brw() -> Command {
    cargo_bin_cmd!("brw")
}
```

### cargo-deny v2

**Decision**: Use cargo-deny v2 configuration format.

**Changes from v1**:
- `[licenses]` section requires `version = 2`
- `unlicensed` key removed (deny is now default)
- `unmaintained`, `yanked`, `notice` take graph filter values (`"all"`, `"workspace"`, `"transitive"`, `"none"`), not lint levels (`"warn"`, `"deny"`)
- `wildcards = "warn"` for workspace path dependencies (they appear as wildcards)

**Implementation**:
```toml
[licenses]
version = 2
allow = ["MIT", "Apache-2.0", ...]

[advisories]
unmaintained = "workspace"  # not "warn"

[bans]
wildcards = "warn"  # path deps trigger this
```

## Version Control

### Cargo.lock

**Decision**: Commit `Cargo.lock` for all crates.

**Reason**: Cargo team guidance changed in August 2023. Benefits:
- Reproducible builds
- Easier bisecting for debugging
- Reliable CI (no surprise breakage from new dependency releases)
- MSRV testing with consistent dependency versions

**Reference**: [Cargo Blog - Change in Guidance on Committing Lockfiles](https://blog.rust-lang.org/2023/08/29/committing-lockfiles/)

### Release workflow

**Decision**: Use `--locked` flag in release builds.

**Reason**: Ensures the committed Cargo.lock is respected, preventing builds with different dependency versions.

```yaml
cargo build --release --locked --target ${{ matrix.target }}
```

## Development Workflow

### Task Runner

**Decision**: Use `just` (justfile) for development tasks.

**Reason**:
- Becoming Rust ecosystem standard
- Better syntax than Makefile
- Cross-platform
- Simple dependency-free recipes

**Setup**: `cargo install just`, then `just setup`

## Browser Detection

### Discovery-First Detection Strategy

**Decision**: Enumerate all registered URL handlers from the OS, then enrich with metadata from a known browser registry.

**Alternatives Considered**:
1. **Registry-first**: Check only browsers in our known registry - misses new/unknown browsers
2. **Discovery-first** (chosen): Enumerate all, enrich known ones

**Reason**: Users may have browsers we don't know about (forks, regional browsers, new releases). A registry-only approach would silently miss them. Discovery-first ensures we never miss a browser, while still providing rich metadata for known ones.

**Implementation**:
```rust
// 1. Enumerate ALL browsers from OS
let all_handlers = platform::enumerate_url_handlers("https");

// 2. For each, enrich with known metadata or derive
for bundle_id in all_handlers {
    match registry::find_by_bundle_id(&bundle_id) {
        Some(meta) => Browser::from_meta(meta, ...),  // Known browser
        None => Browser::unknown(bundle_id, ...),     // Unknown browser
    }
}
```

**Unknown browsers get**:
- ID: bundle ID used directly (e.g., `com.example.browser`)
- Name: from app metadata (`CFBundleName`, desktop `Name=`)
- Variant: `BrowserVariant::Single(BrowserFamily::Other)`

### Registry Data Quality

**Decision**: All browser entries in `KNOWN_BROWSERS` must have verifiable platform identifiers.

**Reason**: Bundle IDs, registry keys, and desktop IDs must be sourced from actual installations or official documentation. Unverified or placeholder data can cause incorrect browser matching.

### Platform-Specific Detection APIs

**macOS**:
- `LSCopyAllHandlersForURLScheme("https")` - enumerate all browsers
- `LSCopyDefaultHandlerForURLScheme("https")` - get default browser
- `LSCopyApplicationURLsForBundleIdentifier()` - get app path from bundle ID
- `CFBundleCopyInfoDictionaryForURL()` - read Info.plist for version

**Windows**:
- `HKLM\SOFTWARE\Clients\StartMenuInternet` - enumerate all browsers
- `HKCU\...\UrlAssociations\http\UserChoice\ProgId` - get default browser

**Linux**:
- Scan XDG directories for `.desktop` files with `MimeType=x-scheme-handler/http`
- `xdg-settings get default-web-browser` - get default browser

### Platform Dependencies

**Decision**: Use lightweight, focused crates for platform APIs.

| Platform | Crate | Version | Purpose |
|----------|-------|---------|---------|
| macOS | `core-foundation` | 0.10.x | Core Foundation types and bindings |
| macOS | `plist` | 1.x | Parse Info.plist files |
| Windows | `windows-registry` | 0.6.x | Registry access (part of windows-rs) |
| Linux | `xdg` | 3.x | XDG Base Directory spec |
| Linux | `home` | 0.5.x | Home directory detection |

**Reason**: These crates are well-maintained, focused on their specific tasks, and avoid pulling in large framework dependencies. For home directory detection we use the `home` crate (maintained by rust-lang) instead of the deprecated `dirs::home_dir()` function.
