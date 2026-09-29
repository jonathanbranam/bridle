+++
id = "br-57be"
title = "P5: port registry: bridle port alloc/release/list with reserved ports"
kind = "feature"
state = "integrated"
created_at = "2026-09-29T06:50:51.728Z"
updated_at = "2026-09-29T07:24:21.976872Z"
branch = "bridle/port-registry"
commit = "3ce74df7284289aa65674fbf6da2d22e6e2d1e7c"
summary = "Port registry: `ports` table (SCHEMA_V16, runtime state, not on the state branch), `[ports] range/reserved` config, `POST/GET /v1/ports` + release, `bridle port alloc|release|list`. Alloc (crates/bridle-daemon/src/ports.rs) takes the lowest port in range not reserved, allocated or bind-failing on 127.0.0.1; owner is the agent's stable id (or principal id for non-agents), task is the caller's claimed task. Freed on agent exit (supervisor hook) and by a 30s tick (dead pid, owner not running). Adds a `label` column beyond the brief for --label. track-web's dev-servers rule was not found under /Volumes/Data/work, so its needs weren't checked. Not built: PORT env injection."
+++

Goal (docs/design/worktrees-and-ports.md; build-order P5): track-web workers run dev servers; the human's ports must never be taken and 'stop what you start' must be checkable. Find track-web's rule on it (its .bridle/rules/dev-servers.md under the workspace, /Volumes/Data/work/*, read-only) and match its needs.

Do: table `ports` (port, agent, task, pid, allocated_at) in the SQLite store (schema migration, docs/design/storage.md; it is runtime state, NOT on the state branch, not rebuilt). Config `[ports] range = [4000, 4999]`, `reserved = [..]` in .bridle/config.toml (crates/bridle-daemon/src/config.rs). API + CLI: `bridle port alloc [--pid N] [--label L]` returns a free port in range not reserved, not allocated, and not currently listening on the host (bind test on 127.0.0.1), recorded against the calling agent and its claimed task; `bridle port release <port>`; `bridle port list [--json]`. The daemon frees a port when its owner agent stops/exits or when its pid is dead (check on the existing tick). Wire types in bridle-api/src/types.rs. Docs: worktrees-and-ports.md (built), storage.md, cli.md, config docs.

Acceptance: just check passes; tests: alloc skips reserved and taken ports, release, cleanup on agent exit. Model: Sonnet. Out of scope: injecting PORT into env, worktree layouts.

## Thread

### note · agent:manager-2 · 2026-09-29T07:24:21.976Z
integrated: 3ce74df7284289aa65674fbf6da2d22e6e2d1e7c (branch bridle/port-registry)
