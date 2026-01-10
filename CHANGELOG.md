# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Initial workspace structure with 7 crates
- `browserware-types` crate with core types (`Browser`, `BrowserFamily`, `BrowserVariant`)
- `browserware-cli` scaffold with `brw` binary
- CI/CD pipeline (test, clippy, fmt, deny, docs)
- Release workflow for multi-platform binaries
- Security audit workflow
- Dependabot configuration

### Infrastructure

- Rust 1.88+ (Edition 2024)
- Workspace-level lints (clippy pedantic + nursery)
- cargo-deny for dependency security and license compliance

[Unreleased]: https://github.com/browserware/browserware/commits/main
