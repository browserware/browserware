# Browserware development tasks
# Install: cargo install just
# Usage: just <recipe>

# List available recipes
default:
    @just --list

# Initial setup after clone
setup:
    @echo "Creating AI assistant symlinks..."
    ln -sf AGENTS.md CLAUDE.md
    ln -sf AGENTS.md GEMINI.md
    ln -sf AGENTS.md CURSOR.md
    ln -sf AGENTS.md COPILOT.md
    @echo "Done. Run 'just check' to verify everything works."

# Run all checks (same as CI)
check:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
    cargo deny check

# Format code
fmt:
    cargo fmt --all

# Run tests
test:
    cargo test --workspace

# Build release
build:
    cargo build --workspace --release

# Build documentation
docs:
    cargo doc --workspace --no-deps --open

# Clean build artifacts
clean:
    cargo clean
    rm -f CLAUDE.md GEMINI.md CURSOR.md COPILOT.md
