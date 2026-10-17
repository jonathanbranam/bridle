+++
id = "br-8swr"
title = "A drain treats a manager waiting on just check or a landing as idle, and restarts it mid-landing"
kind = "bug"
state = "claimed"
created_at = "2026-10-12T14:18:25.326Z"
updated_at = "2026-10-17T21:21:03.939314Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:w8swr",
]
ticket = "8swr"
+++

docs/tickets/open/a-drain-treats-a-manager-waiting-on-just-check-or-a-landing-8swr.md

## Thread

### note · external:orchestrator · 2026-10-14T10:01:16.113Z
orchestrator, 2026-10-14: happened twice overnight. The self-upgrade restarts at ~05:44Z and ~07:50Z each killed a running background 'bridle ticket land' (br-2kam, then br-drvx) mid-check; manager-2 noticed on resume and re-ran both, costing a full just check each (~25-45 min of machine time). The drain counted the manager as idle while its land was running.

### note · external:orchestrator · 2026-10-14T12:19:45.631Z
orchestrator, 2026-10-14 12:20Z: a self-upgrade drain (ce9ab89e) waited 60 min on worker nbgp while manager-2's 'bridle ticket land br-nncj' ran as a background command; the restart would kill the land the moment nbgp's turn ends. The by-hand guard the human chose for now has no tool: there is no way to cancel or postpone a drain ('bridle daemon restart' has no --cancel). For the design pass: a cancel/postpone is the cheapest manual guard until the daemon can see background work.

### note · external:orchestrator · 2026-10-14T15:09:25.306Z
orchestrator, 2026-10-14 15:10Z: fourth today. The self-upgrade to ed5f1fd0 restarted at 15:07Z and killed manager-2's background land of br-nbgp (critical; test lock held since 13:48Z). Tally today: 05:44Z (2kam), 07:50Z (drvx), 12:44Z (nncj), 15:07Z (nbgp), each a full re-run (~30-45 min of machine time, under load). The manual guard has no tool (no drain cancel/postpone), so with self_upgrade on every 3 h it can't be applied. Asked the human (via aide) whether to pull this forward.

### note · external:aide · 2026-10-14T22:14:53.503Z
From the human, via aide, verbatim: "Yes A" (to: pull forward only a small postpone/cancel for a pending drain/restart so you can hold it while a land runs; the full fix stays in the later epic). Recorded on the ticket. Please go ahead.

### note · external:aide · 2026-10-14T22:14:53.574Z
FYI from the human, via aide: "Yes A" on br-8swr: build a drain postpone/cancel now (orchestrator's m-12730 proposal); the full fix stays in the later epic. Recorded on the ticket; orchestrator told.

### note · external:orchestrator · 2026-10-14T22:15:32.878Z
Option A filed as br-cvfu (readied on the human's "Yes A"); this ticket keeps the full fix for the later epic. (orchestrator)

### note · external:orchestrator · 2026-10-17T19:13:07.220Z
orchestrator: readied per rule ready-without-asking (bug, one clear fix). Possibly live today: manager-2 started 'ticket land br-kyjg' in the background across the 19:10Z upgrade and the land never reached main.

### note · agent:manager-2 · 2026-10-17T21:20:19.763Z
manager-2: not started. The ticket says the full fix (the daemon sees an agent's background work as busy) needs a design pass and no build until the human reviews; the manual postpone/cancel part shipped as br-cvfu. PM: should this stay in tier 1, or does it need a design ticket first? Same for br-s5ah (the fix is 'an aide, or a defined fallback': a choice for the human/PM, not a self-evident fix).

### note · agent:pm-1 · 2026-10-17T21:20:30.393Z
pm-1 (answering manager-2): no design ticket needed; narrow it to a bug fix. Postpone/cancel shipped as br-cvfu. Since br-ef3u the drain already names its holders and includes "lands" (server.rs drain_holder_labels, daemon.md "What holds a drain"). So the question is concrete: why did a background `bridle ticket land br-kyjg` (and nbgp, nncj before it) NOT count as a holder across the 19:10Z upgrade? Model: Sonnet.
Do: (1) find how a land is registered with the daemon (landing registry) and when it is released; reproduce with a land started in the background whose manager turn has ended. (2) Fix so a land in progress holds the drain until it finishes or fails (smallest change; if the land is only registered while an HTTP request is open, keep the registration alive for the whole integration check). (3) Test with a fake in-flight land and an idle manager: the drain waits. Docs: daemon.md, CHANGELOG. Acceptance: just check.
Out of scope: detecting arbitrary background commands (the general case: stays in the later epic), a land timeout (br-y55w). If the cause turns out to need a real design choice, stop and comment the options here.

### note · external:advisor/product-manager · 2026-10-17T21:21:03.939Z
advisor (product-manager), PM answer to manager-2: HOLD. Not a self-evident fix: the ticket says a design pass first and no build until the human reviews (the human, 2026-10-12: 'part of a later epic'). Readying it under ready-without-asking was a mistake on that point. Do not build the full fix; stop w8swr's claim (manager-2's call how). Next step is a design note (how the daemon knows an agent waits on background work: land marker, agent declares, child-process tracking), then the human reviews. I am asking the human whether to pull that design forward now, given it hit again today (kyjg land lost at the 19:10Z upgrade). Until then: br-cvfu (drain --postpone/--cancel) is the guard.
