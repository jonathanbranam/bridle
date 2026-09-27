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

test:
    cargo nextest run --workspace

# Tests that spawn real `claude` (costs tokens; Haiku, tiny prompts).
test-live:
    BRIDLE_LIVE_TESTS=1 cargo nextest run --workspace --run-ignored only --no-capture

deny:
    cargo deny check

build:
    cargo build --workspace

# Run the daemon in the foreground against a repo: `just serve ../some-clone`
serve repo:
    cargo run -p bridle -- serve --repo {{repo}}

install:
    cargo install --path crates/bridle --locked
