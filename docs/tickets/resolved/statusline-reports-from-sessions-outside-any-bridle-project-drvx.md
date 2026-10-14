---
id: br-drvx
title: Statusline reports from sessions outside any bridle project are counted in a machine-level bucket (ktgd decision 2)
kind: feature
created: 2026-10-13
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: [br-mksa]
related: [br-ktgd]
theme: performance
parent: br-rdkn
resolved: 2026-10-14T08:36:48Z
---

Split from br-ktgd (held; it holds the human's words and decisions 1-7, read them first). Epic br-rdkn. The human, 2026-10-13 ~7:45 PM ET: "split up into nice little chunks."

## The ask

`bridle statusline` reports only to the daemon of the project the session works in and drops the rest. Send reports from sessions outside any bridle project to one daemon per machine (pick the simplest: e.g. a configured or default project), recorded as `outside any project`, so the human's other Claude Code sessions are counted. The desktop app and claude.ai stay uncounted (accepted).

Acceptance: a session in a non-bridle directory is recorded and shows in `bridle usage` external sessions under the bucket; `just check`. Docs: usage-and-budget.md. Size S.
