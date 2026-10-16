---
id: br-jkkp
title: "Tests for 36a3: role tokens rejected at start raise the incident; a random-token 401 is not counted (then nbgp too)"
kind: fix
created: 2026-10-16
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
resolved: 2026-10-16T21:08:11Z
---

## The ask

Follow-up to br-36a3 (landed without tests; PdM verification 2026-10-16). Tests only, no behaviour change unless a test finds a bug.

- A role token rejected at start raises the incident.
- A 401 from a random (unknown) token is not counted (the filter at crates/bridle-daemon server.rs around lines 543-557).
- Then the same kind of check for br-nbgp (read its ask; add the missing test if it is also untested).

Acceptance: just check. Model: Sonnet. Out of scope: changing the filter's design.
