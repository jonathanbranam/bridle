+++
id = "br-j86p"
title = "Same-machine orchestrator keeps direct tokens (visiting-orchestrator token is submit-only); ticket new --project writes into the wrong clone"
kind = "fix"
state = "claimed"
created_at = "2026-10-17T02:44:03.225Z"
updated_at = "2026-10-17T10:42:39.988499Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:j86p",
    "agent:j86pb",
]
priority = "critical"
priority_at = "2026-10-17T02:44:37.657490Z"
branch = "bridle/j86p"
commit = "46fb56d931b2762e1ad6609ba3bce3666446abbb"
summary = "Step 1 only. The machine's orchestrator keeps a DIRECT token on same-machine daemons that records a home (pair-mint --direct replaces --replace-direct and revokes the old visitor token; pair-check refuses a visitor token on the holder's own machine, so a re-pair swaps it). Daemon: token create accepts home without machine; forward_to_home and wake forward_home forward for a direct orchestrator that has a home (a direct one without a home still keeps wakes). Tests: token_pair repair sim, outbox_test direct orchestrator comments + wakes/mail forward + cross-machine visitor still refused. Docs: cli.md, orchestrator-supervision.md, CHANGELOG (migration: re-run token pair on dalek, vesta, remus). Step 2 (ticket new --project X clone, refused --from-task leaves file, --from-task re-link orphaning) not started."
ticket = "j86p"
+++

docs/tickets/open/same-machine-orchestrator-keeps-direct-tokens-visiting-orche-j86p.md

## Thread

### note · agent:pm-1 · 2026-10-17T02:44:08.830Z
Refile of br-cah9 (its row was linked to a deleted duplicate and could not be readied). Read `bridle ticket show br-cah9` for the full report, the aide's cause note, and the approvals: the human 2026-10-16 (via aide) "It sounds right yes. Go ahead and"; the human ~10:35 PM ET (via advisor) "Yes". CRITICAL, plan as queue tier 1. Model: Sonnet.

Problem: after `bridle token pair`, the same-machine orchestrator holds a VISITOR token on its own machine's daemons (token_pair.rs ~868, --replace-direct), so it cannot ready, plan, queue or comment there.

Step 1: on same-machine daemons the orchestrator keeps DIRECT tokens (full rights). A direct token can record a home so the daemon forwards its messages and wakes to <host>-home (extend the token record, reuse the br-gr3v forwarding, no second mechanism). No --replace-direct on the same machine; re-pair restores direct tokens where it replaced them. Visitors from other machines keep the visitor limits. Tests: same-machine orchestrator can ready and comment; its wakes still forward; a cross-machine visitor is still limited; re-pair replaces a visitor token with a direct one.
Step 2 (side bugs): `ticket new --project X` writes into X's clone or refuses; a refused `ticket new --from-task` leaves no file. Stop after step 1 and tell the manager if step 2 is large (own ticket).
Also file/fix: `ticket new --from-task` on a row that already has a ticket re-links the row, so dropping the duplicate orphans it.
Migration: re-running `bridle token pair` replaces visitor tokens with direct ones automatically; done note says the human re-runs pair on dalek, vesta, remus after the upgrade. Docs: cli.md, orchestrator role file, CHANGELOG. Acceptance: just check. Files: crates/bridle/src/token_pair.rs, daemon visitor/home forwarding. br-r8jf touches the same file: blocked-by this.

### note · external:advisor/product-manager · 2026-10-17T02:44:37.554Z
The human approved this fix twice on br-cah9 (to advisor product-manager ~10:35 PM ET: "Yes"; via aide: "It sounds right yes. Go ahead and"). Refiled because cah9's row links to a deleted duplicate. Readied by advisor (product-manager), priority critical.

### note · external:advisor/product-manager · 2026-10-17T02:44:37.657Z
priority: normal -> critical

### note · external:orchestrator@dalek · 2026-10-17T02:48:10.362Z
From orchestrator: the human, via aide (going to sleep): 'Try to get these tickets through!'. br-j86p is ready and critical. Plan it now (brief from the cah9 body, Sonnet) and put it at the front of the queue, behind only br-tvrp, which is already built and landing. Then keep the queue moving on the other ready tickets.

### note · external:orchestrator@dalek · 2026-10-17T02:48:10.422Z
From orchestrator: the human wants br-j86p built, verified and landed tonight. pm-1 is planning it now. Spawn a worker on it as soon as it's planned, ahead of everything else; tvrp lands first since it's built. Anything that needs the human goes on a --for-human ticket for the morning, not a blocking question.

### note · external:orchestrator@dalek · 2026-10-17T03:15:31.734Z
From orchestrator: br-j86p is already planned and queued (pm-1, 10:55 PM). Spawn its worker now; it shouldn't wait on the whce land. No release tonight: v0.7.6 is cut in the morning with j86p in it.

### note · agent:j86p · 2026-10-17T03:56:52.604Z
built: d23e983d563893bc67ff7a707b29b90bc78acf2c (branch bridle/j86p)

### note · agent:j86p · 2026-10-17T03:56:52.707Z
done step 1: just check exit 0, 1786 tests passed, main merged; d23e983d563893bc67ff7a707b29b90bc78acf2c. Step 2 (ticket new --project / --from-task side bugs) NOT done, deserves its own ticket.

### note · agent:manager-2 · 2026-10-17T03:56:59.442Z
split off br-ncbz: ticket new --project X writes into X's clone; a refused ticket new --from-task leaves no file

### note · agent:manager-2 · 2026-10-17T04:15:57.138Z
integrated: 46fb56d931b2762e1ad6609ba3bce3666446abbb (branch bridle/j86p)

### note · agent:manager-2 · 2026-10-17T04:16:23.801Z
cleanup: removed agent j86p, branch bridle/j86p

### note · external:advisor/product-manager · 2026-10-17T09:47:05.488Z
Checked by advisor (product-manager) 2026-10-17 ~5:50 AM ET: step 1 merged (46fb56d9) and complete in code. NOT resolving yet, gaps: (1) docs/design/agent-host/principals.md:191-193 still says a home is only for visitors and external:orchestrator stays local; (2) workflow/base/roles/orchestrator.md:44 still says 'orchestrator visitor token' (the brief asked for the role file); (3) no test that the same-machine orchestrator can READY (only comment). pm-1: a small follow-up under this ticket, please. Human: after the next release, re-run 'bridle token pair' and 'bridle token check' on dalek, vesta, remus.

### note · agent:pm-1 · 2026-10-17T09:47:13.646Z
Follow-up brief (pm-1), from the PdM's check. Model: Sonnet (small). Fix three gaps, nothing else:
(1) docs/design/agent-host/principals.md ~191-193: it still says a home is only for visitors and external:orchestrator stays local; update to the built behaviour (same-machine orchestrator keeps a direct token that records a home for forwarding).
(2) workflow/base/roles/orchestrator.md ~44: still says 'orchestrator visitor token'; reword to the direct-token-with-home behaviour (check the workflow copy rule for any generated copies).
(3) Add a test that the same-machine orchestrator can READY a ticket (only comment is tested today), next to the existing comment test.
Add a CHANGELOG line only if the docs change user-visible wording. Acceptance: just check. Out of scope: any code change beyond the test; br-ncbz side bugs.
