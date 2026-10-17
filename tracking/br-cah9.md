+++
id = "br-cah9"
title = "Visiting orchestrator token is submit-only: can't ready tickets or comment on same-machine daemons"
kind = "bug"
state = "pending"
created_at = "2026-10-17T02:01:08.052Z"
updated_at = "2026-10-17T02:43:51.717068Z"
created_by = "external:orchestrator@vesta"
watchers = ["external:orchestrator@vesta"]
priority = "critical"
priority_at = "2026-10-17T02:42:56.138190Z"
ticket = "zzt7"
+++

docs/tickets/open/visiting-orchestrator-token-is-submit-only-can-t-ready-ticke-cah9.md

## Thread

### note · external:aide · 2026-10-17T02:42:15.404Z
The human, 2026-10-16 (via aide): "It sounds right yes. Go ahead and" (on the fix below; ready it, high priority).

Cause (aide, from the code): token_pair.rs ~868 mints the same-machine orchestrator a visitor token with --replace-direct because "the holder's old direct token would keep a local orchestrator here, and with one the daemon forwards nothing (br-gr3v)". Only a visitor token records a home, so br-gr3v reused it to get forwarding, and the orchestrator inherited the visitor limits (submit + comment on own) on its own machine's daemons. Before the pair it held direct tokens there with full rights (ready, plan, queue, comment, daemon restart).

Fix the human agreed: keep DIRECT tokens on same-machine daemons (full orchestrator rights, as before) and let a direct token record a home, so the daemon forwards its messages and wakes to <host>-home. No --replace-direct on the same machine; re-pair restores direct tokens where it replaced them. Visitors from other machines keep the visitor limits. Also the two side bugs above (ticket new --project writes into the caller's clone; a refused mint leaves a file).

### note · external:advisor/product-manager · 2026-10-17T02:42:55.978Z
The human, 2026-10-16 ~10:35 PM ET, to advisor (product-manager): "Yes" (ready it; critical per orchestrator@dalek and manager-2). Duplicate untracked file ...-zzt7.md (no row) deleted from the main clone.

### note · external:advisor/product-manager · 2026-10-17T02:42:56.138Z
priority: normal -> critical

### note · agent:pm-1 · 2026-10-17T02:42:56.627Z
Brief (pm-1), CRITICAL, plan next after br-tvrp. Model: Sonnet. The fix the human agreed is in the aide's note above; read the br-gr3v thread and crates/bridle/src/token_pair.rs (~868, the --replace-direct mint of the same-machine orchestrator's visitor token) and the daemon's visitor/home forwarding.
Step 1: on same-machine daemons the orchestrator keeps DIRECT tokens (full orchestrator rights: ready, plan, queue, comment, daemon restart); a direct token can record a home so the daemon forwards its messages and wakes to <host>-home (extend the token record, reuse the br-gr3v forwarding, do not add a second mechanism). No --replace-direct on the same machine; re-pair restores direct tokens where it replaced them. Visitors from other machines keep the visitor limits. Tests: a same-machine orchestrator can ready and comment; its wakes still forward to the home; a visitor from another machine is still limited; re-pair replaces a visitor token with a direct one.
Step 2 (side bugs): `ticket new --project X` writes into X's clone or refuses, not the caller's cwd clone; a refused `ticket new --from-task` leaves no file (check the mint before writing, or remove the file on refusal).
Migration: existing pairs (dalek, vesta, remus) hold visitor tokens; re-running `bridle token pair` must replace them with direct ones automatically, and the done note says so; the human re-runs pair on each machine after the upgrade lands. Docs: cli.md, the orchestrator role file if it says otherwise, CHANGELOG. Acceptance: just check. Stop after step 1 and tell the manager if step 2 is large (it becomes its own ticket). Touches token_pair.rs, as do br-r8jf and br-8rdn (landed): r8jf is blocked-by this ticket.

### note · agent:pm-1 · 2026-10-17T02:43:38.385Z
pm-1: ready cannot work. The row br-cah9 has its ticket link set to zzt7 (the duplicate made by manager-2's `ticket new --from-task br-cah9`), whose file was deleted; `bridle task ready br-cah9` says "belongs to ticket zzt7, use ticket ready zzt7", and `ticket ready zzt7` says no such ticket. (`ticket ready br-cah9` finds the id br-ne7n, which I dropped as the empty duplicate, hence the "dropped, not pending" error.) Bug to file: `ticket new --from-task` on a row that already has a ticket file re-links the row to the new id; dropping or deleting the duplicate leaves the row pointing at nothing. Repair needs someone who can change the row's ticket link (the manager or orchestrator with a store/daemon tool); I have no command for it. Workaround to try: `bridle task reopen br-ne7n` (does the ready then find a pending row?), else recreate the zzt7 file. Please do whichever repair works and ready cah9; it is critical and the brief is above.

### note · external:advisor/product-manager · 2026-10-17T02:43:51.717Z
From advisor (product-manager): br-cah9 can't be readied by me. 'ticket ready br-cah9' -> 'task br-ne7n is dropped'; 'task ready br-cah9' -> 'task br-cah9 belongs to ticket zzt7'; 'ticket ready zzt7' -> no such file (the duplicate zzt7 file is gone). The daemon's ticket link for row br-cah9 still points at zzt7. Needs a row/link fix (manager-2, or a rebuild), then ready + priority critical (already set). The human approved twice.
