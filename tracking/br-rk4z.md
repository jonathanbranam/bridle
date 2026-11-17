+++
id = "br-rk4z"
title = "Settle default 5 minutes; shipped text names the per-project override"
kind = "chore"
state = "planned"
created_at = "2026-11-17T12:17:34.352Z"
updated_at = "2026-11-17T13:04:43.688747Z"
created_by = "external:advisor@remus"
watchers = ["external:advisor@remus"]
ticket = "rk4z"
+++

docs/tickets/open/settle-default-5-minutes-shipped-text-names-the-per-project-rk4z.md

submitted by external:advisor@remus

## The ask

Make the settle period (the cooldown before work on a ticket starts) 5 minutes by default, and have
the shipped role text say that a project's own setting overrides it.

The human, 2026-11-17 (via advisor, flume): "I do want to override that timing per project. I don't
know if that needs to be a formal thing, but I think the bridle rule should end up saying: 'Does it
cool down for each project? The default is 10 minutes. We could make the default 5 minutes.'
Anyway, there can be a per-project override of the cooldown length, and that would live in a local
bridle rule. Let's make the cooldown 5 minutes globally, and then if a project wants it longer, they
can make their own rule to lower it. The official shipped bridle rules should indicate that any
project-local cooldown override can override this."

What to change:

- Default `[tasks] settle` from `10m` to `5m` (daemon default and the invalid-value fallback), and
  every place that states 10 minutes: docs/design/agent-host/roles-and-config.md,
  workflow/base/roles/project-manager.md, workflow/base/roles/manager.md, coordination.md
  ("Settling").
- The shipped settle text says the length is per project: "default 5 minutes; a project's
  `.bridle/config.toml` `[tasks] settle` overrides it (longer or shorter, `0` turns it off)".

Advisor's note on "a local bridle rule": the per-project override already exists as config
(`[tasks] settle`), and the daemon enforces it (it refuses a spawn during settle), which a rule
file can't. Recommended: the override stays in config, and the shipped text points to it. If the
human wants a rule file instead, they will say so.

Related: br-rqrd edits the same settle text (never skip it unless the human explicitly says so;
bridle-only critical-fix exception; reach the orchestrator acting as PM). Do the two together.

## Thread

### note · external:advisor@remus · 2026-11-17T12:17:34.353Z
submitted by external:advisor@remus

### note · external:orchestrator · 2026-11-17T12:18:03.042Z
readied by external:orchestrator
