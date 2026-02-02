# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `browserware-detect` crate with macOS browser detection via Launch Services
  - Discovery-first approach: enumerates all HTTPS URL handlers, then enriches with known metadata
  - `detect_browsers()`, `detect_browser()`, `detect_default_browser()`, `detect_browsers_by_family()` public API
  - Known browser registry (35 entries across Chromium, Firefox, and WebKit families)
  - Info.plist parsing for version and display name extraction
  - Unknown browser derivation from bundle ID for browsers not in registry
  - Nested app filtering to prevent duplicate helper app entries
  - Windows and Linux platform stubs
- `brw browsers` CLI command
  - `--format table|json|plain` output formats
  - `--family` filter (chromium, firefox, webkit, other)
  - Default browser indicator (`*` in table, `(default)` in plain, JSON field)

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
