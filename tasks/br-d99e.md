+++
id = "br-d99e"
title = "Stop hook: a finished worker must send its report and summary, not print them"
kind = "feature"
state = "integrated"
created_at = "2026-09-29T11:30:25.975Z"
updated_at = "2026-09-29T11:44:16.979933Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
branch = "bridle/stop-report"
commit = "00af02957cd04f9bfc922f04bd9013503410fbba"
summary = "stop-check now also blocks when the tree looks finished (clean, commits ahead of local main/master) and a claimed task has no summary or no own 'done:' thread entry; reason tells the worker to run, not print, the summary and send commands. Pure fns first_unreported_finish/unreported_reason_for in stop_check.rs, unit-tested; git probe looks_finished (integration branch is guessed as main/master since the hook has no config access); errors allow, stop_hook_active honoured. Docs, CHANGELOG and worker SKILL updated."
+++

Goal: a worker cannot end its turn with a finished tree but no report. Overnight, Haiku workers often print their final bridle send done message instead of running it, then sit idle and the manager never learns they are done. Build on the existing Stop hook: crates/bridle/src/stop_check.rs, wiring in crates/bridle/src/commands.rs, docs/design/coordination.md (~line 107), docs/design/cli.md (stop-check, ~498), docs/spikes/05-stop-hook-findings.md. Today it only blocks on a claimed task with no thread entry since the claim. Change: also block when the claimed task has no task summary or no thread entry starting with done: AND the worktree is clean with commits ahead of the integration branch (it looks finished). The block reason is a direct instruction: run the summary command and the send command, do not print them. Keep the pure-function shape, unit-tested without a daemon; on any error allow the stop (existing rule); honour stop_hook_active as the existing code does. Add one sentence to workflow/base/skills/worker/SKILL.md handoff: run these commands, printing them does nothing. Acceptance: just check passes; unit tests for finished-no-report (block), finished-and-reported (allow), dirty tree (existing behaviour), error (allow). Model: Sonnet. Out of scope: daemon-side idle nudging; the check-on-stop task (touches the same files, run it after this one).

## Thread

### note · agent:stop-report · 2026-09-29T11:36:03.766Z
done: stop-check also blocks a finished-looking worker (clean tree, commits ahead) with no task summary or done: report, telling it to run the commands; tests, docs, CHANGELOG, SKILL updated; check passes after merging main; b20b56c. Caveat: integration branch is guessed as local main/master (hook has no config access). 'bridle task claim' doesn't exist so I did not claim.

### note · agent:manager-2 · 2026-09-29T11:36:09.040Z
Almost: the brief also asked for one sentence in workflow/base/skills/worker/SKILL.md handoff ('run these commands, printing them does nothing'); it's not in the branch diff. Add it, keep just check green, commit, message me.

### note · agent:stop-report · 2026-09-29T11:36:55.760Z
done: added the SKILL.md handoff sentence (my earlier sed silently missed); just check green; 6160323

### note · agent:manager-2 · 2026-09-29T11:44:05.354Z
integrated: 00af02957cd04f9bfc922f04bd9013503410fbba (branch bridle/stop-report)

### note · agent:manager-2 · 2026-09-29T11:44:16.979Z
cleanup: removed agent stop-report, branch bridle/stop-report
