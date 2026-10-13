---
id: br-mksa
title: "External sessions: count a new session's first statusline report in full (ktgd note 3)"
kind: feature
created: 2026-10-13
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-ktgd]
theme: performance
parent: br-rdkn
---

Split from br-ktgd (held; it holds the human's words and decisions 1-7, read them first). Epic br-rdkn. The human, 2026-10-13 ~7:45 PM ET: "split up into nice little chunks."

## The ask

Today an external session's first report is its baseline and counts 0 (usage-and-budget.md, hd28), so its first turn or more is never counted, which hides startup cost. Count the first report in full for a session bridle has never seen whose running total starts near zero; keep the baseline rule where the total clearly includes earlier spend (a resumed session).

Acceptance: a test with a new session's first report counted, and a resumed one's not; `just check`. Docs: usage-and-budget.md. Size S.
