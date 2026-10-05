+++
id = "br-ft3b"
title = "Per-role handover instructions in the workflow, with project overrides"
kind = "feature"
state = "pending"
created_at = "2026-10-05T10:25:03.907Z"
updated_at = "2026-10-05T10:25:03.907788Z"
created_by = "external:orchestrator@nuc"
watchers = ["external:orchestrator@nuc"]
+++

submitted by external:orchestrator@nuc

The human, 2026-10-05: 'we should be able to have custom handover instructions for a role. That should be part of the bridle workflow system, and there should be project-level overrides allowed there. Similar to whatever else we've built, bridle should come with a standard description of how to do a handover and a prompt for how to do a handover. We should be able to have that be different for each role, and the project should be able to use the same system we have already to layer project-specific commands on top of that.'

Example need: the notes project's advisor keeps the human's daily plan. Its handover must state explicitly 'the wake-up time tomorrow' (and the day's plan state), not leave it to the model. 'if I trust the notes agent to hand over in the night, I have some really specific things I want to emphasize.'

Ask: a standard handover text (how to hand over, and the prompt the daemon sends at a handover) in workflow/base, a per-role section or file (orchestrator, advisor, aide, manager, worker) that can replace or extend it, and project layering (.bridle/roles/<role>.md or a handover part) the same way role docs layer today. Used by context-driven handovers (gq9r), the nightly restart (companion ticket) and 'session restart --handover'.

## Thread

### note · external:orchestrator@nuc · 2026-10-05T10:25:03.907Z
submitted by external:orchestrator@nuc
