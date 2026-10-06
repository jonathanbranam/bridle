+++
id = "br-qect"
title = "Postmortem for incident br-y455: bridle's daemon couldn't restart or self-upgrade for ~23 h (stuck spawning flag)"
kind = "research"
state = "planned"
created_at = "2026-10-06T22:13:54.531Z"
updated_at = "2026-10-06T22:14:05.713726Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

original id: y455
Read-only investigation of the running system, then write the postmortem into ticket y455 (docs/tickets/open/incident-bridle-s-daemon-couldn-t-restart-or-self-upgrade-fo-y455.md): timeline, root cause with the exact code path that leaves manager.spawning() true, why ~23 h of refused upgrades raised no alert, and recommendations. Answer every question in the ticket's 'The postmortem must answer'. Linked: incident br-y455, bug br-btdn (the fix; do NOT make it here), postmortem q7mv (model for format).
Sources: daemon events (`bridle events`), the daemon log, crates/bridle-daemon (the restart wait at server.rs ~3653 and the agent manager's spawning flag/counter and every spawn exit path), br-btdn's body, worker a-897zk (task incident-2y3m) stop-requested 3 s after agent.spawned, 2026-10-06 03:20:31-36Z.
Reproduce: add a test that fails today because the flag stays true after the spawn path in question (e.g. stop during spawn, spawn error). Use the fake claude (crates/bridle-claude/tests/fake-claude.py), no real claude. Because `just check` must stay green on your branch, mark it `#[ignore = "reproduces br-btdn; remove the ignore with the fix"]` and say in the postmortem the exact command that runs it and shows it failing (`cargo nextest run --run-ignored only -p bridle-daemon <test name>`). Do not fix the bug.
Rules: do not restart, signal or change the running daemon; no live tests. ASCII only; times UTC (US Eastern in parentheses only where the human quoted one). Run `bridle ticket check`.
Acceptance: postmortem committed in ticket y455 on your branch, every question answered with evidence (log lines, event ids, code lines); the ignored failing test committed; `just check` passes.
Model: Sonnet. Out of scope: the fix (br-btdn), the alerting change itself (recommend only), any daemon restart.
