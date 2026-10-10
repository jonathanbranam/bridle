# Bridle dev tasks. `just` lists them.
default:
    @just --list

# Format, lint, test: what CI runs.
check: fmt-check lint specs-check bench-test test

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

# Bridle's own specs (design/specs/): well formed, with an id on everything, and every
# executable scenario's id named in a test under crates/.
specs-check:
    cargo run -q -p bridle -- workflow spec check --require-ids
    cargo run -q -p bridle -- workflow spec coverage --tests crates --require-all

# The benchmark sampler's unit tests (python3 stdlib; never samples).
bench-test:
    python3 -I scripts/bench/test_passive_sample.py

lint:
    cargo clippy --workspace --all-targets -- -D warnings

# Git runs with an empty global config, like a CI runner, so a developer's
# `init.defaultBranch` or identity can't hide a failure (g3ck).
test *args:
    GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1 cargo nextest run --workspace {{args}}

# Tests that spawn real `claude` (costs tokens; Haiku, tiny prompts).
test-live:
    BRIDLE_LIVE_TESTS=1 cargo nextest run --workspace --run-ignored only --no-capture

# The Claude Code contract (live, ~$0.10). Run after Claude Code updates
# itself: on success the version is recorded as verified; on failure, fix forward.
test-contract:
    BRIDLE_LIVE_TESTS=1 cargo nextest run -p bridle-claude --test contract_test --run-ignored only --no-capture
    claude --version > crates/bridle-claude/tests/contract-verified.txt
    @echo "verified: $(cat crates/bridle-claude/tests/contract-verified.txt)"

# Fast path for local iteration only: run nextest for just the crates a
# change can affect (changed crates + their reverse-dependency closure), via
# cargo metadata. Crate-level heuristic, not strict modularity: falls back to
# the full `just test` whenever it can't be sure (changes outside crates/,
# no base ref, or the computation itself fails). `just check` — the merge
# gate — always runs the full suite and is unaffected by this recipe.
check-affected base='':
    #!/usr/bin/env bash
    set -euo pipefail

    base="{{base}}"
    if [ -z "$base" ]; then
        if ! base=$(git merge-base main HEAD); then
            echo "no merge-base with main; falling back to full suite" >&2
            exec just test
        fi
    fi

    changed=$(git diff --name-only "$base" --) || { echo "git diff failed; falling back to full suite" >&2; exec just test; }
    if [ -z "$changed" ]; then
        echo "no changes since $base; nothing to test"
        exit 0
    fi

    if echo "$changed" | grep -qvE '^crates/'; then
        echo "changes outside crates/; falling back to full suite" >&2
        exec just test
    fi

    meta=$(cargo metadata --format-version 1 --no-deps) || { echo "cargo metadata failed; falling back to full suite" >&2; exec just test; }

    # "dir name" lines, one per workspace crate (dir under crates/, package name).
    dirnames=$(jq -r '.packages[] | "\(.manifest_path | split("/") | .[-2]) \(.name)"' <<<"$meta")
    # "dep dependent" lines: dependent has a path-dependency on dep.
    redges=$(jq -r '.packages[] | .name as $n | .dependencies[] | select(.path != null) | "\(.name) \($n)"' <<<"$meta")

    dirs=$(echo "$changed" | awk -F/ '{print $2}' | sort -u)

    affected=""
    for dir in $dirs; do
        name=$(echo "$dirnames" | awk -v d="$dir" '$1==d{print $2}')
        if [ -z "$name" ]; then
            echo "changed file under unrecognized crate dir '$dir'; falling back to full suite" >&2
            exec just test
        fi
        case " $affected " in
            *" $name "*) ;;
            *) affected="$affected $name" ;;
        esac
    done

    # Reverse-dependency closure: fixpoint over "dep dependent" edges.
    grew=1
    while [ "$grew" = 1 ]; do
        grew=0
        for name in $affected; do
            for dependent in $(echo "$redges" | awk -v n="$name" '$1==n{print $2}'); do
                case " $affected " in
                    *" $dependent "*) ;;
                    *) affected="$affected $dependent"; grew=1 ;;
                esac
            done
        done
    done

    args=""
    for name in $affected; do
        args="$args -p $name"
    done

    echo "affected crates:$affected"
    cargo nextest run $args

deny:
    cargo deny check

build:
    cargo build --workspace

# Run the daemon in the foreground against a repo: `just serve ../some-clone`
serve repo:
    cargo run -p bridle -- serve --repo {{repo}}

# On a Mac with the "bridle local signing" identity (`just sign-setup`), the installed binary
# is re-signed with it so the firewall's Allow survives rebuilds; else the ad-hoc signature stays.
install:
    cargo install --path crates/bridle --locked
    "${CARGO_HOME:-$HOME/.cargo}/bin/bridle" sign binary "${CARGO_HOME:-$HOME/.cargo}/bin/bridle"

# One time per Mac, works over SSH: create the local signing certificate (asks for the login
# keychain password once; or set BRIDLE_KEYCHAIN_PASSWORD).
sign-setup:
    cargo run -q -p bridle -- sign setup

# Remove stale build artifacts older than N days (default: 7).
# Runs only during orchestrator maintenance windows to avoid conflicts with builds.
# Uses `cargo sweep` if available; falls back to `cargo clean`.
clean-stale days='7':
    #!/usr/bin/env bash
    set -euo pipefail
    if command -v cargo-sweep &> /dev/null; then
        cargo sweep -r -t {{days}}
    else
        echo "cargo sweep not found; falling back to cargo clean" >&2
        cargo clean
    fi

# Regenerate the gateway's TypeScript types (crates/bridle-gateway/bindings/) for bridle-ui.
gateway-types:
    BRIDLE_UPDATE_TYPES=1 cargo nextest run -p bridle-gateway committed_types_are_current
