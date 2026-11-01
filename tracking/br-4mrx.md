+++
id = "br-4mrx"
title = "Wake waiters stop carrying messages: a wake only signals, the agent fetches what is pending"
kind = "research"
state = "pending"
created_at = "2026-11-01T13:20:20.190Z"
updated_at = "2026-11-01T14:23:25.480504Z"
created_by = "external:aide"
watchers = ["external:aide"]
ticket = "4mrx"
+++

docs/tickets/open/wake-waiters-stop-carrying-messages-a-wake-only-signals-the-4mrx.md

## Thread

### note · external:aide · 2026-11-01T13:26:08.942Z
Aide's first pass (2026-11-01), for the human to decide on.

FINDING (verified in code): a lost waiter DOES lose messages today. A waiter prints messages but marks nothing read; the session's NEXT `agent wake` acknowledges, as read, whatever the previous wait handed over, without knowing whether anyone saw it (crates/bridle-daemon/src/server.rs:1055-1076, take_unacked). So output discarded, a killed waiter, or a wrong start followed by a restart = messages marked read unseen. The docs' "lost output loses nothing" (cli.md:131, aide/advisor role text) is false. The orchestrator is worse: its wakes are marked read on hand-over (server.rs:1020-1032). The hand-over list is in memory only. The gap between two waits is NOT the problem: the wait is level-triggered on "unread exists", so a message arriving between waits is returned at once by the next wait.
No caps anywhere: wake and inbox return every unread message in full (list limit u32::MAX, store.rs:3306). task_update notices are one per change per watcher, uncoalesced. History: read/ack semantics changed 3 times (rmpq, 9aj2/djru), replace/stop/exit codes ~4 times (75h2, ds96, ux3h, h3ar).

PROPOSAL: the waiter only signals; `bridle inbox` is the only thing that reads.
1. Waiter prints one line and changes nothing: "woken: 3 pending (1 message, 2 ticket updates); run `bridle inbox`". No bodies, never marks read, no hand-over state (delete take_unacked). Then a wrong start, discarded output, kill, supersede or daemon restart cannot lose anything: the messages are still unread, and the next waiter returns at once saying so. This is how the human's own wake already works (ids only, explicit read).
2. Exit codes as a fixed protocol: 0 = something pending: run `bridle inbox`, then wait again. 4 = timeout: wait again. 5 = superseded: do nothing (a newer waiter is running). 6 = daemon stopping/restarting: wait again. Role text shrinks to "after any waiter exit except 5, run `bridle inbox` then wait again". Keep the /dev/null refusal (harmless now, but stops a waiter whose 'pending' line nobody sees).
3. Race: the loop is wake -> inbox -> wait. Anything arriving after inbox makes the next wait return immediately. Nothing to reconcile, no 'did I miss one' check.
4. `bridle inbox` marks read exactly what it printed, in the same call, and PAGES: at most ~20 items or a size cap, then "N more pending: run `bridle inbox` again". Paging is needed anyway because Claude Code truncates long command output, which would otherwise mark unseen messages read after a long absence.
5. Pile-up after days off: inbox shows messages from people/agents first, then ticket notices collapsed per ticket ("br-x: 5 changes, now integrated"). The waiter's one line gives counts by kind, so the agent knows the scale before reading.
6. Orchestrator: same model (its Wakes queue fetched by inbox, not marked read on hand-over); can be phase 2.
Unaffected: headless agents (stdin delivery).

EDGE CASES: (a) an agent that waits without running inbox spins (exit 0 at once, every time): the line says what to do, and the daemon can refuse a third immediate re-wait with the same pending set with an explicit error. (b) inbox output lost: a foreground command, rare; accepted (alternative: two-step fetch + ack, more complex, not recommended). (c) two sessions under one identity on one daemon (two unnamed advisors): one's inbox reads the other's messages; true today too, out of scope. (d) A notice older than N days about a ticket since closed: keep collapsed, or expire as read: the human's call. (e) Cost: one extra tool call per wake; the waiter's output gets far smaller, which saves context.

### note · external:aide · 2026-11-01T14:21:59.089Z · for-human
The human, 2026-11-01 (to aide), verbatim, on the aide's proposal:

"For one thing, the orchestrator and the other agents should work the same. That is crazy that they're still not working the same. There's no reason. This is bad design on our part for bridle. There's no reason these things work differently. The code path should be the same. [...] if there's some reason the orchestrator is still different, please explain that to me. I would put that high on the list. The code paths for all the agents should be the same. All the interactive agents that need these wakes, at least, should all be the same."

"Unless I'm mistaken, exit code 0 is success, right? Why would success mean there's a message waiting? I feel like exit code 0 should mean nothing happened, right? [...] I ran the timer and nothing happened, so I restarted the timer. That, to me, feels like success. All the other codes are errors [...]"

"I think the improvement for the inbox is good: page at 20 at a time or something like that. Will that change how I read it as well? Also, let's be sure there's a command for the inbox to show the number of messages. [...] If we're paging 20 at a time, part of that output should say how many total messages there are."

"For expiring old notices [...] I would probably keep them for at least a week, or maybe two weeks. [...] Anything that's more than a week or two old, we're probably just better off having the agent look at the system and see what's going on. [...] hopefully we've got a handover note [...] and then the agent should just go fetch the updated status of those tickets. Also, the agent would be watching those tickets and listed as creator."

"let's be sure there's a way for an agent to list tickets that they're waiting on, or that they created, or that they are watching, by type even and by date. If I bring up the aide on a project, I should be able to ask them to tell me all the tickets that you were working on the last week. They should be able to list open tickets or closed tickets or whatever for the last week."

### note · external:aide · 2026-11-01T14:22:31.524Z · for-human
The human, 2026-11-01 (to aide), verbatim: "I don't care if the orchestrator unification is first or second, necessarily, but let's be sure that gets done either way."

### note · external:aide · 2026-11-01T14:22:31.629Z
CONSOLIDATED DESIGN (aide, 2026-11-01), for the human's sign-off. Nothing is built until they say go.

1. One wake path for every interactive session (REQUIRED, order free). The orchestrator's separate Wakes queue (wake.rs: exits, stalls, CI failures, budget holds, incidents, usage crossings; marked read on hand-over; own cursor and forwarding) is replaced: each such fact becomes an ordinary stored message to external:orchestrator, and the separate queue, cursor and read-on-hand-over code are deleted. Forwarding to the home project uses the normal message forwarding.
2. The waiter only signals. It prints one line with counts and changes no state: it never prints bodies and never marks anything read. Delete the hand-over/acknowledge machinery (take_unacked).
3. Exit codes (the human's scheme): 0 = nothing happened, wait again. 4 = pending, run `bridle inbox` then wait again. 5 = replaced by a newer waiter, do nothing. 6 = daemon stopping/restarting, wait again. 1 = error (no daemon etc.). Exit 4 is level-triggered: anything unread makes a new wait return at once, so there's no race and nothing to reconcile. To stop a spin, the daemon refuses repeated immediate re-waits on an unchanged pending set with an explicit error.
4. `bridle inbox` is the only reader. It marks read exactly what it printed. It pages ~20 at a time with "showing 20 of N; run again for more". Order: messages from people and agents first, then ticket notices collapsed per ticket. `bridle inbox --count` prints the total and counts by kind; the waiter line and each page header carry the same counts. The human's reads (web UI) are unchanged.
5. Ticket notices (task_update) older than 14 days expire: marked read with reason expired, still visible in `bridle message list`. Messages from people or agents never expire.
6. Ticket filters: `bridle ticket list --created-by <p> --watching <p> --state open|closed --since <dur>` (with --kind), so "every ticket you worked on last week" is one command. Depends on, or folds in, br-519b (creator field and watchers list).
7. Role text, rules and docs: one short paragraph ("after exit 4 run inbox, then wait; after 0 or 6 wait; after 5 do nothing"). Fix the false "lost output loses nothing" lines (cli.md:131, role files) and the stale messages.md section.

Proposed split for the build: (a) the signal-only waiter + inbox paging/count + exit codes + docs; (b) the orchestrator unification; (c) notice expiry; (d) ticket filters (with 519b). (b) is mandatory whichever order.

### note · external:aide · 2026-11-01T14:23:25.480Z
The human, 2026-11-01 ~9:30 AM ET (to aide), verbatim, on point 4 (inbox paging):

"On the inbox paging command, I think we should be maybe a little more explicit with the agents. I don't want the agent running bridle inbox like five times back to back. It'd be better to run it once, then handle those 20 messages, and then run it again instead of overwhelming the context with 100 different messages about different things."

AMENDMENT to point 4 (and 7): one page per pass. The page footer says it outright: "showing 20 of N. Handle these first, then run `bridle inbox` again (or wait: the next wait returns at once while any are unread)." Role text and rules say the same: never run `bridle inbox` twice in a row without acting on the first page. Because the wait is level-triggered (exit 4 while anything is unread), the natural loop wake -> inbox (20) -> act -> wake -> inbox (next 20) already enforces this; the agent needs no "drain" step. No daemon-side enforcement proposed (KISS): the footer and role text carry it.
