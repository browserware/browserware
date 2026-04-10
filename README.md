# Browserware

A modular ecosystem for intelligent browser routing across macOS, Windows, and Linux.

## Installation

```bash
cargo install browserware-cli
```

## Development Tooling

This repository pins its local development and validation tooling with `mise`.

```bash
mise install
mise exec cargo:just@1.49.0 -- just validate
```

Use the `just ci-*` recipes or `mise exec ...` when running checks locally so your machine exercises the same Rust and cargo-tool versions expected in CI.

## Usage

```bash
# List detected browsers
brw browsers

# List launchable browser contexts
brw contexts

# Open URL with routing
brw open https://github.com
```

## Crates

- `browserware-types` - Shared types
- `browserware-detect` - Browser discovery
- `browserware-profiles` - Profile management
- `browserware-launch` - Browser launching
- `browserware-rules` - Routing rules
- `browserware-system` - OS integration
- `browserware-cli` - CLI (`brw`)

## AI Assistant Context

This repo includes context files for AI coding assistants in `.context/`.
Start with [`AGENTS.md`](AGENTS.md), then load [`.context/RUST_MODERN.md`](.context/RUST_MODERN.md) for the Rust 1.88+ coding guidance shared with Claude-compatible agents and other coding assistants.
When validation or tool version parity matters, use `mise exec` or the `just ci-*` recipes instead of invoking ambient `cargo` tooling directly.

## License

MIT OR Apache-2.0
