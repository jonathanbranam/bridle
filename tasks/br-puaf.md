+++
id = "br-puaf"
title = "Usage readings go stale while agents work, and a stale reading at low usage holds the workforce"
kind = "bug"
state = "open"
created_at = "2026-10-04T15:18:54.499Z"
updated_at = "2026-10-04T15:28:29.205175Z"
created_by = "external:aide"
watchers = ["external:aide"]
priority = "high"
+++

original id: puaf
docs/tickets/open/usage-readings-go-stale-while-agents-work-and-a-stale-readin-puaf.md

## Thread

### note · external:orchestrator · 2026-10-04T15:19:27.972Z
priority: normal -> high

### note · external:orchestrator · 2026-10-04T15:19:28.007Z
The human, 2026-10-04 via aide: 'This is a noisy alert that shouldn't be happening ... If we're at 20% or 30% usage, I don't care if the [reading] is probably an hour old. If we're at 80% usage, then the sampling makes sense.' Readied by the orchestrator (bug fix). Default config changes only; ~/.bridle/config.toml is the human's.

### note · external:aide · 2026-10-04T15:28:29.205Z
The human, 2026-10-04, via aide, on the stopgap of raising max_staleness to 45m in ~/.bridle/config.toml: "Yes, I'm making a stop gap staleness for 45 minutes. I can't make that change right now, though, so we can just deal with it until later. [...] Yes, I approve that." They'll make the edit themselves later. They also want to keep track of what the probe costs (aide's answer: get_usage makes no model call, per spike 01 S8).
