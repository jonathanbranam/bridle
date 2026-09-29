+++
id = "br-eb7a"
title = "Doc audit: cli.md matches every CLI command and flag"
kind = "chore"
state = "integrated"
created_at = "2026-09-29T09:35:22.662Z"
updated_at = "2026-09-29T09:44:15.544927Z"
branch = "bridle/audit-cli"
commit = "23ec204d945a1e9bc77a31e245d5200fff367f41"
summary = """
Audited docs/design/cli.md against every `bridle ... --help`. Added to Built: `ready`, `queue` (+`set`/`add-tier`), `dep add|rm`, `wait`, `task plan`, the `budget` subcommands (hold/release/override/override-clear/max-workers), `arch propose --argument-file/--arch-root`, `goals propose --goals-root`; new bullets for `queue` and `budget`. Fixed: the `task` bullet (plan exists; no longer says nothing can reach `planned`), the `ready` bullet (highest startable tier, backlog excluded), stale "see Planned" pointer. Planned block trimmed to what's genuinely unbuilt (removed task plan, impact, conflict, spec, goals propose, arch propose, explore new/conclude/abandon, trace built ones). README.md and docs/README.md command mentions are all still valid; no change.

Wrong --help / code issues found (not fixed): (1) `bridle budget --help` (and any `bridle budget` parse) panics: clap debug assert, `schedule` has `conflicts_with = "action"` but no arg/group named `action` (cli.rs:783). (2) `bridle statusline` help says it "records a usage snapshot"; it no longer does (cli.md, s8kn). (3) `bridle prime` help says "Orchestrator only for now"; worker and planner are supported. (4) `bridle task` help says scoped to open/planned/claimed/...; omits `plan` state flow wording is stale. (5) `bridle usage --by` help says "role or model" but also accepts agent."""
+++

Goal: docs/design/cli.md is the CLI's reference; tonight added many commands (spec import/coverage/export flags, goals, arch, explore, impact, conflict, trace, port, probe, land, arch-guard, renew, wait, task plan/queue etc.). Run `bridle --help` and every subcommand's --help (recursively) and compare with cli.md: add missing commands/flags, fix wrong descriptions, move built items out of 'Planned' into 'Built', delete Planned entries that no longer apply. Keep the existing doc's style and density; don't paste whole --help output. Also check README.md's and docs/README.md's command mentions. Acceptance: just check passes (doc-link check); list in the thread what you changed. Model: Sonnet. Out of scope: code changes (report any --help text that is wrong).

## Thread

### note · agent:manager-2 · 2026-09-29T09:44:15.544Z
integrated: 23ec204d945a1e9bc77a31e245d5200fff367f41 (branch bridle/audit-cli)
