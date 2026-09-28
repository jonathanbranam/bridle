# Cleaning stale build artifacts

The main clone's `target/` directory grows without bound during development because cargo retains artifacts from all historical build hashes. On macOS, debug `.o` files are particularly problematic: each incremental rebuild adds new `.o` files for stale hashes without removing the old ones.

Observed: 28G in `target/` (21G in `target/debug/deps` alone, ~186,000 `.o` files), and ~400s `just check` time. After `cargo clean`, time dropped to ~35s.

## Strategy

Cleanup happens **only during orchestrator budget-hold maintenance windows**, when no builds are in flight. This ensures serialization: cleanup is never concurrent with `just check` in the main clone.

The orchestrator runs the cleanup recipe before reinstalling the daemon:

```bash
just clean-stale  # Remove artifacts older than 7 days (default)
cargo install --path crates/bridle
```

## Implementation

### `just clean-stale`

Uses `cargo sweep` to remove artifacts older than a threshold (7 days by default). `cargo sweep` is safer than `cargo clean`: it preserves recent artifacts and only removes old ones.

If `cargo sweep` is not installed, falls back to `cargo clean` (nuclear; removes all artifacts).

The recipe is:

```bash
just clean-stale [days]    # Default: 7 days
```

### Serialization

The orchestrator's budget hold maintenance window is the sole time cleanup runs. It is inherently serialized: the orchestrator runs cleanup before reinstalling the daemon, and no workers build during this window.

Worker worktrees are cleaned up separately via `bridle rm --delete-branch` after merge.
