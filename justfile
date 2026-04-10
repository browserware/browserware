# Browserware development tasks
# Install: cargo install just
# Usage: just <recipe>

set shell := ["bash", "-euo", "pipefail", "-c"]

stable_rust := "rust@1.94.1"
msrv_rust := "rust@1.88.0"
just_tool := "cargo:just@1.49.0"
deny_tool := "cargo:cargo-deny@0.19.1"
audit_tool := "cargo:cargo-audit@0.22.1"
windows_target := "x86_64-pc-windows-msvc"
linux_target := "x86_64-unknown-linux-gnu"

# List available recipes
default:
    @just --list

# Initial setup after clone
setup: install-tools
    @echo "Creating AI assistant symlinks..."
    ln -sf AGENTS.md CLAUDE.md
    ln -sf AGENTS.md GEMINI.md
    ln -sf AGENTS.md CURSOR.md
    ln -sf AGENTS.md COPILOT.md
    @echo "Done. Run 'just validate' to mirror the local pre-PR checks."

# Install the pinned toolchain and cargo subcommands from mise.toml
install-tools:
    mise install --yes {{stable_rust}} {{just_tool}} {{deny_tool}} {{audit_tool}}

# Install cross-check targets used for optional CI smoke checks on non-native platforms
install-ci-targets:
    mise exec {{stable_rust}} -- rustup target add {{linux_target}} {{windows_target}}
    mise exec {{msrv_rust}} -- rustup target add {{linux_target}} {{windows_target}}

# Run all checks (same as CI)
check: validate

# Run the local pre-PR validation pipeline. This mirrors the CI job coverage
# available on the current host and pins tool versions through mise.
validate: ci-rustfmt ci-clippy ci-clippy-linux-stable ci-docs ci-deny ci-audit ci-test-stable ci-test-msrv
    @echo "Validation passed for rustfmt, host clippy, Linux CI clippy, docs, cargo-deny, cargo-audit, stable tests, and MSRV tests."

# Optional compile-only CI smoke tests for non-native targets.
validate-targets: ci-check-linux-stable ci-check-linux-msrv ci-check-windows-stable ci-check-windows-msrv
    @echo "Cross-target compile checks passed for Linux and Windows on stable and MSRV."

# CI-aligned job recipes
ci-rustfmt:
    mise exec {{stable_rust}} -- rustup component add rustfmt
    mise exec {{stable_rust}} -- cargo fmt --all -- --check

ci-clippy:
    mise exec {{stable_rust}} -- rustup component add clippy
    mise exec {{stable_rust}} -- cargo clippy --workspace --all-targets --locked -- -D warnings

ci-clippy-linux-stable: install-ci-targets
    mise exec {{stable_rust}} -- cargo clippy --workspace --all-targets --locked --target {{linux_target}} -- -D warnings

ci-docs:
    RUSTDOCFLAGS="-D warnings" mise exec {{stable_rust}} -- cargo doc --workspace --no-deps --all-features --locked

ci-deny:
    mise exec {{stable_rust}} -- cargo deny check advisories
    mise exec {{stable_rust}} -- cargo deny check bans
    mise exec {{stable_rust}} -- cargo deny check licenses
    mise exec {{stable_rust}} -- cargo deny check sources

ci-audit:
    db_dir="$(mktemp -d "${TMPDIR:-/tmp}/browserware-cargo-audit.XXXXXX")"; trap 'rm -rf "$db_dir"' EXIT; mise exec {{stable_rust}} -- cargo audit --deny warnings --db "$db_dir"

ci-test-stable:
    mise exec {{stable_rust}} -- cargo test --workspace --all-targets --locked

ci-test-msrv:
    mise exec {{msrv_rust}} -- cargo test --workspace --all-targets --locked

ci-check-linux-stable: install-ci-targets
    mise exec {{stable_rust}} -- cargo check --workspace --all-targets --locked --target {{linux_target}}

ci-check-linux-msrv: install-ci-targets
    mise exec {{msrv_rust}} -- cargo check --workspace --all-targets --locked --target {{linux_target}}

ci-check-windows-stable: install-ci-targets
    mise exec {{stable_rust}} -- cargo check --workspace --all-targets --locked --target {{windows_target}}

ci-check-windows-msrv: install-ci-targets
    mise exec {{msrv_rust}} -- cargo check --workspace --all-targets --locked --target {{windows_target}}

# Format code
fmt:
    mise exec {{stable_rust}} -- rustup component add rustfmt
    mise exec {{stable_rust}} -- cargo fmt --all

# Run tests
test:
    mise exec {{stable_rust}} -- cargo test --workspace --all-targets --locked

# Build release
build:
    mise exec {{stable_rust}} -- cargo build --workspace --release --locked

# Build documentation
docs:
    RUSTDOCFLAGS="-D warnings" mise exec {{stable_rust}} -- cargo doc --workspace --no-deps --all-features --locked --open

# Clean build artifacts
clean:
    cargo clean
    rm -f CLAUDE.md GEMINI.md CURSOR.md COPILOT.md
