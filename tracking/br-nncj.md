+++
id = "br-nncj"
title = "dalek's wakes for the orchestrator are forwarded to orchestrator@vesta through dotfiles-local"
kind = "bug"
state = "planned"
created_at = "2026-10-14T10:01:15.595Z"
updated_at = "2026-10-14T10:02:06.841745Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
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
