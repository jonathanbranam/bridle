+++
id = "br-rk4z"
title = "Settle default 5 minutes; shipped text names the per-project override"
kind = "chore"
state = "integrated"
created_at = "2026-11-17T12:17:34.352Z"
updated_at = "2026-11-17T15:49:05.507725Z"
created_by = "external:advisor@remus"
watchers = [
    "external:advisor@remus",
    "agent:rk4z",
]
branch = "bridle/rk4z"
commit = "cd63f5274ca46cf116b052b8b60acd9f533785bc"
summary = """Default [tasks] settle is now 5m (was 10m). Changed: DEFAULT_SETTLE and the invalid-value fallback in crates/bridle-daemon/src/config.rs (warning now says 5m); config test expectations; settle_hold_test sets 5m explicitly. Shipped role text (project-manager.md, manager.md, orchestrator.md reference) says default 5 minutes and that a project's .bridle/config.toml [tasks] settle overrides it (longer or shorter, 0 turns it off). Design docs updated: roles-and-config.md, coordination.md "Settling". The override stays in config (the daemon enforces it), not a rule file. The br-rqrd skip wording is untouched. No migration: projects relying on the default get 5m on the next daemon upgrade; CHANGELOG line under Changed says so. Historical resolved/open tickets still say 10m and were left alone."""
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

### note · agent:rk4z · 2026-11-17T15:27:34.042Z
built: df5d1fae (branch bridle/rk4z)

### note · agent:rk4z · 2026-11-17T15:27:36.208Z
done: default [tasks] settle is 5m (daemon default and invalid fallback), shipped role text and design docs say a project's .bridle/config.toml [tasks] settle overrides it; br-rqrd skip wording untouched; CHANGELOG under Changed notes the 5m upgrade. just check exit 0 with BRIDLE_* unset, 2013 tests run, 2013 passed, 6 skipped; commit df5d1fae (branch bridle/rk4z contains main; main unchanged since). Ticket summary written, branch pushed, marked built.

### note · agent:manager-2 · 2026-11-17T15:27:46.337Z
integrated: cd63f5274ca46cf116b052b8b60acd9f533785bc (branch bridle/rk4z)

### note · agent:manager-2 · 2026-11-17T15:27:53.838Z
cleanup: removed origin branch bridle/rk4z; kept agent rk4z (background job pid 11172)

### note · agent:rk4z · 2026-11-17T15:49:05.507Z
CI on main (run 44114639704): the ubuntu check job passed. The macos-latest check job was cancelled after 15m without acquiring a hosted runner (conclusion cancelled, zero steps run, annotation: job not acquired by Runner of type hosted after multiple attempts), so no code failure appears in CI. I cannot read the job logs: gh run view --log-failed prints nothing and gh run view --job --log says log not found. Nothing to fix in code; I found no test or doc check still expecting 10m. Branch: merged current main (cd63f527) into bridle/rk4z, tip e11acc05 (merge commit, no code change); just check run with env -i, exit 0, 2013 tests run, 2013 passed, 6 skipped. Pushed. The macOS runner may need a re-run from you.
