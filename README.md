# Browserware

A modular ecosystem for intelligent browser routing across macOS, Windows, and Linux.

## Installation

```bash
cargo install browserware-cli
```

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

## License

MIT OR Apache-2.0
