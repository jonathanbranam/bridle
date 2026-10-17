+++
id = "br-q4vb"
title = "Wake forwarding sends one agent_exited event 7 times on a daemon restart"
kind = "bug"
state = "integrated"
created_at = "2026-10-17T03:16:03.081Z"
updated_at = "2026-10-17T22:48:55.091732Z"
created_by = "external:orchestrator@dalek"
watchers = [
    "external:orchestrator@dalek",
    "agent:wq4vb",
]
branch = "bridle/wq4vb"
commit = "898df76a8ddb1715d9e8ac4175fda4dca9c36df0"
summary = "Cause: forward_home was fed from the wake scan, whose cursor (orchestrator_wake_cursor) is saved only when a local waiter takes wakes. A daemon whose orchestrator is a visitor never saves it, so each restart rescanned all events since the old cursor and re-forwarded every agent exit (one more copy per restart). Fix in crates/bridle-daemon/src/wake.rs: separate orchestrator_forward_cursor, saved after each forward; events at or below it are not re-forwarded (absent key starts at the tail). Test a_restart_does_not_forward_old_events_again. CHANGELOG added; no behaviour docs. Not fixed: usage wakes (usage_fired is in memory) may re-forward after a restart."
ticket = "q4vb"
+++

docs/tickets/open/wake-forwarding-sends-one-agent-exited-event-7-times-on-a-da-q4vb.md

## Thread

### note · external:orchestrator · 2026-10-17T11:56:50.038Z
orchestrator: rights check after the 0.7.6 pair (j86p): a comment on a ticket I didn't submit.

### note · external:advisor/product-manager · 2026-10-17T15:26:10.807Z
Readied by advisor (product-manager) 2026-10-17 ~11:45 AM ET under the human's rule (br-mhjn): bugs, fixes and chores go through without asking. Checked still real on main.

### note · agent:wq4vb · 2026-10-17T21:51:05.164Z
built: c5f430d9 (branch bridle/wq4vb)

### note · agent:wq4vb · 2026-10-17T21:51:05.357Z
done: forwarding had no cursor of its own (shared the wake cursor, saved only on local-waiter delivery), so each restart re-forwarded old exits; now has orchestrator_forward_cursor. just check exit 0, 1830 tests, c5f430d9

### note · agent:manager-2 · 2026-10-17T22:27:51.499Z
integrated: 898df76a8ddb1715d9e8ac4175fda4dca9c36df0 (branch bridle/wq4vb)

### note · agent:manager-2 · 2026-10-17T22:28:08.363Z
cleanup: removed agent wq4vb, branch bridle/wq4vb

### note · external:orchestrator · 2026-10-17T22:48:55.091Z
orchestrator: seen again 2026-10-17 22:48:40Z after the d88f1321 restarts: one agent_exited for a-lykjs (daemon_shutdown) forwarded 12 times (m-16057..m-16068), plus exits for already-stopped bridle-ui agents (a-5anzd, a-9xugx, a-v0g1p) as if they had been running.
