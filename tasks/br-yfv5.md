+++
id = "br-yfv5"
title = "One scheduler for timed actions: scheduled messages (hrcn), nightly session restarts (cbbn), maintenance windows for upgrades and reboots (3nyk)"
kind = "feature"
state = "integrated"
created_at = "2026-10-05T10:47:07.760Z"
updated_at = "2026-10-10T18:19:41.552736Z"
created_by = "external:orchestrator@nuc"
watchers = [
    "external:orchestrator@nuc",
    "external:advisor/product-manager",
]
branch = "bridle/dyfv5"
commit = "699154b8d622cb54e38a4aaf110de7d4eb7f6f16"
summary = "Design options written into ticket yfv5: recommend cbbn as a restart action on the schedules table (not a separate config), nullable action column, per project now (kuw2 changes noted, not designed), externals may schedule for themselves, 3nyk waits on kuw2/cy2v."
ticket = "yfv5"
+++

Role: designer (workflow/base/roles/designer.md). Ticket: docs/tickets/open/one-scheduler-for-timed-actions-scheduled-messages-hrcn-nigh-yfv5.md (read all of it, plus the built first slice: br-9xze, docs/design/agent-host/daemon.md Scheduled messages). Write a Design options section into the ticket: settle its For the PM / human to settle points, each with a recommendation; keep the scheduler per project for now and say what would change under one daemon per machine (kuw2) without designing for it (YAGNI). Cover how cbbn (nightly session restarts) and 3nyk (maintenance windows) would reuse the schedules table and loop, and note the known gap that schedules refuse external principals (orchestrator, advisor, aide) today, with a recommendation. Build nothing, edit only the ticket. Report on the task; the PdM sends it to the human. Model: Sonnet. Migration: none (design only).

## Thread

### note · external:orchestrator@nuc · 2026-10-05T10:47:07.761Z
submitted by external:orchestrator@nuc

### note · agent:pm-1 · 2026-10-05T10:47:20.780Z
Triage (pm-1): accept as a design ticket (minted, uncommitted; commit on main). No build task: it needs the human's refinement first, as the submitter says, so the task stays pending. My recommendation (also in the ticket): ship br-cbbn first as its own small restart_at config, then fold it into the scheduler; per-project messages and restarts before the machine-wide maintenance window (waits on cy2v). Related: hrcn, 3nyk, br-ft3b.

### note · external:orchestrator · 2026-10-08T14:28:58.547Z
split off br-9xze: Scheduled messages, first slice: an agent schedules a message to itself (one-time or recurring), bridle schedule add/list/rm

### note · external:orchestrator · 2026-10-08T14:29:05.481Z
The human approved the first slice (per-project scheduled messages an agent sets for itself, one-time or recurring), 2026-10-08, relayed verbatim by the meta-notes aide (m-7089); filed as br-9xze with the quote. cbbn (restarts) and 3nyk (maintenance windows) are not part of this approval. Same for hrcn.

### note · external:advisor/product-manager · 2026-10-09T11:04:38.816Z
watching the task

### note · external:advisor/product-manager · 2026-10-10T14:20:57.820Z
advisor/product-manager (PdM): readied for a DESIGNER pass (next designer task, after br-re57). The human, 2026-10-10 ~10:15 AM ET: 'the scheduled messages, is a real high priority for me. Assuming the setup for the WSL2 work is done, assuming the machine work is done, adding scheduled messages is the next priority. My only caveat to that is that we have some big architecture considerations about whether we have one daemon per machine or one per project. Not ready to tackle that, but that would entirely rewrite message delivery.' Designer: write '## Design options' into ticket yfv5; settle its 'For the PM / human to settle' points with a recommendation; keep the scheduler per project for now and say what would change under one daemon per machine (kuw2), without designing for it (YAGNI). br-g5y2 (planned) can be built meanwhile.

### note · agent:dyfv5 · 2026-10-10T18:19:27.461Z
done: Design options in ticket yfv5 (cbbn as restart action on schedules table; per project; externals self-schedule; 3nyk waits on kuw2); 8177aa3a

### note · agent:manager-2 · 2026-10-10T18:19:37.165Z
integrated: 699154b8d622cb54e38a4aaf110de7d4eb7f6f16 (branch bridle/dyfv5)

### note · agent:manager-2 · 2026-10-10T18:19:41.552Z
cleanup: removed agent dyfv5, branch bridle/dyfv5
