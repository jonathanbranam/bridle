+++
id = "br-nncj"
title = "dalek's wakes for the orchestrator are forwarded to orchestrator@vesta through dotfiles-local"
kind = "bug"
state = "integrated"
created_at = "2026-10-14T10:01:15.595Z"
updated_at = "2026-10-14T13:40:49.543244Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:nncj",
]
branch = "bridle/nncj"
commit = "18780a468c9e63fbcfccd3aac80a0bd68840c66f"
summary = "The forwarding is Wakes::forward_home in crates/bridle-daemon/src/wake.rs (3haz slice 5, br-rhba): every non-message wake was mailed as a system message to the home of every visitor token external:orchestrator@<machine>. Vesta's orchestrator holds such a visitor token on dalek's daemon, so dalek's wakes were copied to it. Fix: forward_home returns early when the daemon has an active local external:orchestrator; the local wake path is untouched, so dalek's orchestrator still gets its own wakes. The remote-supervision case (no local orchestrator) still forwards. Test added in outbox_test.rs; docs (orchestrator-supervision.md) and CHANGELOG updated. Not touched: br-z5kz (system@ duplicate), a separate path."
ticket = "nncj"
+++

docs/tickets/open/dalek-s-wakes-for-the-orchestrator-are-forwarded-to-orchestr-nncj.md

## Thread

### note · external:advisor/product-manager · 2026-10-14T10:01:57.276Z
Readied by advisor (product-manager) as a critical reliability fix (the orchestrator's own context/handover wakes go to vesta, so dalek's orchestrator can be stopped without notice); the human to be told this morning. br-2uzx (low) held for the human.

### note · agent:pm-1 · 2026-10-14T10:02:06.517Z
Brief (pm-1). Model: Sonnet. Size: S-M. CRITICAL per the PdM: do first. The ticket body is the spec.

Goal: a wake about a machine's orchestrator reaches only that machine's orchestrator (its home daemon), never a copy to another machine's daemon. Today wakes for dalek's external:orchestrator are also forwarded (visitor wake forwarding) into the vesta dotfiles-local daemon and delivered to vesta's orchestrator.

Start: find where wakes (restart, agent exit, usage, context, handover, uptime) are forwarded to visiting/external principals: git grep -n "visitor" crates/bridle-daemon (wake forwarding, the "@dalek"/"@machine" principal handling), and the notes in br-z5kz and br-2ax5 (related). Decide which daemon owns each wake: a wake is delivered by the daemon it is about, to principals registered on it; a visitor (a principal homed on another machine) must not receive wakes that belong to its home daemon, unless it explicitly waits on this daemon. Check first that dalek's orchestrator still gets all its own wakes on its home daemon (it does tonight); do not break that.

Tests: a visitor principal homed elsewhere does not get this daemon's system wakes; the home principal still does; context/handover wakes reach the home orchestrator. Docs: the wake/visitor section of docs/design/agent-host (find with git grep -n "visitor" docs/design), CHANGELOG.md. No migration. Out of scope: br-z5kz (system@dalek duplicate), unless it is the same line of code. Acceptance: just check. Report on the thread what you found about where the forwarding lives.

### note · agent:nncj · 2026-10-14T10:37:38.608Z
done: wakes not forwarded to visiting orchestrators when a local one exists; just check exit 0 (1666 passed); 21a7c336

### note · agent:nncj · 2026-10-14T10:37:40.461Z
Found: forwarding lives in Wakes::forward_home (wake.rs), via visitor_homes_named('orchestrator'). Fixed by skipping it when a local external:orchestrator exists. just check exit 0, 1666 tests passed, commit 21a7c336.

### note · agent:manager-2 · 2026-10-14T10:37:41.056Z
built: 21a7c336 (branch bridle/nncj)

### note · agent:manager-2 · 2026-10-14T13:40:35.335Z
integrated: 18780a468c9e63fbcfccd3aac80a0bd68840c66f (branch bridle/nncj)

### note · agent:manager-2 · 2026-10-14T13:40:49.543Z
cleanup: removed agent nncj, branch bridle/nncj
