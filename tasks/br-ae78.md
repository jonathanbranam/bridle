+++
id = "br-ae78"
title = "Design: incident notices to agents, withdrawn when the incident ends (nc7r)"
kind = "research"
state = "integrated"
created_at = "2026-09-29T12:22:49.335Z"
updated_at = "2026-09-29T16:51:15.342043Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
branch = "bridle/incidents-design"
commit = "c086457"
summary = "Design in docs/design/agent-host/incidents.md: an incidents table (open/closed; update rewrites body) plus messages.incident_id; audience all or role:<name>, no cross-project or external push in v1; human/orchestrator/managers open, daemon opens none yet, budget holds stay as-is; delivery reuses system notes (when idle), close drops pending/held notices and sends resolved to the rest, agents starting during an incident get a notice at start, not in the system prompt; CLI/status/API/events specified. Build split deliberately left open per orchestrator."
+++

Ticket: docs/questions/open/*nc7r.md (read it fully, plus docs/design/agent-host/ on held/pending message delivery, the budget governor hold notices, docs/context/incidents.md, docs/design/coordination.md). This is a DESIGN task: write no code. Deliverable: a design section (new file docs/design/agent-host/incidents.md, or a section in the messages doc if smaller), and update the ticket with the decisions. Decide, KISS/YAGNI (rules in workflow/base/rules/): 1) the record: fields, states (open/updated/closed), storage in the SQLite store; 2) audience: all agents / by role / external principals; say whether cross-project is in v1 (recommend not: a per-project record the orchestrator opens on each daemon); 3) who may open/update/close: which principals, and whether the daemon opens any itself (budget hold, failed push, red main) now or later; 4) delivery: to running agents, to agents that start or resume while open (first message or system prompt), closing sends resolved only to those who saw it, an undelivered notice is dropped never delivered late; how much of the held/pending delivery machinery is reused; 5) surfaces: CLI (bridle incident open/update/close/list), status, API and events. End with a split into 2-3 build tasks, each finishable on one branch, with the files involved. Acceptance: just check passes (docs only); the design fits the existing store and delivery code (cite files). Model: Sonnet. Out of scope: implementing it; onboarding budget holds as incidents beyond noting whether to.

## Thread

### note · agent:manager-2 · 2026-09-29T12:31:54.070Z
Orchestrator: you may finish the design doc, but do NOT split into build tasks yet; a unified framing (incidents + human to-dos as one pattern) will follow from the orchestrator. Commit and report when the doc is done.

### note · agent:incidents-design · 2026-09-29T12:37:24.570Z
done: incident design in docs/design/agent-host/incidents.md, ticket nc7r updated; no build split per your note; 25dc8d8

### note · agent:manager-2 · 2026-09-29T16:50:18.687Z
main moved. Merge main into your branch (no other changes), run just check, commit, message me the sha.

### note · agent:incidents-design · 2026-09-29T16:51:04.637Z
main merged, just check passes (684 tests); ff7d7c2

### note · agent:manager-2 · 2026-09-29T16:51:08.236Z
integrated: c086457 (branch bridle/incidents-design)

### note · agent:manager-2 · 2026-09-29T16:51:15.342Z
cleanup: removed agent incidents-design, branch bridle/incidents-design
