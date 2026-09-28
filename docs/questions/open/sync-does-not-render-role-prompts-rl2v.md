---
id: rl2v
title: bridle sync does not render role prompts from layers
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: []
---

## What happened

br-1e0c moved the 5 driver-facing role prompts (`worker`, `manager`, `product-manager`,
`orchestrator`, `advisor`) from `.bridle/roles/` into `workflow/base/roles/`, so a second
project (data-contracts) can reuse them via the `workflow` layer path instead of copying
the files. `.bridle/config.toml`'s `system_prompt = "workflow/base/roles/<role>.md"` keys
were updated to the new location, and the daemon (`crates/bridle-daemon/src/config.rs`,
`stable_system_prompt`) still reads that path directly with `std::fs::read_to_string`, so
this works today.

But `bridle sync` (`crates/bridle-daemon/src/sync.rs`) does not know about `roles/` as a
layer content type: it only discovers `rules/`, `skills/`, `hooks/` and `agents/` (the
last renders Claude Code subagent defs into `.claude/agents/*.md` — a different thing,
confirmed while doing br-1e0c: see `docs/design/agent-host/roles-and-config.md` and
`docs/design/workflow-layers.md`). A project onboarding onto the base layer still has to
hand-write its own `system_prompt = "workflow/base/roles/<role>.md"` line per role in its
own `.bridle/config.toml`; nothing is resolved or rendered automatically the way rules,
skills and Claude Code agent defs are.

## Options

1. Leave it: a project's `.bridle/config.toml` already has to declare `[roles.*]` by hand
   for models/tools/etc, so one more explicit path per role is a small addition, not a new
   burden.
2. Give role prompts a default path convention (e.g. `system_prompt` defaults to
   `<workflow>/base/roles/<role-name>.md` when unset) so a project only needs to override
   it, the same way rules/skills need no per-project wiring for the common case.
3. Extend `bridle sync`'s layer-content conventions (`docs/design/workflow-layers.md`,
   "Rendering into what the agent harness reads") to cover `roles/<role>.md` explicitly,
   if it turns out the daemon should resolve role prompts through layer overlay (project
   overrides a base role prompt) rather than a single static path.

Not resolved here — flagged as a follow-up, out of scope for br-1e0c.
