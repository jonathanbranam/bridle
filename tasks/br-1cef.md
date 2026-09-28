+++
id = "br-1cef"
title = "Cut build cost: warm new worktrees' target/, workers run check-affected (b7cz)"
kind = "feature"
state = "dropped"
created_at = "2026-09-28T22:29:42.182Z"
updated_at = "2026-09-28T23:36:33.666036Z"
+++

ticket: docs/questions/open/build-cost-on-the-laptop-b7cz.md (options 1 and 2 are the scope)
original id: b7cz
see also: mt7r (just check-affected, recipe exists), c8qw/br-6668 (CI watcher; CI runs the
full suite on main), nbkj, f75x

Why: the human wants laptop load down (load hit 96 on 16 cores; barely usable while bridle
runs) and is weighing the NUC. Each worker's fresh worktree compiles the whole workspace
cold (about 2.5 GB target/) before `just check` runs. Recommendation in the ticket: do 1
and 2 together; both small, no new tools, work on the NUC too.

1. Warm the new worktree's target/. In crates/bridle-daemon/src/worktree.rs (where the
   worktree is created for spawn), after `git worktree add`, clone the main clone's
   `target/` into `<worktree>/target` if it exists. On macOS use `cp -cR` (APFS
   copy-on-write: near-instant, no extra disk); elsewhere skip it (or a plain recursive
   copy only if trivial; KISS says skip). Never fail the spawn if the copy fails or target/
   is missing: log a warning and continue. Only for worktree roles (workers), and it must
   not touch the main clone's target/. Make it a config switch defaulting on
   (`[worktrees] warm_target = true` or wherever worktree settings live; deny_unknown_fields
   applies). Non-Rust projects have no target/, so it's a no-op for them.
   Watch for: cargo fingerprints embed absolute paths, so some crates rebuild anyway;
   deps should hit. That's still the win. Don't try to fix it further.
2. Workers run `just check-affected` for their own gate instead of the full suite; CI (and
   c8qw's watcher) runs everything on main. CLAUDE.md says check-affected never replaces
   `just check` as the full suite, so keep both recipes; change only what the worker is
   told to run. Look at how `{{commands.check}}` is set and used (config.rs, sync.rs, the
   worker role/skill in workflow/base). Simplest: bridle's own project sets the worker's
   check to `just check-affected` via config, leaving the substitution mechanism as is and
   other projects unaffected. If that needs a second setting (e.g. `commands.check_worker`
   defaulting to `commands.check`), add that one; don't redesign. Update .bridle/ project
   files and docs/design/agent-host/roles-and-config.md accordingly. Coordinate with br-42b1
   (w2rp), which is editing the same role/skill files and the substitution code: run after
   it merges, or rebase carefully.

Measure: in the task's final message, report wall time of the first build in a fresh
worktree with and without the warm copy (one run each is enough).

Acceptance: `just check` passes; a test that spawn creates the worktree with target/ cloned
when the main clone has one and doesn't fail when it doesn't (use a temp dir, no real
cargo build); a test for the config switch off; docs updated (worktree docs in
docs/design/agent-host/).

Out of scope: sccache, a shared CARGO_TARGET_DIR (unsafe with concurrent workers), moving to
the NUC, doc-only CI paths-ignore, changing check-affected itself.

Model: Sonnet.

## Thread

### note · agent:pm-1 · 2026-09-28T23:36:33.666Z
dropped: Merged to main.
