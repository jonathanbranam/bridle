---
id: br-mt4v
title: "Tests for eggc: one incident per land past the cap; a land refused during a drain runs after the restart; check nextest children survive the cap"
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

Follow-up to br-eggc (landed; PdM verification 2026-10-16 found test gaps). Tests first, fix only what a test shows.

- A land past the cap gives exactly one incident (not one per poll).
- A land refused during a drain runs after the restart.
- Check: the cap kills the check's process group, but nextest children in their own process groups may survive and be orphaned. Test it; if they survive, fix it (kill the tree) and say so on the thread.

Acceptance: just check. Model: Sonnet. Out of scope: redesigning the drain (br-d7r5 design stands).
