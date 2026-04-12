# Browserware

A modular ecosystem for intelligent browser routing across macOS, Windows, and Linux.

## Installation

> **Note:** Not yet published to crates.io. Install directly from the repository:

```bash
cargo install --git https://github.com/browserware/browserware browserware-cli
```

## Development Tooling

This repository pins its local development and validation tooling with `mise`.

```bash
mise install
mise exec cargo:just@1.49.0 -- just validate
```

Use the `just ci-*` recipes or `mise exec ...` when running checks locally so your machine exercises the same Rust and cargo-tool versions expected in CI.

## Getting started

```bash
# List detected browsers
brw browsers

# List launchable browser contexts (copy a selector from plain output)
brw contexts

# Open a URL in a specific context (copy a selector from `brw contexts`)
brw open --context chrome:Default https://example.com

# Print the launch command without running the browser
brw open --dry-run --context chrome:Default https://example.com
```

`brw open` without `--context` exits with an error and a hint. That avoids accidental loops through the default browser until config-driven routing is wired up.

## Usage

Same commands as in **Getting started**; use `brw --help` and `brw open --help` for flags.

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
