---
id: br-6p6r
title: Why project role files drift from and override base instructions, and whether rules are applied and used well
kind: research
created: 2026-11-01
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

Relayed by orchestrator@vesta (m-0564 on dalek-home), from the human via aide on notes (notes m-0143, 2026-11-01 8:06 AM ET). The human, verbatim: "Yes, fix that, and this should come from the overall system instructions or the overall bridle instructions. I guess we're overriding that. I feel like the rules system we've built doesn't actually get work properly or get used properly."

What prompted it: the notes project's `.bridle/roles/aide.md` hard-coded `external:orchestrator@vesta`, which isn't a recipient on the notes daemon. That overrode the correct form base `aide.md` already gives (`--project <host>-home`). orchestrator@vesta fixes the notes file itself; this ticket is about the pattern.

Questions: (1) Why do per-project role files drift from base instructions and end up contradicting them? (2) Are rules actually applied and used well by agents (evidence: which rules do agents cite or break, from session logs)? (3) Directions to evaluate: a check (bridle doctor / ticket check style) that flags per-project role text contradicting base, or addresses in role files that are not valid recipients; letting project role files only add, never restate, base text; rendering the base text from one source.

Findings go in the ticket as a comment and, if lasting, in docs/design; a build that comes out of it is a new feature ticket linked `related`. Kind research, so it waits for the human's scoping before it is readied (rule ready-without-asking). Model: Sonnet. Out of scope: editing any project's role files (the owner of each project does that).
