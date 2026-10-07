---
id: ft3b
title: Per-role handover instructions in the workflow, with project overrides
kind: feature
opened: 2026-10-05
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: [br-ft3b]
---

## The ask

## The ask

The human, 2026-10-05: "we should be able to have custom handover instructions for a role. That should be part of the bridle workflow system, and there should be project-level overrides allowed there. Similar to whatever else we've built, bridle should come with a standard description of how to do a handover and a prompt for how to do a handover. We should be able to have that be different for each role, and the project should be able to use the same system we have already to layer project-specific commands on top of that."

Example need: the notes project's advisor keeps the human's daily plan. Its handover must state explicitly the wake-up time tomorrow and the day's plan state, not leave it to the model.

## What's wanted

- A standard handover text in workflow/base: how to hand over, and the prompt the daemon sends at a handover.
- A per-role section or file (orchestrator, advisor, aide, manager, worker) that can replace or extend it.
- Project layering (a handover part of .bridle/roles/<role>.md, or similar) the same way role docs layer today. Reuse the existing layering; don't invent a new mechanism.
- Used by context-driven handovers (gq9r), the nightly restart (cbbn) and `session restart --handover`.

Migration: none needed; project overrides are optional and absent by default. Say so in the design doc. Update docs/design/ for workflow layering. Verify: just check, plus a test that a project override replaces/extends the base text for one role.

Source: orchestrator@nuc, 2026-10-05. Sonnet.
