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
These files help AI tools understand modern Rust features and project conventions.

## License

MIT OR Apache-2.0
