---
id: d48r
title: Machine load notes go to aide, not the orchestrator
kind: feature
opened: 2026-10-09
filed_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: []
---

## The ask

The human, 2026-10-09 ~6:10 PM ET, to the orchestrator: "send the machine load messaging to aide and deal with it".

Today the load watch sends its note ("Machine load is high: ... Don't add work: wait.") to `external:orchestrator`
only; the recipient is hard-coded at `crates/bridle-daemon/src/load.rs` (`ToTarget::External(crate::wake::ORCHESTRATOR)`).
While builds run, load hovers around the threshold and the note fires on every crossing: on 2026-10-09 evening
it woke the orchestrator about every two minutes on each of three daemons, and the orchestrator has nothing to do
with it but wait.

Change:
- The load note goes to `external:aide` instead of the orchestrator (one per crossing, as now).
- The orchestrator no longer gets it, so a load note is no longer an orchestrator wake: update
  `workflow/base/roles/orchestrator.md` (the wake list and "machine load note" bullet) and
  `docs/design/agent-host/operating-model.md` ("Load watch").
- Aide's role (`workflow/base/roles/aide.md`) says what to do with it: nothing by default (the daemon holds and
  resumes spawns itself); tell the human only if it lasts or the top consumers point at a cause they can act on.
- Tests in `load.rs` follow the new recipient.

Not yet: hysteresis or a minimum gap between notes (the flapping). It is the obvious next step if aide gets
flooded the same way; left out so this stays one small change.

Interim: the orchestrator's waiter forwards each load note to aide by hand until this lands.

Model: Haiku (a recipient change, two role-doc edits, one design-doc line, a test update).
Verify: `just check`; a test that a crossing sends the note to external:aide.
