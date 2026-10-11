+++
id = "br-rhba"
title = "Mail between daemons, slice 5: every orchestrator wake is a message, sent home; one waiter per principal (3haz)"
kind = "feature"
state = "integrated"
created_at = "2026-10-10T02:59:02.102Z"
updated_at = "2026-10-11T01:43:07.245327Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "external:advisor",
]
branch = "bridle/wrhba"
commit = "bcd1b7bf40fb17b1a14d883905cc4e2eb03440d0"
summary = "Slice 5 of 3haz: a wake the project daemon raises for the orchestrator (incident, exit/crash/stall, CI failure, budget hold, usage, context) is also queued as a MessageKind::System message (new, additive; old builds read it as note) in the outbox for the home of every visiting external:orchestrator@<machine> token, from system@<machine>, body '[reason] text (as of time)'; /v1/forward dedups, so it arrives once and wakes the single waiter on the home daemon. Old local wake path untouched (per-project waiters keep working); the home daemon itself sends nothing. Files: types.rs, wake.rs, store.rs, lib.rs, outbox_test.rs, orchestrator role, orchestrator-supervision.md, CHANGELOG. Caveat: only remote projects forward; message/question/daemon_stopping are not forwarded. Not done: reading k8jn read-on-delivery fix, 'peer unreachable' message (P6)."
parent = "br-3haz"
+++

Ticket: docs/tickets/open/daemons-deliver-mail-to-each-other-across-machines-store-and-3haz.md, sections 'Decided: one waiter per principal, on its home daemon' and 'Caveats of every orchestrator wake as a message' (all five accepted by the human). Also docs/design/agent-host/orchestrator-supervision.md section 5, crates/bridle-daemon wake.rs. Goal: every reason a daemon wakes the orchestrator today (red CI, agent death, stall, context warning, task wakes) is sent as a message of a system kind and, via br-n7cg forwarding, reaches the orchestrator's home daemon; the orchestrator then needs ONE waiter on its home daemon instead of one per project. Update the orchestrator role (workflow/base/roles/orchestrator.md: one waiter, home daemon) and the supervision docs. Wire change (message kind) goes in crates/bridle-api/src/types.rs with all clients; keep the old wake path working until the message path is verified (serde defaults). Blocked by br-n7cg. Acceptance: just check passes; tests: each wake reason arrives as a message on the orchestrator's home daemon and wakes its single waiter; a remote project's wake is forwarded home once (dedup). Migration: new message kind is additive; role text reaches projects via workflow sync; existing orchestrators with per-project waiters keep working. State this. Model: Sonnet. Out of scope: status lines (br-cufw), CI watch fix (br-ysmu).

## Thread

### note · external:advisor/product-manager · 2026-10-10T02:59:22.432Z
advisor (product-manager): readied. The human, 2026-10-09 ~10:55 PM ET: "if the machine work finishes up, let's prioritize work that makes sending and receiving messages work better and more reliable, reducing waiter counts like the orc has 5 waiters; I think the scheduled message work is also an important epic to finish up soon". New epic messaging (theme agents-and-cli), ranked right after machine-setup. Order: br-2msq, br-n7cg, br-rhba, br-ysmu, br-cufw.

### note · agent:wrhba · 2026-10-11T01:30:11.732Z
done: wakes forwarded home as system messages, role/docs/CHANGELOG updated; just check exit 0, 1480 tests; ed4535c5 (merge of main after the check was a ticket-doc move only)

### note · agent:manager-2 · 2026-10-11T01:42:52.510Z
integrated: bcd1b7bf40fb17b1a14d883905cc4e2eb03440d0 (branch bridle/wrhba)

### note · agent:manager-2 · 2026-10-11T01:43:07.245Z
cleanup: removed agent wrhba, branch bridle/wrhba
