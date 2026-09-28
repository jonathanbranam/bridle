# Bridle dev tasks. `just` lists them.
default:
    @just --list

# Format, lint, test: what CI runs.
check: fmt-check lint test

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

lint:
    cargo clippy --workspace --all-targets -- -D warnings

test *args:
    cargo nextest run --workspace {{args}}

# Tests that spawn real `claude` (costs tokens; Haiku, tiny prompts).
test-live:
    BRIDLE_LIVE_TESTS=1 cargo nextest run --workspace --run-ignored only --no-capture

# The Claude Code contract (live, ~$0.10). Run after Claude Code updates
# itself: on success the version is recorded as verified; on failure, fix forward.
test-contract:
    BRIDLE_LIVE_TESTS=1 cargo nextest run -p bridle-claude --test contract_test --run-ignored only --no-capture
    claude --version > crates/bridle-claude/tests/contract-verified.txt
    @echo "verified: $(cat crates/bridle-claude/tests/contract-verified.txt)"

deny:
    cargo deny check

build:
    cargo build --workspace

# Run the daemon in the foreground against a repo: `just serve ../some-clone`
serve repo:
    cargo run -p bridle -- serve --repo {{repo}}

install:
    cargo install --path crates/bridle --locked
