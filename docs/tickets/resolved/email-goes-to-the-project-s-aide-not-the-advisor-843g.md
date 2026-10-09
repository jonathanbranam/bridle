---
id: 843g
title: Email goes to the project's aide, not the advisor
kind: feature
opened: 2026-10-08
filed_by: external:aide
repos: [bridle]
changes: []
specs: []
needs: []
see: [rs7p]
tasks: [br-843g]
closed: 2026-10-09T23:11:07Z
---

## The ask

The human, 2026-10-07 ~8:45 PM ET, via aide, on hearing that mail to `bridle@dev.branam.us` goes to the unnamed advisor (else the orchestrator): "ah shucks. that needs to change to 'aide' now - let's get that change made; I'll still test once but aide is the new role for communication and chatting - one aide per project; orchestrator is moving to machine-wide probably"

Today (`docs/design/mail.md`, "Advisor liveness"; `crates/bridle-mail/src/bridge.rs` ~395, `local.rs` `advisor_running`): mail goes to `external:advisor` while `$BRIDLE_HOME/advisor-<project>.pid` names a live process, else to `external:orchestrator`. The advisor role carries a mail-only waiter (`bridle wait-for-wake --mail`) as an exception.

What changes (aide's reading; the human may correct):
- Mail for a project goes to `external:aide`, the project's aide (one per project).
- Liveness: the aide is a registered session (`bridle status` lists it under `sessions` with identity `aide` and a pid), so the bridge can ask the daemon instead of reading a pid file. Fallback while no aide is running: `external:orchestrator`, as today, until the orchestrator moves machine-wide; then revisit.
- The aide needs no special waiter: its `bridle agent wake` already ends on any message.
- The aide role (`workflow/base/roles/aide.md`) says what to do with mail: treat it as the human's words (it is `via email`), act or relay as for chat, and answer `bridle send external:mail "got it: <what was done>" --reply-to <id>` so the human gets the reply mail.
- Drop the advisor pid file and the advisor's `--mail` waiter if nothing else uses them; update `docs/design/mail.md`, the advisor role and rs7p's "Delivery to the advisor" to match.

## Resolution

Resolved 2026-10-09 by advisor (product-manager). Done: the work landed in this ticket's tasks (see `tasks:`). Resolved in the PdM's sweep of tickets whose tasks were all finished.
