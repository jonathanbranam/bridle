---
id: br-j4bk
title: "integration/target grows without limit (833K files, 152 GB): prune it so syspolicyd stops stalling every land, check and self-upgrade"
kind: fix
created: 2026-10-14
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-z7y5, br-q7mv, br-aqa7]
---

Critical (orchestrator, 2026-10-14): it breaks lands and the self-upgrade. The human's standing call: critical work (a crash, a block, a broken build) the orchestrator acts on itself. Incident br-z7y5 (2026-10-06) found the cause and named this fix; it was never built.

## What happened today (2026-10-14)

- `integration/target/debug/deps` held **833,634 files** (152 GB); z7y5 saw 564K and the machine stalled then. The clone's `target/debug/deps` has ~12K.
- syspolicyd ran at 70-260% CPU all morning assessing each new ad-hoc-signed test binary. nextest's `--list` of each binary waited 8+ min to exec at a load of ~2; the nncj land and nbgp's `just check` ran past their background time limits; the 14:31Z self-upgrade (18780a46) was refused because the new binary's self-check timed out (3 x 60 s; same shape as q7mv/aqa7).
- `worktree::warm_target` (`crates/bridle-daemon/src/worktree.rs`) clones this directory into every new worktree with `cp -cR`, so each spawn copies the whole pile.

## The ask

Keep the integration target (the warm source) bounded, automatically:

- After each land's integration check (and/or the warm build), remove artifacts the current build didn't produce or use: for example delete `target/debug/deps`, `target/debug/build`, `target/debug/incremental` entries older than the run's start, or clean when the file count passes a threshold. No dependency on `cargo-sweep` (not installed here).
- `warm_target` skips the copy (continues cold, with a log line) when the source is over a threshold, so a bloated source can't hold a spawn.
- Docs: `docs/design/agent-host/` where warm_target/integration are described; CHANGELOG.

Files likely: `crates/bridle-daemon/src/integrator.rs`, `worktree.rs`, `warm_build.rs`, their tests.

Acceptance: a test that a stale artifact older than the run is pruned and a current one kept; a test that warm_target skips an over-threshold source; `just check`.

Model: Sonnet. Size: S.

Out of scope: a shared CARGO_TARGET_DIR or sccache (z7y5's bigger option; its own design), Spotlight / Developer Tools settings (the human's machine), worktrees' own target/ dirs.

Meanwhile the orchestrator moves `integration/target` aside between lands by hand (2026-10-14).
