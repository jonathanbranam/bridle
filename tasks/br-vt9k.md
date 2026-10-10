+++
id = "br-vt9k"
title = "Incident: the benchmark sampler's script and ~2 h of data were lost when landing br-4vmc removed the worker that was also running the sampler"
kind = "incident"
state = "pending"
created_at = "2026-10-10T13:46:10.059Z"
updated_at = "2026-10-10T13:46:44.735157Z"
created_by = "external:aide"
watchers = ["external:aide"]
priority = "high"
priority_at = "2026-10-10T13:46:10.060030Z"
+++

The human, 2026-10-10 ~9:45 AM ET, verbatim (to the aide): "wow - we lost everything from the benchmark sampler! OMG that is a huge miss; traack in the incident log."

What happened (2026-10-10, times UTC):
- 08:26Z: w4vmc (the worker on br-4vmc, idle with a parked branch) was given br-v6kr phase 1 and started the baseline sampler: scripts/baseline-sample.sh, writing docs/context/baseline-2026-10.csv on branch bridle/w4vmc-v6kr. Neither the script nor the CSV was committed.
- 08:46Z: the daemon self-upgraded; the sampler died with it and w4vmc restarted it (~1.5 min gap).
- 11:21Z: after the human's stop and then "keep the benchmark going a bit", w4vmc restarted it for ~2 h more (89 min of data collected over ~3 h by then).
- ~11:25Z: landing br-4vmc (f49d8ebe) removed worker w4vmc and its worktree, and with it the uncommitted script and all the sample data.

Causes:
1. Landing removes the task's worker without checking whether it holds another task's work (fixed: ticket 37r9, br-37r9 integrated).
2. One worker shared between two tasks (br-4vmc, br-v6kr) to fit the worker cap.
3. The brief let the sampler run before its script was committed, and kept the data uncommitted until the end.
4. The 12 h passive duration (no scenarios) came from reading the human's "overnight" (when to run it) as how long to run it; nobody caught it (PdM, br-v6kr thread).

Follow-up: the human's redesign of the benchmark (sent to the PdM): script committed and merged first, results committed and kept permanently, a designer pass, the human signs off before any run.

## Thread

### note · external:aide · 2026-10-10T13:46:10.060Z
priority: normal -> high

### note · external:orchestrator · 2026-10-10T13:46:44.735Z
orchestrator: logged in docs/context/incidents.md (2026-10-10 11:25 entry), with these four causes and the human's words; follow-ups 37r9 (br-37r9) and the PdM's benchmark redesign. Cause 2 includes my part: the fallback 'hand br-v6kr to w4vmc' and then 'br-4vmc can land' without flagging that w4vmc carried the sampler.
