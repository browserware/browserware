# Contributing to Browserware

Thank you for your interest in contributing! This guide covers development setup and workflow for the main browserware repository.

For organization-wide policies, see the [.github repository](https://github.com/browserware/.github).

## Prerequisites

- **[mise](https://mise.jdx.dev/)** for pinned local tool versions
- **Git** with DCO sign-off configured

Install the pinned toolchain and validation tools:

```bash
mise install
```

## Development Setup

```bash
git clone https://github.com/browserware/browserware.git
cd browserware
mise exec cargo:just@1.49.0 -- just setup    # Creates AI assistant symlinks
mise exec cargo:just@1.49.0 -- just build
mise exec cargo:just@1.49.0 -- just test
```

> `mise.toml` pins the Rust toolchain plus `just`, `cargo-deny`, and `cargo-audit`. Run all repo validation through `just` so your local checks match CI as closely as possible.

### Useful Commands

```bash
mise exec cargo:just@1.49.0 -- just          # List all available tasks
mise exec cargo:just@1.49.0 -- just validate # Run the full local pre-PR pipeline
mise exec cargo:just@1.49.0 -- just validate-targets  # Optional Linux/Windows compile smoke checks
mise exec cargo:just@1.49.0 -- just fmt      # Format code
mise exec cargo:just@1.49.0 -- just test     # Run tests
mise exec cargo:just@1.49.0 -- just docs     # Build and open documentation
mise exec cargo:just@1.49.0 -- just clean    # Clean build artifacts + symlinks
```

Or directly with `mise exec` when you need a specific toolchain:

```bash
mise exec rust@1.94.1 -- cargo test -p browserware-types --locked
mise exec rust@1.88.0 -- cargo test --workspace --all-targets --locked
mise exec rust@1.94.1 -- cargo run -p browserware-cli -- --help
```

## Code Quality

This project enforces strict quality standards:

| Check | Command | CI Job |
|-------|---------|--------|
| Formatting | `just ci-rustfmt` | `fmt` |
| Linting | `just ci-clippy` | `clippy` |
| Documentation | `just ci-docs` | `docs` |
| Dependencies | `just ci-deny` | `deny` |
| Security Audit | `just ci-audit` | `audit` |
| Tests (stable) | `just ci-test-stable` | `test (stable)` |
| Tests (MSRV 1.88.0) | `just ci-test-msrv` | `test (1.88)` |

Run `just validate` before opening a PR. On macOS or Linux, `just validate-targets` adds compile-only smoke checks for the Linux and Windows targets that CI also exercises.

## Making Changes

### Branch Naming

- `feat/description` - New features
- `fix/description` - Bug fixes
- `docs/description` - Documentation
- `refactor/description` - Code refactoring
- `test/description` - Test additions

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): description
```

**Types**: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`

**Scopes**: `types`, `detect`, `profiles`, `launch`, `rules`, `system`, `cli`

**Examples**:

```
feat(detect): add Arc browser detection on macOS
fix(launch): handle spaces in profile names
docs(readme): add installation instructions
```

### Developer Certificate of Origin (DCO)

All commits must be signed off:

```bash
git commit -s -m "feat(detect): add browser support"
```

This adds `Signed-off-by: Your Name <email@example.com>` to the commit.

See [DCO.md](https://github.com/browserware/.github/blob/main/DCO.md) for details.

## Pull Request Process

1. Fork the repository
2. Create a feature branch from `main`
3. Make changes with signed commits
4. Ensure all CI checks pass
5. Submit a pull request

PRs require:
- All CI checks passing
- DCO sign-off on all commits
- Review approval

## Workspace Structure

```
crates/
  browserware-types/    # Shared types (Browser, Profile, etc.)
  browserware-detect/   # Browser detection
  browserware-profiles/ # Profile discovery
  browserware-launch/   # Browser launching
  browserware-rules/    # URL routing rules
  browserware-system/   # OS integration
  browserware-cli/      # CLI binary (brw)
```

Each crate has a single responsibility. See [AGENTS.md](https://github.com/browserware/.github/blob/main/AGENTS.md) for architecture details.

## Testing

### Unit Tests

```bash
just test
```

### Integration Tests

```bash
mise exec rust@1.94.1 -- cargo test -p browserware-cli --test cli --locked
```

### Manual Testing

```bash
mise exec rust@1.94.1 -- cargo run -p browserware-cli -- --help
mise exec rust@1.94.1 -- cargo run -p browserware-cli -- browsers
mise exec rust@1.94.1 -- cargo run -p browserware-cli -- contexts
mise exec rust@1.94.1 -- cargo run -p browserware-cli -- open https://example.com
```

## AI Assistant Context

`just setup` creates assistant-friendly symlinks. For coding agents working in this repo, start with [AGENTS.md](AGENTS.md), load [`.context/RUST_MODERN.md`](.context/RUST_MODERN.md), and use `mise exec` or the `just ci-*` recipes whenever a task depends on a specific Rust or cargo-tool version.

## License

By contributing, you agree that your contributions will be dual licensed under MIT OR Apache-2.0.
