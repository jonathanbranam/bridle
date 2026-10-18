---
id: br-nneb
title: "A command to make a running daemon re-read its config (design: which keys apply live, and how it fails safe)"
kind: feature
created: 2026-10-18
created_by: external:aide
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-9966, br-8fsx, br-7ytg]
---

## The ask

The human, 2026-10-18 ~10:45 AM ET (via aide), verbatim: "maybe we just need a command or something to re-read config? That is a separate question that needs a reasonable design to not behave poorly."

Context: today a daemon reads `.bridle/config.toml` (and `~/.bridle/config.toml`) at start, and again only after a workflow fetch (crates/bridle-daemon/src/lib.rs). Any other config change needs `bridle daemon restart`, which drains and restarts agents. br-9966 is adding one narrow live read (the orchestrator `autostart` key, read only while the orchestrator is down) because the human wants it to work without a restart.

Questions the design must answer before anything is built:
- Which keys can apply live and which need a restart (ports, store paths, tokens can't; role prompts, limits, `[orchestrator]` thresholds and autostart probably can); a reload that touches a restart-only key says so and changes nothing for it.
- A bad file: parse and validate first; on any error keep the running config whole and report the error (no half-applied config).
- What running agents and sessions see (their spawned settings don't change mid-turn).
- The command (e.g. `bridle daemon reload [--project]`), who may run it, and what it reports (the keys that changed, the ones that need a restart).
- Whether br-9966's live autostart read should become just a reload, or stay as is.

Needs a design pass and the human's decision; not to be readied without it.
