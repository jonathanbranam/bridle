+++
id = "br-q7my"
title = "Home project refusals: refuse an orchestrator and orchestrator messages on non-home projects (7ytg slice 2)"
kind = "feature"
state = "planned"
created_at = "2026-10-18T13:32:43.398Z"
updated_at = "2026-10-20T22:42:23.576590Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
ticket = "q7my"
+++

docs/tickets/open/home-project-refusals-refuse-an-orchestrator-and-orchestrato-q7my.md

## Thread

### note · agent:pm-1 · 2026-10-18T13:32:50.978Z
pm-1, from the PdM's brief on br-7ytg (slice 3 there): (1) this ticket lands only after the per-machine check (the PdM's slice 2: the aide confirms dalek, vesta and remus each run, wait and are configured in their home; dalek needs br-e7s5 and a relaunch first; every human step is a --for-human to-do). That step is not code; the aide/orchestrator marks it done by commenting here before a worker starts. (2) Ship so a daemon self-upgrading onto it with a misconfigured orchestrator is not stranded: refuse NEW launches, never kill a running orchestrator. (3) Allow the orchestrator agent in the home project (br-z485's design assumes it). (4) Items 1, 3, 4, A, C, D as listed; B as (a).

### note · external:aide · 2026-10-20T22:42:01.831Z
From the human, 2026-10-20 ~6:45 PM ET (to aide), verbatim: "yes ready br-q7my". Readied by aide.

### note · external:orchestrator · 2026-10-20T22:42:12.857Z
Approval, human via aide (m-0372, 2026-10-20 ~6:45 PM ET): "yes ready br-q7my".

### note · external:orchestrator · 2026-10-20T22:42:19.214Z
orchestrator: likely touches the same home-project code as br-b2g8 (in progress, wb2g8); queue it after b2g8 lands, not alongside.

### note · agent:pm-1 · 2026-10-20T22:42:23.576Z
pm-1: added blocked-by br-b2g8 (same home-project code); q7my starts after b2g8 lands.
