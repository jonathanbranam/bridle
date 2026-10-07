+++
id = "br-qect"
title = "Postmortem for incident br-y455: bridle's daemon couldn't restart or self-upgrade for ~23 h (stuck spawning flag)"
kind = "research"
state = "planned"
created_at = "2026-10-06T22:13:54.531Z"
updated_at = "2026-10-07T00:52:11.596147Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
summary = "Postmortem written into ticket y455. Finding: manager.spawning() is a counter released by an RAII drop guard (supervisor.rs:803-814), so no spawn exit path (error, stop, cancel, panic) leaks it; stop-during-spawn is refuted as the trigger. The counter can only stay up while a spawn future hangs forever; candidate unbounded awaits listed (warm_target cp -cR, worktree::add, store/state locks), exact one unproven (daemon was kickstarted, evidence gone). Explained fully: no alert (self_upgrade_tick returns silently when spawning, before any upgrade.* event or the 3h give-up clock; CI watch ssh failure also hid it), restart error names nothing (bare counter; busy-agent names mask it), first refusal undatable (refusals are not logged). Recommendations: observable spawn registry, bound spawn awaits, alert on long-skipped upgrades, log restart refusals, forced restart for orchestrator, fix CI ssh key. Test crates/bridle-daemon/tests/spawning_flag_test.rs PASSES today (cannot reproduce a failing case; committed un-ignored as a guard) - the brief's required failing test is not delivered; reason in the ticket. just check exit 0, 1259 tests (last full: 1258)."
+++

original id: y455
Read-only investigation of the running system, then write the postmortem into ticket y455 (docs/tickets/open/incident-bridle-s-daemon-couldn-t-restart-or-self-upgrade-fo-y455.md): timeline, root cause with the exact code path that leaves manager.spawning() true, why ~23 h of refused upgrades raised no alert, and recommendations. Answer every question in the ticket's 'The postmortem must answer'. Linked: incident br-y455, bug br-btdn (the fix; do NOT make it here), postmortem q7mv (model for format).
Sources: daemon events (`bridle events`), the daemon log, crates/bridle-daemon (the restart wait at server.rs ~3653 and the agent manager's spawning flag/counter and every spawn exit path), br-btdn's body, worker a-897zk (task incident-2y3m) stop-requested 3 s after agent.spawned, 2026-10-06 03:20:31-36Z.
Reproduce: add a test that fails today because the flag stays true after the spawn path in question (e.g. stop during spawn, spawn error). Use the fake claude (crates/bridle-claude/tests/fake-claude.py), no real claude. Because `just check` must stay green on your branch, mark it `#[ignore = "reproduces br-btdn; remove the ignore with the fix"]` and say in the postmortem the exact command that runs it and shows it failing (`cargo nextest run --run-ignored only -p bridle-daemon <test name>`). Do not fix the bug.
Rules: do not restart, signal or change the running daemon; no live tests. ASCII only; times UTC (US Eastern in parentheses only where the human quoted one). Run `bridle ticket check`.
Acceptance: postmortem committed in ticket y455 on your branch, every question answered with evidence (log lines, event ids, code lines); the ignored failing test committed; `just check` passes.
Model: Sonnet. Out of scope: the fix (br-btdn), the alerting change itself (recommend only), any daemon restart.

## Thread

### note · agent:y455-postmortem · 2026-10-07T00:29:30.804Z
done: postmortem in ticket y455 (summary set). Stop-during-spawn is NOT the leak: counter is an RAII guard; test spawning_flag_test passes today, so no failing reproducer exists (brief's failing-test criterion not met, reason in ticket). Cause is a hung spawn future, exact await unproven. just check exit 0, 1259 tests. commit b2ef1c32

### note · agent:manager-2 · 2026-10-07T00:37:35.687Z
manager-2: land refused: main clone has uncommitted edits to the y455 ticket file (the incident log owner; not mine). Needs the owner to commit them, then I retry 'bridle task land br-qect --branch bridle/y455-postmortem --checked-commit b2ef1c32'. Expect a merge conflict in that file if the edits overlap the postmortem body; the worker (stopped) can be resumed to merge main.

### note · external:orchestrator · 2026-10-07T00:38:20.292Z
From orchestrator: committed the ticket edits in the main clone (1f854393: task links, read-marks, eight filed tickets). One edit remains: workflow/base/rules/no-kill-by-name.md, a stray duplicate 'aide' in its roles line, which I can't revert here. Retry the land; if that file still blocks it, tell me.

### note · agent:manager-2 · 2026-10-07T00:52:11.596Z
manager-2: land hit a merge conflict in the y455 ticket file (orchestrator edited it on main in 1f854393). Merge main into your branch keeping both sides (your postmortem body plus the incident-log edits on main), run just check, commit, update the task note with the new sha and quote the exit status and test count.
