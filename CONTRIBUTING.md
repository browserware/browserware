# Contributing to Browserware

Thank you for your interest in contributing! This guide covers development setup and workflow for the main browserware repository.

For organization-wide policies, see the [.github repository](https://github.com/browserware/.github).

## Prerequisites

- **Rust 1.88+** (Edition 2024)
- **Git** with DCO sign-off configured

Verify your Rust version:

```bash
rustc --version  # Should be 1.88.0 or later
```

## Development Setup

```bash
git clone https://github.com/browserware/browserware.git
cd browserware
just setup    # Creates AI assistant symlinks
cargo build --workspace
cargo test --workspace
```

> **Note**: Install [just](https://github.com/casey/just) with `cargo install just`

### Useful Commands

```bash
just          # List all available tasks
just check    # Run all CI checks
just fmt      # Format code
just test     # Run tests
just docs     # Build and open documentation
just clean    # Clean build artifacts + symlinks
```

Or directly with cargo:

```bash
cargo test -p browserware-types    # Test specific crate
cargo run -p browserware-cli -- --help
```

## Code Quality

This project enforces strict quality standards:

| Check | Command | CI Job |
|-------|---------|--------|
| Formatting | `cargo fmt --all -- --check` | `fmt` |
| Linting | `cargo clippy --workspace --all-targets -- -D warnings` | `clippy` |
| Tests | `cargo test --workspace` | `test` |
| Dependencies | `cargo deny check` | `deny` |
| Documentation | `cargo doc --workspace --no-deps` | `docs` |

All checks must pass before merge.

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
cargo test --workspace
```

### Integration Tests

```bash
cargo test -p browserware-cli --test cli
```

### Manual Testing

```bash
cargo run -p browserware-cli -- --help
cargo run -p browserware-cli -- browsers
cargo run -p browserware-cli -- open https://example.com
```

## License

By contributing, you agree that your contributions will be dual licensed under MIT OR Apache-2.0.
