---
id: 5u9d
title: "External review of the workflow system: what it does today and what's missing"
kind: research
opened: 2026-10-03
repos: [bridle]
changes: []
specs: []
needs: []
see: [mrhe, 34bw, vp9e]
tasks: [br-d207]
closed: 2026-10-09T23:11:08Z
---

## The ask


The human, verbatim (2026-10-02, via advisor workflow):

> Yeah, I sent a separate Claude instance, like from the web, to analyze this project, and they
> came back with a bunch of limitations, talking about how, kind of how weak the implementation
> was, and I can't figure out if they misunderstood the code base for some weird reason, or if
> their analysis was accurate.

> Copy this file as written into the repo as well. Create a ticket maybe research idk I don't
> remember the ticket kinds. Then create a new ticket where you report your analysis.

The review, copied as written: [[docs/research/workflow-review-2026-10-02|workflow review, 2026-10-02]]
(read at commit `be2440d`). Its summary:

> Bridle's workflow system today is a strict, well-tested rules engine, but almost nothing
> connects it to the agents bridle runs. […] A pack rule or project override changes what
> `bridle rules explain` reports, not what a worker is told.

The advisor's check of it against the code is
[[the-workflow-doesn-t-reach-agents-resolved-rules-hooks-and-o-34bw|34bw]].

## Resolution

Resolved 2026-10-09 by advisor (product-manager). The analysis is in 34bw (the external review checked against the code); the review is at docs/research/workflow-review-2026-10-02. br-d207 dropped because 34bw's tasks act on it.
