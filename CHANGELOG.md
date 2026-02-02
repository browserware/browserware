# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Milestone 1: Cross-Platform Browser Detection**
  - `browserware-detect` crate with full macOS, Windows, and Linux support
  - Discovery-first detection strategy: enumerates all URL handlers from OS, then enriches with metadata
  - Public API: `detect_browsers()`, `detect_browser()`, `detect_default_browser()`, `detect_browsers_by_family()`
  - Known browser registry with 30 entries (19 Chromium, 8 Firefox, 3 WebKit variants)
  - Full Rustdoc documentation with usage examples

- **macOS Detection** (via Launch Services)
  - `LSCopyAllHandlersForURLScheme()` for browser enumeration
  - Info.plist parsing for version and display name extraction
  - `LSCopyDefaultHandlerForURLScheme()` for default browser
  - Nested app filtering to prevent duplicate helper app entries
  - Unknown browser derivation from bundle IDs

- **Windows Detection** (via Registry)
  - Enumeration of `HKLM\SOFTWARE\Clients\StartMenuInternet` subkeys
  - Command string parsing to extract executable paths
  - PE file version info extraction
  - Default browser detection via `HKCU\...\UrlAssociations\http\UserChoice\ProgId`

- **Linux Detection** (via XDG Desktop Files)
  - Scanning of XDG data directories for `.desktop` files
  - Desktop file parsing (Name, Exec, MimeType fields)
  - Filtering for `x-scheme-handler/http` MIME type handlers
  - Support for Flatpak and Snap application paths
  - Default browser detection via `xdg-settings`

- **CLI Enhancement**
  - `brw browsers` command with cross-platform support
  - `--format table|json|plain` output formats
  - `--family` filter (chromium, firefox, webkit, other)
  - Default browser indicator (`*` prefix in table, `(default)` suffix in plain, boolean field in JSON)
  - Comprehensive CLI integration tests

## [0.1.0] - 2026-01-10

### Added

- Initial workspace structure with 7 crates
- `browserware-types` crate with core types (`Browser`, `BrowserFamily`, `BrowserVariant`)
- `browserware-cli` scaffold with `brw` binary
- CI/CD pipeline (test, clippy, fmt, deny, docs)
- Release workflow for multi-platform binaries
- Security audit workflow
- Dependabot configuration
- Development task runner (`justfile`)
- Contributing guidelines and architecture documentation

### Infrastructure

- Rust 1.88+ (Edition 2024)
- Workspace-level lints (clippy pedantic + nursery)
- cargo-deny for dependency security and license compliance

[Unreleased]: https://github.com/browserware/browserware/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/browserware/browserware/releases/tag/v0.1.0
