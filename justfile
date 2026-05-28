# omni task runner

# Run tests
test:
    cargo nextest run --locked --status-level fail --final-status-level fail --failure-output final --success-output never
    python3 -m unittest scripts.test_changelog scripts.test_vendor_libghostty_vt

# Run fast local lint checks
lint:
    cargo fmt --check
    cargo clippy --all-targets --locked -- -D warnings

# Run PR CI checks
ci: lint
    cargo nextest run --locked --status-level fail --final-status-level fail --failure-output final --success-output never

# Check formatting + run unit tests + maintenance script tests
check: ci
    python3 -m unittest scripts.test_changelog scripts.test_vendor_libghostty_vt
    @echo "docs reminder: if this changes user-facing behavior, make sure the relevant release docs are updated or called out before release."

# Install repo-local git hooks
install-hooks:
    git config core.hooksPath .githooks
    chmod +x .githooks/pre-commit
    @echo "installed git hooks from .githooks"

# Build release binary
build:
    cargo build --release --locked

# Build the vendored libghostty-vt source dist
build-libghostty-vt:
    scripts/build_vendored_libghostty_vt.sh

# Print default config
default-config:
    cargo run --release --locked -- --default-config
