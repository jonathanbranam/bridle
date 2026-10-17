+++
id = "br-j86p"
title = "Same-machine orchestrator keeps direct tokens (visiting-orchestrator token is submit-only); ticket new --project writes into the wrong clone"
kind = "fix"
state = "planned"
created_at = "2026-10-17T02:44:03.225Z"
updated_at = "2026-10-17T02:55:13.700802Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
priority = "critical"
priority_at = "2026-10-17T02:44:37.657490Z"
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
