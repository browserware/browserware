# Browserware Architecture

For comprehensive architecture documentation, see [AGENTS.md](../AGENTS.md).

## Crate Dependency Graph

```
┌─────────────────────────────────────────────────────────────────┐
│                    browserware-cli (brw)                        │
│                    browserware-gui (closed)                     │
└─────────────────────────────────────────────────────────────────┘
        │           │           │           │           │
        ▼           ▼           ▼           ▼           ▼
┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐
│  detect  │ │ profiles │ │  launch  │ │  rules   │ │  system  │
└──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘
                                │
                                ▼
                         ┌──────────┐
                         │  types   │
                         └──────────┘
```

## Crate Responsibilities

| Crate | Question It Answers | Status |
|-------|---------------------|--------|
| `browserware-types` | Shared types (`Browser`, `Profile`, etc.) | In progress |
| `browserware-detect` | "What browsers are installed?" | Scaffold |
| `browserware-profiles` | "What profiles exist for a browser?" | Scaffold |
| `browserware-launch` | "Run this browser with these options" | Scaffold |
| `browserware-rules` | "Which browser/profile for this URL?" | Scaffold |
| `browserware-system` | "OS integration (default browser, etc.)" | Scaffold |
| `browserware-cli` | User-facing CLI (`brw` binary) | Scaffold |

## Data Flow

Example: `brw open https://example.com`

```
1. CLI parses URL
2. rules::discover_rules() → find config file
3. rules::evaluate(rules, context) → BrowserTarget
4. detect::detect_browser("chrome") → Browser
5. profiles::find_profile(family, "Work") → Profile
6. launch::launch(executable, family, urls, options) → Result
```

## Design Principles

1. **Modularity** - Each crate has exactly one responsibility
2. **Independence** - Libraries are useful standalone (`cargo add browserware-detect`)
3. **Explicit inputs** - Functions receive explicit parameters, no global state
4. **Graceful errors** - Return `Result<T, Error>`, never panic
5. **Platform parity** - Features work identically across macOS, Windows, Linux

## Licensing

- **Open source** (MIT OR Apache-2.0): All library crates + CLI
- **Closed source**: GUI application (separate private repository)
