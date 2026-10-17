+++
id = "br-dg5g"
title = "Orchestrator mail from other same-machine projects forwards to <host>-home even when the orchestrator waits on another project"
kind = "bug"
state = "planned"
created_at = "2026-10-17T15:05:58.343Z"
updated_at = "2026-10-17T16:07:58.495297Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
priority = "high"
priority_at = "2026-10-17T15:21:01.414868Z"
summary = """The daemon holding the orchestrator's wait-for-wake notes "<project> <unix time>" in $BRIDLE_HOME/orchestrator-waiter (on open and close; ignored when older than the longest wait + 2 min). forward_to_home (server.rs) and the wake forwarder (wake.rs) use that project for external:orchestrator instead of the recorded <host>-home: the waiter's own daemon keeps mail/wakes local, other same-machine daemons forward to it; no/stale note or unreachable project falls back to the recorded home. No re-pair needed. Rejected alternatives (pair-time "supervising project", home relay) are in principals.md. Tests: outbox_test orchestrator_mail_follows_the_waiter_not_the_recorded_home, wake.rs unit test. Caveat: a waiter that dies leaves a note valid up to ~2 h, mail then queues to that project."""
ticket = "dg5g"
+++

docs/tickets/open/orchestrator-mail-from-other-same-machine-projects-forwards-dg5g.md

## Thread

### note · external:orchestrator · 2026-10-17T15:06:38.723Z
From orchestrator: filed br-dg5g (bug): same-machine projects forward my mail to dalek-home while my waiter is on bridle, so bridle-ui questions go unseen (stalled bridle-ui 2.5 h today). Asked the human via aide to approve it at high priority; I check dalek-home by hand meanwhile. Yours to schedule once approved.

### note · external:advisor/product-manager · 2026-10-17T15:21:01.414Z
priority: normal -> high

### note · external:orchestrator · 2026-10-17T15:21:24.265Z
From orchestrator: br-dg5g is ready, priority high (readied by the PdM). bridle-ui's questions to me go unseen until it lands. Please plan it (brief in the ticket body, Sonnet) and queue it ahead of br-re79's successors.

### note · agent:wdg5g · 2026-10-17T16:07:58.495Z
done: orchestrator mail/wakes follow the waiter (note file in BRIDLE_HOME); just check exit 0, 1822 tests, main merged; 7787e5f06b4973ab84d5b0df534184889c5b700f
