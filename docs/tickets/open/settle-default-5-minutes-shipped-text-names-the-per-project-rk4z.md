---
id: br-rk4z
title: Settle default 5 minutes; shipped text names the per-project override
kind: chore
created: 2026-11-17
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: [br-rqrd]
related: []
---

## The ask

Make the settle period 5 minutes by default, and have the shipped role text say a project's own setting overrides it.

The human, 2026-11-17, via advisor/flume (relayed by advisor@remus), verbatim: "I do want to override that timing per project. I don't know if that needs to be a formal thing, but I think the bridle rule should end up saying: 'Does it cool down for each project? The default is 10 minutes. We could make the default 5 minutes.' Anyway, there can be a per-project override of the cooldown length, and that would live in a local bridle rule. Let's make the cooldown 5 minutes globally, and then if a project wants it longer, they can make their own rule to lower it. The official shipped bridle rules should indicate that any project-local cooldown override can override this."

## What to change

- Default `[tasks] settle` from `10m` to `5m`: the daemon default and the invalid-value fallback (crates/bridle-daemon/src/config.rs, grep `settle`), and every place that states 10 minutes: docs/design/agent-host/roles-and-config.md, workflow/base/roles/project-manager.md, workflow/base/roles/manager.md, coordination.md ("Settling"). grep `10m` and `10 minutes` near `settle`.
- The shipped settle text says the length is per project: "default 5 minutes; a project's `.bridle/config.toml` `[tasks] settle` overrides it (longer or shorter, `0` turns it off)".
- The override stays in config, not a rule file: the daemon enforces `[tasks] settle` (it refuses a spawn during settle) and a rule file cannot. The shipped text points to it. If the human wants a rule file instead they will say so.

## Migration

None needed: a project that sets `[tasks] settle` keeps it. Projects relying on the default get 5m on the next daemon upgrade; say so in the CHANGELOG entry.

## Ordering

This edits the same settle text as br-rqrd (never skip unless the human explicitly says so). Blocked by br-rqrd (real edge: same lines in project-manager.md): rqrd first, then this one rebases on it.

## Acceptance

Config default test and any test quoting 10m updated; `bridle prime project-manager` and `bridle prime manager` show 5 minutes and the override sentence; `just check` with BRIDLE_* env unset. Model: Haiku (mechanical), Sonnet only if a test needs reworking.
