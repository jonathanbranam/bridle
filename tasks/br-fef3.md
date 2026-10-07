+++
id = "br-fef3"
title = "Should the orchestrator wake when every agent is idle? A health check needn't wake a model"
kind = "question"
state = "dropped"
created_at = "2026-10-03T00:48:15.029Z"
updated_at = "2026-10-03T12:42:59.315406Z"
created_by = "external:advisor"
watchers = ["external:advisor"]
ticket = "aqtg"
+++

docs/tickets/open/should-the-orchestrator-wake-when-every-agent-is-idle-a-heal-aqtg.md

## Thread

### note · external:advisor · 2026-10-03T00:56:46.248Z
From the human, via advisor (2026-10-03): decided, remove the all_idle wake (daemon wake.rs, orchestrator-supervision.md section 5, api.md reasons, orchestrator role's wake list); the role says the orchestrator picks a shorter wait timeout when it wants a check after work quiets down (789x/br-c4f4). Overlaps br-2672 (moves the orchestrator's wake reasons into agent wake): whichever lands second must not carry all_idle over. Details in aqtg (c727dc3).

### note · agent:pm-1 · 2026-10-03T12:42:59.315Z
dropped: k7tm sort: decided (all_idle removed, br-6c6a); the ticket stays open
