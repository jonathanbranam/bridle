+++
id = "br-4d5a"
title = "The orchestrator stays running: a watcher for the watchman (fx7x)"
kind = "feature"
state = "dropped"
created_at = "2026-09-29T12:42:32.432Z"
updated_at = "2026-10-06T00:48:44.903138Z"
created_by = "external:advisor"
watchers = ["external:advisor"]
ticket = "fx7x"
+++

Ticket: docs/questions/open/the-orchestrator-stays-running-fx7x.md

## Thread

### note · system · 2026-10-05T15:24:32.040Z
open 4h, never planned: back to pending. Ready it again once someone will plan it.

### note · external:aide · 2026-10-06T00:48:12.352Z
From the human, via aide (2026-10-05 ~9:20 PM ET), on br-4d5a (fx7x, the orchestrator stays running): "We are closing br-4d5a without implementing it because it has already been built separate. I agree". Orchestrator supervision slices 1a, 1b, 2, 3 (br-a424, br-e949, br-65b8, br-4573) built it. Please close br-4d5a and resolve ticket fx7x (docs/tickets/open/the-orchestrator-stays-running-fx7x.md) with a Resolution naming docs/design/agent-host/orchestrator-supervision.md. Remaining extras (orchestrator pause, percentage thresholds, etc.) stay as that design's Planned items, not this task.

### note · external:orchestrator · 2026-10-06T00:48:44.903Z
dropped: The human closed it unimplemented (2026-10-05 ~9:20 PM ET): already built by orchestrator supervision slices br-a424, br-e949, br-65b8 and br-4573 (docs/design/agent-host/orchestrator-supervision.md). Ticket fx7x resolved in 0ce4269a.
