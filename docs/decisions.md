# Technical Decisions

Lightweight Architecture Decision Records (ADRs) for tooling and configuration choices.

## CI/CD

### MSRV Testing

**Decision**: Test MSRV via CI matrix, not `cargo-msrv` tool.

**Reason**: `taiki-e/install-action@cargo-msrv` doesn't exist. The cargo-msrv tool requires separate installation and is slower than just running the build with the MSRV toolchain.

**Implementation**:
```yaml
matrix:
  rust: [stable, "1.88"]  # 1.88 is MSRV
```

### GitHub Actions Versions

As of January 2026:
- `actions/checkout@v6`
- `actions/cache@v5`
- `dtolnay/rust-toolchain@master` (or `@stable`)
- `taiki-e/install-action@<tool>` for cargo-deny, cargo-audit, cross

## Rust Tooling

### assert_cmd

**Decision**: Use `cargo_bin_cmd!` macro, not `Command::cargo_bin()`.

**Reason**: `Command::cargo_bin()` is deprecated as of assert_cmd 2.x due to incompatibility with custom cargo build directories.

**Implementation**:
```rust
use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin_cmd;

fn brw() -> Command {
    cargo_bin_cmd!("brw")
}
```

### cargo-deny v2

**Decision**: Use cargo-deny v2 configuration format.

**Changes from v1**:
- `[licenses]` section requires `version = 2`
- `unlicensed` key removed (deny is now default)
- `unmaintained`, `yanked`, `notice` take graph filter values (`"all"`, `"workspace"`, `"transitive"`, `"none"`), not lint levels (`"warn"`, `"deny"`)
- `wildcards = "warn"` for workspace path dependencies (they appear as wildcards)

**Implementation**:
```toml
[licenses]
version = 2
allow = ["MIT", "Apache-2.0", ...]

[advisories]
unmaintained = "workspace"  # not "warn"

[bans]
wildcards = "warn"  # path deps trigger this
```

## Version Control

### Cargo.lock

**Decision**: Commit `Cargo.lock` for all crates.

**Reason**: Cargo team guidance changed in August 2023. Benefits:
- Reproducible builds
- Easier bisecting for debugging
- Reliable CI (no surprise breakage from new dependency releases)
- MSRV testing with consistent dependency versions

**Reference**: [Cargo Blog - Change in Guidance on Committing Lockfiles](https://blog.rust-lang.org/2023/08/29/committing-lockfiles/)

### Release workflow

**Decision**: Use `--locked` flag in release builds.

**Reason**: Ensures the committed Cargo.lock is respected, preventing builds with different dependency versions.

```yaml
cargo build --release --locked --target ${{ matrix.target }}
```

## Development Workflow

### Task Runner

**Decision**: Use `just` (justfile) for development tasks.

**Reason**:
- Becoming Rust ecosystem standard
- Better syntax than Makefile
- Cross-platform
- Simple dependency-free recipes

**Setup**: `cargo install just`, then `just setup`
