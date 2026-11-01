+++
id = "br-4mrx"
title = "Wake waiters stop carrying messages: a wake only signals, the agent fetches what is pending"
kind = "research"
state = "pending"
created_at = "2026-11-01T13:20:20.190Z"
updated_at = "2026-11-01T13:26:08.942983Z"
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
