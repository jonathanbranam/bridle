---
id: br-8rdn
title: "7r2h gaps: token pair on a machine with no home keeps direct tokens; make re-pair a prompt step; startup check names projects that do not forward"
kind: fix
created: 2026-10-16
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

Follow-up to br-7r2h (landed; PdM verification 2026-10-16). Read the 7r2h and br-gr3v threads.

- Item 2 interim: on a machine with no <machine>-home (dalek today) `bridle token pair` keeps direct orchestrator tokens, so bridle-ui and track-web do not forward, and the orchestrator prompt gives no instruction for them. The prompt must say what to do for those projects, and make "re-run bridle token pair after the home exists" an explicit step.
- Item 3 (not built): a startup check or status field naming the projects that do not forward to the home daemon.

Files likely: the orchestrator role file under workflow/, token_pair.rs, status code. Acceptance: just check. Model: Sonnet. Out of scope: new forwarding mechanisms (PdM decision first).
